use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ident,
    Auto,
    Break,
    Case,
    Char,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    Float,
    For,
    Goto,
    If,
    Inline,
    Int,
    Long,
    Register,
    Restrict,
    Return,
    Short,
    Signed,
    Sizeof,
    Static,
    Struct,
    Switch,
    Typedef,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,
    Bool,
    Complex,
    Imaginary,
    Alignas,
    Alignof,
    Atomic,
    Generic,
    Noreturn,
    StaticAssert,
    ThreadLocal,
    True,
    False,
    Null,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PlusPlus,
    MinusMinus,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    LtLt,
    GtGt,
    EqEq,
    Ne,
    Lt,
    Gt,
    LtEq,
    GtEq,
    AmpersandAmpersand,
    PipePipe,
    Exclamation,
    Question,
    Colon,
    Semicolon,
    Comma,
    Dot,
    Arrow,
    DotDotDot,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eq,
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
    Hash,
    HashHash,
    Newline,

    IntLit,
    UIntLit,
    LongLit,
    ULongLit,
    LongLongLit,
    ULongLongLit,
    FloatLit,
    DoubleLit,
    HexFloatLit,
    CharLit,
    LCharLit,
    UCharLit,
    StringLit,
    LStringLit,
    UStringLit,

    Eof,
    Error,
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Auto
                | TokenKind::Break
                | TokenKind::Case
                | TokenKind::Char
                | TokenKind::Const
                | TokenKind::Continue
                | TokenKind::Default
                | TokenKind::Do
                | TokenKind::Double
                | TokenKind::Else
                | TokenKind::Enum
                | TokenKind::Extern
                | TokenKind::Float
                | TokenKind::For
                | TokenKind::Goto
                | TokenKind::If
                | TokenKind::Inline
                | TokenKind::Int
                | TokenKind::Long
                | TokenKind::Register
                | TokenKind::Restrict
                | TokenKind::Return
                | TokenKind::Short
                | TokenKind::Signed
                | TokenKind::Sizeof
                | TokenKind::Static
                | TokenKind::Struct
                | TokenKind::Switch
                | TokenKind::Typedef
                | TokenKind::Union
                | TokenKind::Unsigned
                | TokenKind::Void
                | TokenKind::Volatile
                | TokenKind::While
                | TokenKind::Bool
                | TokenKind::Complex
                | TokenKind::Imaginary
                | TokenKind::Alignas
                | TokenKind::Alignof
                | TokenKind::Atomic
                | TokenKind::Generic
                | TokenKind::Noreturn
                | TokenKind::StaticAssert
                | TokenKind::ThreadLocal
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
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
