//! Objects can nest hundreds of levels deep (469 in one Workshop level), more than a 1 MB stack
//! holds when reading them recursively, so recursion grows the stack as needed.

/// Runs `f` with enough stack left for one more level of nesting.
pub(crate) fn grow<R>(f: impl FnOnce() -> R) -> R {
    #[cfg(not(target_family = "wasm"))]
    {
        stacker::maybe_grow(256 * 1024, 4 * 1024 * 1024, f)
    }
    #[cfg(target_family = "wasm")]
    {
        f()
    }
}
