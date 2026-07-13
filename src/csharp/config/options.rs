#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSharpVersion {
    CSharp7,
    CSharp8,
    CSharp9,
    CSharp10,
    CSharp11,
    CSharp12,
    CSharp13,
    Latest,
}

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub version: CSharpVersion,
    pub capture_comments: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        ParserOptions {
            version: CSharpVersion::Latest,
            capture_comments: true,
        }
    }
}
