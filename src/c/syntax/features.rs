#[derive(Clone, Copy, Debug)]
pub struct SyntaxFeatures {
    pub bool_: bool,
    pub complex: bool,
    pub atomic: bool,
    pub generic: bool,
}

impl SyntaxFeatures {
    pub fn new() -> Self {
        SyntaxFeatures {
            bool_: false,
            complex: false,
            atomic: false,
            generic: false,
        }
    }

    pub fn all() -> Self {
        SyntaxFeatures {
            bool_: true,
            complex: true,
            atomic: true,
            generic: true,
        }
    }
}

impl Default for SyntaxFeatures {
    fn default() -> Self {
        SyntaxFeatures::new()
    }
}
