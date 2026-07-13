//! ripex-lang-test: Rust string utils — closures, lifetimes.
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub fn mask_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    format!("{}***@{}", &parts[0][..1], parts[1])
}

pub fn filter_positive<'a>(xs: &'a [i32]) -> Vec<i32> {
    xs.iter().filter(|&&x| x > 0).copied().collect()
}
