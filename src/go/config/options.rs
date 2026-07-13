#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoVersion {
    Go1,
    Go1_18,
    Go1_19,
    Go1_20,
    Go1_21,
    Go1_22,
    Go1_23,
    Latest,
}

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub version: GoVersion,
    pub capture_comments: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        ParserOptions {
            version: GoVersion::Latest,
            capture_comments: true,
        }
    }
}
