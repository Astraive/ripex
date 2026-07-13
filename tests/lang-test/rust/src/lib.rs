//! ripex-lang-test: Rust crate root — module declarations, re-exports.
pub mod models;
pub mod utils;
pub mod services;

pub use models::user::User;
pub use models::product::Product;

pub fn run_demo() {
    let alice = User::new("Alice", "alice@example.com");
    alice.roles.push("admin".to_string());
    println!("{}", alice.describe());
    println!("admin? {}", alice.is_admin());

    let widget = crate::models::product::make_widget();
    println!("tax={}", widget.calculate_tax());

    println!("{}", utils::greet(&alice.name));
    println!("pos={:?}", utils::filter_positive(&[-1, 2, 3]));
}
