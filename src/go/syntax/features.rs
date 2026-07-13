#[derive(Clone, Copy, Debug)]
pub struct SyntaxFeatures {
    pub generics: bool,
    pub error_handling: bool,
}

impl SyntaxFeatures {
    pub fn new() -> Self {
        SyntaxFeatures {
            generics: false,
            error_handling: false,
        }
    }

    pub fn all() -> Self {
        SyntaxFeatures {
            generics: true,
            error_handling: true,
        }
    }
}

impl Default for SyntaxFeatures {
    fn default() -> Self {
        SyntaxFeatures::new()
    }
}
