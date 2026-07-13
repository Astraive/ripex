#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    Ed2015,
    Ed2018,
    Ed2021,
    Ed2024,
    Latest,
}

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub edition: Edition,
    pub capture_comments: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        ParserOptions {
            edition: Edition::Latest,
            capture_comments: true,
        }
    }
}
