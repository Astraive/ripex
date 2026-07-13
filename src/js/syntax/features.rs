#[derive(Clone, Copy, Debug)]
pub struct SyntaxFeatures {
    pub jsx: bool,
    pub typescript: bool,
    pub decorators: bool,
    pub import_attributes: bool,
    pub explicit_resource_management: bool,
    pub regexp_v_flag: bool,
    pub decimal: bool,
}

impl SyntaxFeatures {
    pub fn new() -> Self {
        SyntaxFeatures {
            jsx: false,
            typescript: false,
            decorators: false,
            import_attributes: false,
            explicit_resource_management: false,
            regexp_v_flag: false,
            decimal: false,
        }
    }

    pub fn all() -> Self {
        SyntaxFeatures {
            jsx: true,
            typescript: true,
            decorators: true,
            import_attributes: true,
            explicit_resource_management: true,
            regexp_v_flag: true,
            decimal: true,
        }
    }
}

impl Default for SyntaxFeatures {
    fn default() -> Self {
        SyntaxFeatures::new()
    }
}
