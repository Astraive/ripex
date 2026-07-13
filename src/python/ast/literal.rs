use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64, String, Span),
    Float(f64, String, Span),
    Complex {
        real: f64,
        imag: f64,
        text: String,
        span: Span,
    },
    String(String, String, Span),
    Bytes(Vec<u8>, String, Span),
    Boolean(bool, Span),
    None_(Span),
    Ellipsis(Span),
}
