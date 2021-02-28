pub(crate) fn hex_dump<T: AsRef<[u8]>>(source: &T) -> String {
    pretty_hex::pretty_hex(&source.as_ref())
}
