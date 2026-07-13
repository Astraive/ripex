#[derive(Clone, Copy, Debug)]
pub struct SyntaxFeatures {
    pub const_generics: bool,
    pub impl_trait: bool,
    pub async_fn: bool,
    pub generic_associated_types: bool,
}

impl SyntaxFeatures {
    pub fn new() -> Self {
        SyntaxFeatures {
            const_generics: false,
            impl_trait: false,
            async_fn: false,
            generic_associated_types: false,
        }
    }
    pub fn all() -> Self {
        SyntaxFeatures {
            const_generics: true,
            impl_trait: true,
            async_fn: true,
            generic_associated_types: true,
        }
    }
}
impl Default for SyntaxFeatures {
    fn default() -> Self {
        Self::new()
    }
}
