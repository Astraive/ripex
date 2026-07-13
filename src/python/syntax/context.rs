#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub in_loop: bool,
    pub in_function: bool,
    pub in_class: bool,
    pub in_comprehension: bool,
    pub in_async: bool,
    pub in_try: bool,
}

impl Context {
    pub fn new() -> Self {
        Context {
            in_loop: false,
            in_function: false,
            in_class: false,
            in_comprehension: false,
            in_async: false,
            in_try: false,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new()
    }
}
