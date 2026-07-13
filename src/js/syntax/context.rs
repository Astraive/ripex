#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub strict_mode: bool,
    pub is_module: bool,
    pub allow_await: bool,
    pub allow_yield: bool,
    pub allow_super: bool,
    pub allow_new_target: bool,
    pub in_function: bool,
    pub in_constructor: bool,
    pub in_ternary: u32,
    pub in_arrow_head: bool,
}

impl Context {
    pub fn new() -> Self {
        Context {
            strict_mode: false,
            is_module: false,
            allow_await: false,
            allow_yield: false,
            allow_super: false,
            allow_new_target: false,
            in_function: false,
            in_constructor: false,
            in_ternary: 0,
            in_arrow_head: false,
        }
    }

    pub fn script() -> Self {
        Context::new()
    }

    pub fn module() -> Self {
        Context {
            is_module: true,
            strict_mode: true,
            allow_await: true,
            ..Context::new()
        }
    }

    pub fn enter_function(self) -> Self {
        Context {
            in_function: true,
            allow_super: false,
            allow_new_target: true,
            ..self
        }
    }

    pub fn enter_constructor(self) -> Self {
        Context {
            in_constructor: true,
            allow_super: true,
            allow_new_target: true,
            ..self
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new()
    }
}
