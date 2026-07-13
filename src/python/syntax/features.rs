#[derive(Clone, Copy, Debug)]
pub struct SyntaxFeatures {
    pub match_stmt: bool,
    pub walrus: bool,
    pub exception_group: bool,
    pub type_params: bool,
    pub free_threaded: bool,
}

impl SyntaxFeatures {
    pub fn new() -> Self {
        SyntaxFeatures {
            match_stmt: false,
            walrus: false,
            exception_group: false,
            type_params: false,
            free_threaded: false,
        }
    }

    pub fn all() -> Self {
        SyntaxFeatures {
            match_stmt: true,
            walrus: true,
            exception_group: true,
            type_params: true,
            free_threaded: true,
        }
    }
}

impl Default for SyntaxFeatures {
    fn default() -> Self {
        Self::new()
    }
}
