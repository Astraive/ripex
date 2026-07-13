#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub in_loop: bool,
    pub in_function: bool,
    pub in_unsafe: bool,
    pub in_async: bool,
}

impl Context {
    pub fn new() -> Self {
        Context {
            in_loop: false,
            in_function: false,
            in_unsafe: false,
            in_async: false,
        }
    }
}
impl Default for Context {
    fn default() -> Self {
        Context::new()
    }
}
