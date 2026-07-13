use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ident,
    PrivateName,

    // Keywords
    Abstract,
    Accessor,
    Any,
    As,
    Assert,
    Async,
    Await,
    Boolean,
    Break,
    Case,
    Catch,
    Class,
    Const,
    Constructor,
    Continue,
    Debugger,
    Declare,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    False,
    Finally,
    For,
    From,
    Function,
    Get,
    If,
    Implements,
    Import,
    In,
    Infer,
    Instanceof,
    Interface,
    KeyOf,
    Let,
    Module,
    Namespace,
    Never,
    New,
    Null,
    Of,
    Package,
    Private,
    Protected,
    Public,
    Readonly,
    Return,
    Satisfies,
    Set,
    Static,
    StringLocal,
    Super,
    Switch,
    Symbol,
    This,
    Throw,
    True,
    Try,
    Type,
    Typeof,
    Undefined,
    Unique,
    Unknown,
    Using,
    Var,
    Void,
    While,
    With,
    Yield,

    // Punctuators
    Backtick,
    Colon,
    Comma,
    Dot,
    DotDotDot,
    FatArrow,
    LBrace,
    LBracket,
    LParen,
    Question,
    QuestionDot,
    RBrace,
    RBracket,
    RParen,
    Semicolon,

    // Operators
    Ampersand,
    AmpersandAmpersand,
    AmpersandAmpersandEq,
    AmpersandEq,
    Arrow,
    Caret,
    CaretEq,
    Eq,
    EqEq,
    EqEqEq,
    Ne,
    Neq,
    Exclamation,
    Gt,
    GtEq,
    GtGt,
    GtGtEq,
    GtGtGt,
    GtGtGtEq,
    Lt,
    LtEq,
    LtLt,
    LtLtEq,
    Minus,
    MinusEq,
    MinusMinus,
    Percent,
    PercentEq,
    Pipe,
    PipeEq,
    PipePipe,
    PipePipeEq,
    Plus,
    PlusEq,
    PlusPlus,
    QuestionQuestion,
    QuestionQuestionEq,
    Slash,
    SlashEq,
    Star,
    StarEq,
    StarStar,
    StarStarEq,
    Tilde,

    // Literals
    BigInt,
    Number,
    Regex,
    String,
    Template,
    TemplateHead,
    TemplateMiddle,
    TemplateTail,

    // Special
    Eof,
    Error,
    Hashbang,

    // Stage-3 decorators
    At,

    // Pipeline operator
    PipeGt,

    // Record & Tuple literals
    HashLBrace,
    HashLBracket,
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Abstract
                | TokenKind::Accessor
                | TokenKind::Any
                | TokenKind::As
                | TokenKind::Assert
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Boolean
                | TokenKind::Break
                | TokenKind::Case
                | TokenKind::Catch
                | TokenKind::Class
                | TokenKind::Const
                | TokenKind::Constructor
                | TokenKind::Continue
                | TokenKind::Debugger
                | TokenKind::Declare
                | TokenKind::Default
                | TokenKind::Delete
                | TokenKind::Do
                | TokenKind::Else
                | TokenKind::Enum
                | TokenKind::Export
                | TokenKind::Extends
                | TokenKind::False
                | TokenKind::Finally
                | TokenKind::For
                | TokenKind::From
                | TokenKind::Function
                | TokenKind::Get
                | TokenKind::If
                | TokenKind::Implements
                | TokenKind::Import
                | TokenKind::In
                | TokenKind::Infer
                | TokenKind::Instanceof
                | TokenKind::Interface
                | TokenKind::KeyOf
                | TokenKind::Let
                | TokenKind::Module
                | TokenKind::Namespace
                | TokenKind::Never
                | TokenKind::New
                | TokenKind::Null
                | TokenKind::Of
                | TokenKind::Package
                | TokenKind::Private
                | TokenKind::Protected
                | TokenKind::Public
                | TokenKind::Readonly
                | TokenKind::Return
                | TokenKind::Satisfies
                | TokenKind::Set
                | TokenKind::Static
                | TokenKind::StringLocal
                | TokenKind::Super
                | TokenKind::Switch
                | TokenKind::Symbol
                | TokenKind::This
                | TokenKind::Throw
                | TokenKind::True
                | TokenKind::Try
                | TokenKind::Type
                | TokenKind::Typeof
                | TokenKind::Undefined
                | TokenKind::Unique
                | TokenKind::Unknown
                | TokenKind::Using
                | TokenKind::Var
                | TokenKind::Void
                | TokenKind::While
                | TokenKind::With
                | TokenKind::Yield
        )
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::Number
                | TokenKind::String
                | TokenKind::Template
                | TokenKind::Regex
                | TokenKind::BigInt
                | TokenKind::TemplateHead
                | TokenKind::TemplateMiddle
                | TokenKind::TemplateTail
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
    pub span: Span,
    pub text: String,
    pub multi_line: bool,
}

impl Comment {
    pub fn new(span: Span, text: impl Into<String>, multi_line: bool) -> Self {
        Comment {
            span,
            text: text.into(),
            multi_line,
        }
    }
}
