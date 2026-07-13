#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonVersion {
    Py37,
    Py38,
    Py39,
    Py310,
    Py311,
    Py312,
    Py313,
    Latest,
}

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub version: PythonVersion,
    pub capture_comments: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        ParserOptions {
            version: PythonVersion::Latest,
            capture_comments: true,
        }
    }
}
