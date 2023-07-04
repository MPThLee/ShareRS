use infer::Type;

pub fn get_mime(buf: &[u8]) -> Type {
    infer::get(buf).unwrap_or(Type::new(
        infer::MatcherType::App,
        "application/octet-stream",
        "bin",
        |_| true,
    ))
}
