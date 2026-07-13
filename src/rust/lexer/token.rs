use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ident,
    Fn,
    Let,
    Mut,
    Const,
    Static,
    Impl,
    Trait,
    Pub,
    Crate,
    Self_,
    Super,
    Use,
    Mod,
    Struct,
    Enum,
    Union,
    Type_,
    Where,
    For,
    In,
    If,
    Else,
    Match,
    While,
    Loop,
    Break,
    Continue,
    Return,
    Async,
    Await,
    Unsafe,
    Extern,
    Ref,
    Move,
    Dyn,
    As,
    Macro,
    True,
    False,
    None_,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
    Bool,
    Char,
    Str,
    Box_,
    Vec_,
    Option_,
    Result_,
    String_,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    LtLt,
    GtGt,
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
    EqEq,
    Ne,
    Lt,
    Gt,
    LtEq,
    GtEq,
    AmpersandAmpersand,
    PipePipe,
    Dot,
    DotDot,
    DotDotDot,
    DotDotEq,
    Comma,
    Semicolon,
    Colon,
    ColonColon,
    FatArrow,
    Arrow,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eq,
    Exclamation,
    Question,
    QuestionDot,
    PlusPlus,
    MinusMinus,
    Hash,
    At,
    Dollar,
    Underscore,

    IntLit,
    FloatLit,
    StringLit,
    RawStringLit,
    ByteLit,
    ByteStringLit,
    CharLit,
    Lifetime,
    DocComment,

    Eof,
    Error,
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Fn
                | TokenKind::Let
                | TokenKind::Mut
                | TokenKind::Const
                | TokenKind::Static
                | TokenKind::Impl
                | TokenKind::Trait
                | TokenKind::Pub
                | TokenKind::Crate
                | TokenKind::Self_
                | TokenKind::Super
                | TokenKind::Use
                | TokenKind::Mod
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Union
                | TokenKind::Type_
                | TokenKind::Where
                | TokenKind::For
                | TokenKind::In
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::Match
                | TokenKind::While
                | TokenKind::Loop
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Return
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Unsafe
                | TokenKind::Extern
                | TokenKind::Ref
                | TokenKind::Move
                | TokenKind::Dyn
                | TokenKind::As
                | TokenKind::True
                | TokenKind::False
                | TokenKind::None_
                | TokenKind::Bool
                | TokenKind::Char
                | TokenKind::Str
                | TokenKind::I8
                | TokenKind::I16
                | TokenKind::I32
                | TokenKind::I64
                | TokenKind::I128
                | TokenKind::Isize
                | TokenKind::U8
                | TokenKind::U16
                | TokenKind::U32
                | TokenKind::U64
                | TokenKind::U128
                | TokenKind::Usize
                | TokenKind::F32
                | TokenKind::F64
                | TokenKind::Box_
                | TokenKind::Vec_
                | TokenKind::Option_
                | TokenKind::Result_
                | TokenKind::String_
                | TokenKind::Macro
                | TokenKind::Underscore
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
    Doc,
}
