//! ripex-lang-test: Rust User model — struct, impl, trait, derive macro.
use std::fmt;

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
}

pub trait Describe {
    fn describe(&self) -> String;
}

impl Describe for User {
    fn describe(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }
}

impl User {
    pub fn new(name: &str, email: &str) -> Self {
        Self {
            name: name.to_string(),
            email: email.to_string(),
            roles: Vec::new(),
        }
    }

    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.describe())
    }
}
