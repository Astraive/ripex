use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ident,
    Underscore,
    // Keywords
    False,
    None_,
    True,
    And,
    As,
    Assert,
    Async,
    Await,
    Break,
    Case,
    Class,
    Continue,
    Def,
    Del,
    Elif,
    Else,
    Except,
    Finally,
    For,
    From,
    Global,
    If,
    Import,
    In,
    Is,
    Lambda,
    Match,
    Nonlocal,
    Not,
    Or,
    Pass,
    Raise,
    Return,
    Try,
    While,
    With,
    Yield,
    // Type hint keywords (soft)
    Type,
    Self_,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    SlashSlash,
    Percent,
    StarStar,
    At,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    SlashSlashEq,
    PercentEq,
    StarStarEq,
    AtEq,
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
    Eq,
    AmpersandAmpersand,
    PipePipe,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,
    DotDotDot,
    Ellipsis,
    Arrow,
    Walrus,
    SemicolonSynthetic,

    // Literals
    IntLit,
    FloatLit,
    ComplexLit,
    StringLit,
    BytesLit,
    FStringLit,
    FStringHead,
    FStringMid,
    FStringTail,

    // Special
    Indent,
    Dedent,
    Newline,
    Eof,
    Error,
    SoftKeyword,
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::False
                | TokenKind::None_
                | TokenKind::True
                | TokenKind::And
                | TokenKind::As
                | TokenKind::Assert
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Break
                | TokenKind::Class
                | TokenKind::Continue
                | TokenKind::Def
                | TokenKind::Del
                | TokenKind::Elif
                | TokenKind::Else
                | TokenKind::Except
                | TokenKind::Finally
                | TokenKind::For
                | TokenKind::From
                | TokenKind::Global
                | TokenKind::If
                | TokenKind::Import
                | TokenKind::In
                | TokenKind::Is
                | TokenKind::Lambda
                | TokenKind::Match
                | TokenKind::Nonlocal
                | TokenKind::Not
                | TokenKind::Or
                | TokenKind::Pass
                | TokenKind::Raise
                | TokenKind::Return
                | TokenKind::Try
                | TokenKind::While
                | TokenKind::With
                | TokenKind::Yield
                | TokenKind::Type
                | TokenKind::Self_
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
    Docstring,
}
