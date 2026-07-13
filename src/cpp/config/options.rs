#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CppStandard {
    Cpp98,
    Cpp11,
    Cpp14,
    Cpp17,
    Cpp20,
    Cpp23,
    Latest,
}

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub standard: CppStandard,
    pub capture_comments: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        ParserOptions {
            standard: CppStandard::Latest,
            capture_comments: true,
        }
    }
}
