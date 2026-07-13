//! ripex-lang-test: Rust Product — enum, generic impl, const.
#[derive(Debug)]
pub enum Category {
    Electronics,
    Clothing,
}

pub struct Product<T> {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub category: T,
}

impl<T> Product<T> {
    pub const TAX_RATE: f64 = 0.1;

    pub fn calculate_tax(&self) -> f64 {
        self.price * Self::TAX_RATE
    }
}

pub fn make_widget() -> Product<Category> {
    Product {
        id: 1,
        name: "Widget".to_string(),
        price: 19.99,
        category: Category::Electronics,
    }
}
