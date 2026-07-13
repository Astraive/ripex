#[derive(Clone, Copy, Debug)]
pub struct SyntaxFeatures {
    pub concepts: bool,
    pub ranges: bool,
    pub coroutines: bool,
    pub modules: bool,
}

impl SyntaxFeatures {
    pub fn new() -> Self {
        SyntaxFeatures {
            concepts: false,
            ranges: false,
            coroutines: false,
            modules: false,
        }
    }
    pub fn all() -> Self {
        SyntaxFeatures {
            concepts: true,
            ranges: true,
            coroutines: true,
            modules: true,
        }
    }
}
impl Default for SyntaxFeatures {
    fn default() -> Self {
        Self::new()
    }
}
