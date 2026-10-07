using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Net;
using System.Reflection;
using System.Text;
using BepInEx;
using HarmonyLib;
using Serializers;
using UnityEngine;

namespace BytesTrace;

/// <summary>
/// Runs the game as an HTTP server that loads posted .bytes files with the game's own
/// BinaryDeserializer and returns every value it read, as JSON lines.
///
/// Standalone: don't install alongside BytesServer, whose level-settings cleanup would destroy
/// this plugin's level. The Steam and splash-screen patches are copied from BytesServer.
/// </summary>
[BepInPlugin("BytesTrace", "BytesTrace", "0.1.0")]
public class BytesTracePlugin : BaseUnityPlugin
{
    private static readonly Queue<HttpListenerContext> Requests = new();
    private static Level _level;

    private void Awake()
    {
        var harmony = new Harmony("BytesTrace");
        harmony.PatchAll(typeof(GamePatches));
        Tracer.Patch(harmony);
    }

    private void Start()
    {
        Application.targetFrameRate = 100;
        _level = new Level(true);

        var port = 18501;
        if (int.TryParse(Environment.GetEnvironmentVariable("BYTES_TRACE_PORT"), out var p) && p is > 0 and < 65536)
            port = p;

        var listener = new HttpListener();
        listener.Prefixes.Add($"http://localhost:{port}/");
        listener.Start();
        listener.BeginGetContext(OnRequest, listener);
        Console.WriteLine($"BytesTrace listening on port {port}.");
    }

    private static void OnRequest(IAsyncResult result)
    {
        var listener = (HttpListener)result.AsyncState;
        HttpListenerContext context;
        try
        {
            context = listener.EndGetContext(result);
        }
        finally
        {
            listener.BeginGetContext(OnRequest, listener);
        }

        lock (Requests)
            Requests.Enqueue(context);
    }

    // Requests are handled on the main thread because they use Unity APIs.
    private void Update()
    {
        HttpListenerContext context;
        lock (Requests)
        {
            if (Requests.Count == 0)
                return;
            context = Requests.Dequeue();
        }

        var response = context.Response;
        try
        {
            Handle(context.Request, response);
        }
        catch (Exception e)
        {
            Debug.LogException(e);
            response.StatusCode = (int)HttpStatusCode.InternalServerError;
        }
        finally
        {
            response.Close();
        }
    }

    private static void Handle(HttpListenerRequest request, HttpListenerResponse response)
    {
        var path = request.Url.AbsolutePath;
        if (path == "/ping")
        {
            Respond(response, "OK");
            return;
        }

        if (request.HttpMethod != "POST")
        {
            response.StatusCode = (int)HttpStatusCode.MethodNotAllowed;
            return;
        }

        var data = ReadBody(request);
        switch (path)
        {
            case "/level-bytes-trace":
            {
                bool ok;
                Tracer.Begin(data);
                try
                {
                    ok = _level.LoadFromBytes(data, true, false);
                }
                finally
                {
                    Tracer.End();
                }

                Tracer.Result(ok);
                Respond(response, Tracer.Output);
                _level.ClearAndReset(true);
                Resource.UnloadUnusedAssets();
                break;
            }
            case "/gameobject-bytes-trace":
            {
                GameObject obj;
                Tracer.Begin(data);
                try
                {
                    obj = Deserializer.LoadGameObjectFromBytes<BinaryDeserializer>(data);
                }
                finally
                {
                    Tracer.End();
                }

                Tracer.Result(obj != null);
                Respond(response, Tracer.Output);
                if (obj != null)
                    Destroy(obj);
                Resource.UnloadUnusedAssets();
                break;
            }
            default:
                response.StatusCode = (int)HttpStatusCode.NotFound;
                break;
        }
    }

    private static byte[] ReadBody(HttpListenerRequest request)
    {
        using var memory = new MemoryStream();
        var buffer = new byte[65536];
        int n;
        while ((n = request.InputStream.Read(buffer, 0, buffer.Length)) > 0)
            memory.Write(buffer, 0, n);
        return memory.ToArray();
    }

    private static void Respond(HttpListenerResponse response, string body)
    {
        var bytes = Encoding.UTF8.GetBytes(body);
        response.ContentType = "application/x-ndjson; charset=utf-8";
        response.ContentLength64 = bytes.Length;
        response.OutputStream.Write(bytes, 0, bytes.Length);
    }

    private static class GamePatches
    {
        [HarmonyPatch(typeof(GSystemsManager), "Awake")]
        [HarmonyPrefix]
        private static void DisableSteam(GSystemsManager __instance)
        {
            __instance.managerPrefabsToCreate_ = __instance.managerPrefabsToCreate_
                .Where(obj => obj.GetComponentsInChildren<SteamworksManager>(true).Length == 0)
                .ToArray();
        }

        [HarmonyPatch(typeof(SplashScreenLogic), "Awake")]
        [HarmonyPrefix]
        private static bool StopSplashScreenProgressing(SplashScreenLogic __instance)
        {
            DestroyImmediate(__instance.gameObject);
            return false;
        }

        [HarmonyPatch(typeof(Level), "UpgradeLevelVersion")]
        [HarmonyPostfix]
        private static void FixLevelSettingLeak()
        {
            foreach (var settings in FindObjectsOfType<LevelSettings>())
            {
                if (settings != _level.Settings_ && settings != G.Sys.GameManager_.LevelSettings_)
                    Destroy(settings.gameObject);
            }
        }

        [HarmonyPatch(typeof(Serializer), nameof(Serializer.GetPrefabWithName))]
        [HarmonyPrefix]
        private static bool AllowLoadingReplaysWithoutSpecifyingPrefab(ref GameObject __result, string prefabName)
        {
            if (!prefabName.StartsWith("Replay:"))
                return true;
            __result = Resource.LoadPrefab("CarReplayData", false);
            return false;
        }
    }
}

/// <summary>
/// Harmony patches on every BinaryDeserializer method that reads from the stream. Each event is
/// one JSON object per line with the absolute offset "o" and length "l" of what was read.
///
/// When the game reads past the end of a scope and then seeks back to its end, the events beyond
/// the end are marked "overread": they come from misinterpreting the following bytes.
/// </summary>
internal static class Tracer
{
    private struct Event
    {
        public long Offset;
        public long Length;
        public string Body;
        public bool IsScope;
        public bool Overread;
    }

    private static bool _active;
    private static byte[] _data = new byte[0];
    private static List<Event> _events = new();

    public static string Output
    {
        get
        {
            var sb = new StringBuilder(_events.Count * 64);
            foreach (var e in _events)
            {
                sb.Append("{\"o\":").Append(e.Offset).Append(",\"l\":").Append(e.Length).Append(',').Append(e.Body);
                if (e.Overread)
                    sb.Append(",\"overread\":true");
                sb.Append("}\n");
            }
            return sb.ToString();
        }
    }

    // Deserializers reading the posted data. While loading, the game also deserializes other
    // data (e.g. resources), which must not be traced.
    private static readonly List<Deserializer> Traced = new();

    public static void Begin(byte[] data)
    {
        _data = data;
        _events = new List<Event>();
        Traced.Clear();
        _active = true;
    }

    public static void End()
    {
        _active = false;
        Traced.Clear();
    }

    private static bool IsTraced(Deserializer d)
    {
        if (!_active)
            return false;
        foreach (var t in Traced)
        {
            if (ReferenceEquals(t, d))
                return true;
        }
        return false;
    }

    private static void PreLoadFromBytes(Deserializer __instance, byte[] bytes)
    {
        if (_active && ReferenceEquals(bytes, _data))
            Traced.Add(__instance);
    }

    public static void Result(bool ok) => Add(0, 0, "\"t\":\"result\",\"ok\":" + (ok ? "true" : "false"));

    private static void Add(long offset, long length, string body, bool isScope = false) =>
        _events.Add(new Event { Offset = offset, Length = length, Body = body, IsScope = isScope });

    public static void Patch(Harmony harmony)
    {
        var type = typeof(BinaryDeserializer);
        var pre = new HarmonyMethod(typeof(Tracer), nameof(SavePos));

        void PatchReadSet(Type valueType, string postfix)
        {
            var method = AccessTools.Method(type, "ReadSet", [typeof(string), valueType.MakeByRefType()])
                ?? throw new Exception($"ReadSet({valueType}) not found");
            harmony.Patch(method, pre, new HarmonyMethod(typeof(Tracer), postfix));
        }

        PatchReadSet(typeof(string), nameof(PostString));
        PatchReadSet(typeof(int), nameof(PostInt));
        PatchReadSet(typeof(uint), nameof(PostUInt));
        PatchReadSet(typeof(short), nameof(PostShort));
        PatchReadSet(typeof(ushort), nameof(PostUShort));
        PatchReadSet(typeof(long), nameof(PostLong));
        PatchReadSet(typeof(bool), nameof(PostBool));
        PatchReadSet(typeof(char), nameof(PostChar));
        PatchReadSet(typeof(float), nameof(PostFloat));
        PatchReadSet(typeof(double), nameof(PostDouble));

        harmony.Patch(AccessTools.Method(type, "VisitGenericObject"), pre, new HarmonyMethod(typeof(Tracer), nameof(PostGeneric)));
        harmony.Patch(AccessTools.Method(type, "ReadBytes"), pre, new HarmonyMethod(typeof(Tracer), nameof(PostBytes)));
        harmony.Patch(AccessTools.Method(type, "Empty"), pre, new HarmonyMethod(typeof(Tracer), nameof(PostEmpty)));
        harmony.Patch(AccessTools.Method(type, "ReadStartScopeHelper"), new HarmonyMethod(typeof(Tracer), nameof(PreScopeStart)));
        harmony.Patch(AccessTools.Method(type, "ReadEndScopeHelper"), new HarmonyMethod(typeof(Tracer), nameof(PreScopeEnd)));

        foreach (var load in AccessTools.GetDeclaredMethods(typeof(Deserializer)).Where(m =>
                     m.Name == "LoadFromBytes" && m.GetParameters().FirstOrDefault()?.ParameterType == typeof(byte[])))
            harmony.Patch(load, new HarmonyMethod(typeof(Tracer), nameof(PreLoadFromBytes)));
    }

    private static long Pos(BinaryDeserializer d) => d.reader_.BaseStream.Position;

    private static void SavePos(BinaryDeserializer __instance, out long __state) =>
        __state = IsTraced(__instance) ? Pos(__instance) : 0;

    private static void Emit(BinaryDeserializer d, long start, string type, string name, string value)
    {
        if (IsTraced(d))
            Add(start, Pos(d) - start, "\"t\":\"" + type + "\",\"n\":" + Json(name) + ",\"v\":" + value);
    }

    private static void PostString(BinaryDeserializer __instance, long __state, string name, ref string val) =>
        Emit(__instance, __state, "string", name, Json(val));

    private static void PostInt(BinaryDeserializer __instance, long __state, string name, ref int val) =>
        Emit(__instance, __state, "int", name, val.ToString(CultureInfo.InvariantCulture));

    private static void PostUInt(BinaryDeserializer __instance, long __state, string name, ref uint val) =>
        Emit(__instance, __state, "uint", name, val.ToString(CultureInfo.InvariantCulture));

    private static void PostShort(BinaryDeserializer __instance, long __state, string name, ref short val) =>
        Emit(__instance, __state, "short", name, val.ToString(CultureInfo.InvariantCulture));

    private static void PostUShort(BinaryDeserializer __instance, long __state, string name, ref ushort val) =>
        Emit(__instance, __state, "ushort", name, val.ToString(CultureInfo.InvariantCulture));

    private static void PostLong(BinaryDeserializer __instance, long __state, string name, ref long val) =>
        Emit(__instance, __state, "long", name, val.ToString(CultureInfo.InvariantCulture));

    private static void PostBool(BinaryDeserializer __instance, long __state, string name, ref bool val) =>
        Emit(__instance, __state, "bool", name, val ? "true" : "false");

    private static void PostChar(BinaryDeserializer __instance, long __state, string name, ref char val) =>
        Emit(__instance, __state, "char", name, ((int)val).ToString(CultureInfo.InvariantCulture));

    // Floats are emitted as their bit patterns so comparisons are exact.
    private static void PostFloat(BinaryDeserializer __instance, long __state, string name, ref float val) =>
        Emit(__instance, __state, "float", name, FloatBits(val));

    private static void PostDouble(BinaryDeserializer __instance, long __state, string name, ref double val) =>
        Emit(__instance, __state, "double", name, BitConverter.DoubleToInt64Bits(val).ToString(CultureInfo.InvariantCulture));

    private static void PostGeneric(BinaryDeserializer __instance, long __state, string name, Type type, object __result)
    {
        var value = __result switch
        {
            null => "null",
            string s => Json(s),
            float f => FloatBits(f),
            double d => BitConverter.DoubleToInt64Bits(d).ToString(CultureInfo.InvariantCulture),
            bool b => b ? "true" : "false",
            char c => ((int)c).ToString(CultureInfo.InvariantCulture),
            IFormattable f => f.ToString(null, CultureInfo.InvariantCulture),
            _ => Json(__result.ToString()),
        };
        Emit(__instance, __state, "generic:" + type.FullName, name, value);
    }

    private static void PostBytes(BinaryDeserializer __instance, long __state, string name, ref byte[] val) =>
        Emit(__instance, __state, "bytes", name, (val?.Length ?? 0).ToString(CultureInfo.InvariantCulture));

    private static void PostEmpty(BinaryDeserializer __instance, long __state, bool __result)
    {
        if (__result)
            Emit(__instance, __state, "empty", "", "null");
    }

    // Called after the scope mark was read, with the stream at the scope length.
    private static void PreScopeStart(BinaryDeserializer __instance, int mark)
    {
        if (!IsTraced(__instance))
            return;
        var pos = Pos(__instance);
        var len = pos + 8 <= _data.Length ? BitConverter.ToInt64(_data, (int)pos) : -1;
        Add(pos - 4, 12, "\"t\":\"scope\",\"mark\":" + mark + ",\"len\":" + len, isScope: true);
    }

    private static readonly Type ScopeInfoType = AccessTools.Inner(typeof(BinaryDeserializer), "ScopeInfo");
    private static readonly AccessTools.FieldRef<object, long> ScopeStartPos = AccessTools.FieldRefAccess<long>(ScopeInfoType, "startPos_");
    private static readonly AccessTools.FieldRef<object, long> ScopeEndPos = AccessTools.FieldRefAccess<long>(ScopeInfoType, "endPos_");

    // Logged only when the game didn't read a scope exactly to its end and seeks to its end.
    private static void PreScopeEnd(BinaryDeserializer __instance, object scopeInfo)
    {
        if (!IsTraced(__instance))
            return;
        var pos = Pos(__instance);
        var end = ScopeEndPos(scopeInfo);
        if (pos == end)
            return;
        if (pos > end)
        {
            // Mark everything read past the end since this scope started.
            var headerOffset = ScopeStartPos(scopeInfo) - 12;
            var i = _events.Count - 1;
            while (i >= 0 && !(_events[i].IsScope && _events[i].Offset == headerOffset))
                i--;
            for (var j = i + 1; j < _events.Count; j++)
            {
                var e = _events[j];
                if (e.Offset + e.Length > end)
                {
                    e.Overread = true;
                    _events[j] = e;
                }
            }
        }
        Add(pos, 0, "\"t\":\"skip\",\"end\":" + end);
    }

    private static string FloatBits(float f) =>
        BitConverter.ToInt32(BitConverter.GetBytes(f), 0).ToString(CultureInfo.InvariantCulture);

    private static string Json(string s)
    {
        if (s == null)
            return "null";
        var sb = new StringBuilder(s.Length + 2).Append('"');
        foreach (var c in s)
        {
            switch (c)
            {
                case '"': sb.Append("\\\""); break;
                case '\\': sb.Append("\\\\"); break;
                case '\n': sb.Append("\\n"); break;
                case '\r': sb.Append("\\r"); break;
                case '\t': sb.Append("\\t"); break;
                default:
                    if (c < 0x20 || char.IsSurrogate(c))
                        sb.Append("\\u").Append(((int)c).ToString("x4"));
                    else
                        sb.Append(c);
                    break;
            }
        }
        return sb.Append('"').ToString();
    }
}
