#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CStandard {
    C89,
    C99,
    C11,
    C17,
    C23,
    Latest,
}

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub standard: CStandard,
    pub capture_comments: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        ParserOptions {
            standard: CStandard::Latest,
            capture_comments: true,
        }
    }
}
