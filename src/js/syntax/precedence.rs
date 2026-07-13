use crate::js::lexer::TokenKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Precedence(pub u8);

impl Precedence {
    pub const MIN: Precedence = Precedence(0);
    pub const MAX: Precedence = Precedence(19);

    pub const SEQUENCE: Precedence = Precedence(0);
    pub const YIELD: Precedence = Precedence(1);
    pub const ASSIGN: Precedence = Precedence(2);
    pub const CONDITIONAL: Precedence = Precedence(3);
    pub const NULLISH: Precedence = Precedence(4);
    pub const LOGICAL_OR: Precedence = Precedence(5);
    pub const LOGICAL_AND: Precedence = Precedence(6);
    pub const BITWISE_OR: Precedence = Precedence(7);
    pub const BITWISE_XOR: Precedence = Precedence(8);
    pub const BITWISE_AND: Precedence = Precedence(9);
    pub const EQUALITY: Precedence = Precedence(10);
    pub const RELATIONAL: Precedence = Precedence(11);
    pub const SHIFT: Precedence = Precedence(12);
    pub const ADDITIVE: Precedence = Precedence(13);
    pub const MULTIPLICATIVE: Precedence = Precedence(14);
    pub const EXPONENTIATION: Precedence = Precedence(15);
    pub const PREFIX: Precedence = Precedence(16);
    pub const POSTFIX: Precedence = Precedence(17);
    pub const CALL: Precedence = Precedence(18);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fixity {
    Left(Precedence),
    Right(Precedence),
}

pub fn prefix_bp(kind: TokenKind) -> Option<Precedence> {
    match kind {
        TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Exclamation
        | TokenKind::Tilde
        | TokenKind::Typeof
        | TokenKind::Void
        | TokenKind::Delete => Some(Precedence::PREFIX),
        TokenKind::PlusPlus | TokenKind::MinusMinus => Some(Precedence::POSTFIX),
        TokenKind::Await => Some(Precedence::PREFIX),
        _ => None,
    }
}

pub fn infix_bp(kind: TokenKind) -> Option<Fixity> {
    match kind {
        TokenKind::PipePipe => Some(Fixity::Left(Precedence::LOGICAL_OR)),
        TokenKind::AmpersandAmpersand => Some(Fixity::Left(Precedence::LOGICAL_AND)),
        TokenKind::Pipe => Some(Fixity::Left(Precedence::BITWISE_OR)),
        TokenKind::Caret => Some(Fixity::Left(Precedence::BITWISE_XOR)),
        TokenKind::Ampersand => Some(Fixity::Left(Precedence::BITWISE_AND)),
        TokenKind::EqEq | TokenKind::Ne | TokenKind::EqEqEq | TokenKind::Neq => {
            Some(Fixity::Left(Precedence::EQUALITY))
        }
        TokenKind::Lt
        | TokenKind::Gt
        | TokenKind::LtEq
        | TokenKind::GtEq
        | TokenKind::In
        | TokenKind::Instanceof => Some(Fixity::Left(Precedence::RELATIONAL)),
        TokenKind::LtLt | TokenKind::GtGt | TokenKind::GtGtGt => {
            Some(Fixity::Left(Precedence::SHIFT))
        }
        TokenKind::Plus | TokenKind::Minus => Some(Fixity::Left(Precedence::ADDITIVE)),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => {
            Some(Fixity::Left(Precedence::MULTIPLICATIVE))
        }
        TokenKind::StarStar => Some(Fixity::Right(Precedence::EXPONENTIATION)),
        TokenKind::QuestionQuestion => Some(Fixity::Left(Precedence::NULLISH)),
        _ => None,
    }
}
