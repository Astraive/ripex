use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ident,
    Package,
    Import,
    Func,
    Var,
    Const,
    Type,
    Struct,
    Interface,
    Map,
    Chan,
    Defer,
    Go,
    Select,
    Case,
    Switch,
    If,
    Else,
    For,
    Range,
    Break,
    Continue,
    Return,
    Fallthrough,
    Default,
    Goto,
    Nil,
    True,
    False,
    Iota,
    String,
    Int8,
    Int16,
    Int32,
    Int64,
    Int,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint,
    Uintptr,
    Float32,
    Float64,
    Complex64,
    Complex128,
    Byte,
    Rune,
    Bool,
    Any,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Ampersand,
    Pipe,
    Caret,
    LtLt,
    GtGt,
    AmpersandCaret,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    AmpersandEq,
    PipeEq,
    CaretEq,
    LtLtEq,
    GtGtEq,
    AmpersandCaretEq,
    AmpersandAmpersand,
    PipePipe,
    Lt,
    Gt,
    EqEq,
    Ne,
    LtEq,
    GtEq,
    Eq,
    Exclamation,
    Dot,
    DotDotDot,
    Comma,
    Semicolon,
    Colon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Arrow,
    Define,
    PlusPlus,
    MinusMinus,
    Tilde,

    RawString,
    InterpretedString,
    RuneLit,
    IntLit,
    FloatLit,
    ImagLit,

    Newline,
    Eof,
    Error,
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Package
                | TokenKind::Import
                | TokenKind::Func
                | TokenKind::Var
                | TokenKind::Const
                | TokenKind::Type
                | TokenKind::Struct
                | TokenKind::Interface
                | TokenKind::Map
                | TokenKind::Chan
                | TokenKind::Defer
                | TokenKind::Go
                | TokenKind::Select
                | TokenKind::Case
                | TokenKind::Switch
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::For
                | TokenKind::Range
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Return
                | TokenKind::Fallthrough
                | TokenKind::Default
                | TokenKind::Goto
                | TokenKind::Nil
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Iota
                | TokenKind::String
                | TokenKind::Int
                | TokenKind::Int8
                | TokenKind::Int16
                | TokenKind::Int32
                | TokenKind::Int64
                | TokenKind::Uint
                | TokenKind::Uint8
                | TokenKind::Uint16
                | TokenKind::Uint32
                | TokenKind::Uint64
                | TokenKind::Uintptr
                | TokenKind::Float32
                | TokenKind::Float64
                | TokenKind::Complex64
                | TokenKind::Complex128
                | TokenKind::Byte
                | TokenKind::Rune
                | TokenKind::Bool
                | TokenKind::Any
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub value: String,
    pub leading_comments: Vec<Comment>,
    pub has_line_break: bool,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token {
            kind,
            span,
            value: String::new(),
            leading_comments: Vec::new(),
            has_line_break: false,
        }
    }
    pub fn with_value(kind: TokenKind, span: Span, value: impl Into<String>) -> Self {
        Token {
            kind,
            span,
            value: value.into(),
            leading_comments: Vec::new(),
            has_line_break: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub kind: CommentKind,
    pub span: Span,
    pub text: String,
}

impl Comment {
    pub fn new(kind: CommentKind, span: Span, text: impl Into<String>) -> Self {
        Comment {
            kind,
            span,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Line,
    Block,
}
