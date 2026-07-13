use super::TokenKind;
use std::collections::HashMap;

pub fn keyword_map() -> HashMap<&'static str, TokenKind> {
    let mut m = HashMap::new();
    m.insert("False", TokenKind::False);
    m.insert("None", TokenKind::None_);
    m.insert("True", TokenKind::True);
    m.insert("and", TokenKind::And);
    m.insert("as", TokenKind::As);
    m.insert("assert", TokenKind::Assert);
    m.insert("async", TokenKind::Async);
    m.insert("await", TokenKind::Await);
    m.insert("break", TokenKind::Break);
    m.insert("class", TokenKind::Class);
    m.insert("continue", TokenKind::Continue);
    m.insert("def", TokenKind::Def);
    m.insert("del", TokenKind::Del);
    m.insert("elif", TokenKind::Elif);
    m.insert("else", TokenKind::Else);
    m.insert("except", TokenKind::Except);
    m.insert("finally", TokenKind::Finally);
    m.insert("for", TokenKind::For);
    m.insert("from", TokenKind::From);
    m.insert("global", TokenKind::Global);
    m.insert("if", TokenKind::If);
    m.insert("import", TokenKind::Import);
    m.insert("in", TokenKind::In);
    m.insert("is", TokenKind::Is);
    m.insert("lambda", TokenKind::Lambda);
    m.insert("match", TokenKind::Match);
    m.insert("nonlocal", TokenKind::Nonlocal);
    m.insert("not", TokenKind::Not);
    m.insert("or", TokenKind::Or);
    m.insert("pass", TokenKind::Pass);
    m.insert("raise", TokenKind::Raise);
    m.insert("return", TokenKind::Return);
    m.insert("try", TokenKind::Try);
    m.insert("while", TokenKind::While);
    m.insert("with", TokenKind::With);
    m.insert("yield", TokenKind::Yield);
    m.insert("type", TokenKind::Type);
    m.insert("self", TokenKind::Self_);
    m
}
