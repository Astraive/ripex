//! ripex-lang-test: Rust math — functions, generics, macros (gap: body dropped).
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn squares(xs: &[i32]) -> Vec<i32> {
    xs.iter().map(|x| x * x).collect()
}

pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// Macro definition — body content is a known silent drop in ripex.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("[INFO] {}", format_args!($($arg)*))
    };
}
