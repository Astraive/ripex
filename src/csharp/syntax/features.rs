#[derive(Clone, Copy, Debug)]
pub struct SyntaxFeatures {
    pub nullable: bool,
    pub records: bool,
    pub primary_ctors: bool,
    pub init_only: bool,
}

impl SyntaxFeatures {
    pub fn new() -> Self {
        SyntaxFeatures {
            nullable: false,
            records: false,
            primary_ctors: false,
            init_only: false,
        }
    }
    pub fn all() -> Self {
        SyntaxFeatures {
            nullable: true,
            records: true,
            primary_ctors: true,
            init_only: true,
        }
    }
}
impl Default for SyntaxFeatures {
    fn default() -> Self {
        Self::new()
    }
}
