use crate::arena::Arena;
use crate::diagnostics::{DiagnosticCode, ParseError};
use crate::js::ast::*;
use crate::js::config::ParserOptions;
use crate::js::lexer::{Comment, Token, TokenKind};
use crate::span::Span;

use super::declarations;
use super::expressions;
use super::modules;
use super::patterns;
use super::statements;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    TopLevel,
    InFunction,
    InLoop,
    InSwitch,
    InClass,
    InConstructor,
    InModule,
    InAsync,
    InGenerator,
    InArrow,
}

impl Context {
    pub fn has_await(&self) -> bool {
        matches!(self, Context::InAsync | Context::InArrow)
    }

    pub fn has_yield(&self) -> bool {
        matches!(self, Context::InGenerator)
    }

    pub fn has_return(&self) -> bool {
        matches!(
            self,
            Context::InFunction
                | Context::InArrow
                | Context::InConstructor
                | Context::InAsync
                | Context::InGenerator
        )
    }
}

pub struct Parser<'a> {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub ast: Arena<Expr>,
    pub errors: Vec<ParseError>,
    pub comments: Vec<Comment>,
    pub ctx: Vec<Context>,
    pub options: &'a ParserOptions,
    pub in_async: bool,
    pub in_generator: bool,
    pub depth: u32,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, options: &'a ParserOptions) -> Self {
        let mut ctx = Vec::new();
        ctx.push(if options.is_module() {
            Context::InModule
        } else {
            Context::TopLevel
        });
        Parser {
            tokens,
            pos: 0,
            ast: Arena::new(),
            errors: Vec::new(),
            comments: Vec::new(),
            ctx,
            options,
            in_async: false,
            in_generator: false,
            depth: 0,
        }
    }

    pub fn try_new(
        tokens: Vec<Token>,
        options: &'a ParserOptions,
    ) -> Result<Self, Vec<ParseError>> {
        Ok(Self::new(tokens, options))
    }

    pub fn peek(&self) -> TokenKind {
        if self.pos >= self.tokens.len() {
            return TokenKind::Eof;
        }
        self.tokens[self.pos].kind
    }

    pub fn peek_ahead(&self, n: usize) -> TokenKind {
        let idx = self.pos + n;
        if idx >= self.tokens.len() {
            return TokenKind::Eof;
        }
        self.tokens[idx].kind
    }

    pub fn advance(&mut self) -> Token {
        let tok = if self.pos < self.tokens.len() {
            self.tokens[self.pos].clone()
        } else {
            return Token::new(TokenKind::Eof, crate::span::Span::ZERO);
        };
        if !tok.leading_comments.is_empty() && self.options.capture_comments {
            self.comments.extend(tok.leading_comments.iter().cloned());
        }
        self.pos += 1;
        tok
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        if self.peek() == kind {
            Ok(self.advance())
        } else {
            let tok = self.current_token().clone();
            let err = self.error(DiagnosticCode::UnexpectedToken, &tok);
            self.advance();
            Err(err)
        }
    }

    pub fn expect_ident(&mut self) -> String {
        if self.peek() == TokenKind::Ident {
            let token = self.advance();
            token.value
        } else {
            let tok = self.current_token().clone();
            let err = self.error(DiagnosticCode::UnexpectedToken, &tok);
            self.errors.push(err);
            self.advance();
            String::new()
        }
    }

    pub fn expect_advance(&mut self) -> Token {
        self.advance()
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenKind::Eof
    }

    pub fn current_pos(&self) -> usize {
        self.pos
    }

    pub fn current_token(&self) -> &Token {
        if self.pos >= self.tokens.len() {
            static EOF: Token = Token {
                kind: TokenKind::Eof,
                span: Span::ZERO,
                value: String::new(),
                leading_comments: Vec::new(),
                has_line_break: false,
            };
            return &EOF;
        }
        &self.tokens[self.pos]
    }

    pub fn previous_token(&self) -> Option<&Token> {
        if self.pos == 0 {
            return None;
        }
        Some(&self.tokens[self.pos - 1])
    }

    pub fn token_at(&self, i: usize) -> &Token {
        &self.tokens[i]
    }

    pub fn span_since(&self, start: usize) -> Span {
        let end_pos = if self.pos > 0 {
            self.tokens[self.pos - 1].span.end
        } else {
            self.tokens[start].span.start
        };
        Span::new(self.tokens[start].span.start, end_pos)
    }

    pub fn span_between(&self, start: usize, end: usize) -> Span {
        let s = self.tokens[start].span.start;
        let e = if end > 0 && end < self.tokens.len() {
            self.tokens[end - 1].span.end
        } else if !self.tokens.is_empty() {
            self.tokens[self.tokens.len() - 1].span.end
        } else {
            s
        };
        Span::new(s, e)
    }

    pub fn error(&mut self, code: DiagnosticCode, t: &Token) -> ParseError {
        ParseError::new(code, t.span)
    }

    pub fn error_at(&mut self, code: DiagnosticCode, span: Span) -> ParseError {
        ParseError::new(code, span)
    }

    pub fn error_msg(
        &mut self,
        code: DiagnosticCode,
        span: Span,
        msg: impl Into<String>,
    ) -> ParseError {
        ParseError::with_message(code, span, msg)
    }

    pub fn expr_span(&self, id: ExprRef) -> Span {
        use super::expressions::HasSpan;
        self.ast[id].span()
    }

    // ---- Recursion guard ----

    pub fn bump_recursion(&mut self) -> Result<(), ParseError> {
        self.depth += 1;
        if self.depth > crate::limits::MAX_RECURSION {
            let err = ParseError::new(
                DiagnosticCode::MaxRecursionExceeded,
                self.current_token().span,
            );
            if !self
                .errors
                .iter()
                .any(|existing| existing.code == DiagnosticCode::MaxRecursionExceeded)
            {
                self.errors.push(err.clone());
            }
            self.depth -= 1;
            return Err(err);
        }
        Ok(())
    }

    pub fn pop_recursion(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    // ---- Context helpers ----

    pub fn push_ctx(&mut self, ctx: Context) {
        self.ctx.push(ctx);
    }

    pub fn pop_ctx(&mut self) {
        self.ctx.pop();
    }

    pub fn in_async_ctx(&self) -> bool {
        for context in self.ctx.iter().rev() {
            match context {
                Context::InAsync | Context::InArrow | Context::InModule => return true,
                Context::InFunction | Context::InConstructor => return false,
                _ => {}
            }
        }
        false
    }

    pub fn in_generator_ctx(&self) -> bool {
        self.ctx.iter().any(|c| c.has_yield())
    }

    pub fn in_function_ctx(&self) -> bool {
        self.ctx.iter().any(|c| c.has_return())
    }

    pub fn in_loop_ctx(&self) -> bool {
        self.ctx.iter().any(|c| matches!(c, Context::InLoop))
    }

    pub fn in_switch_ctx(&self) -> bool {
        self.ctx.iter().any(|c| matches!(c, Context::InSwitch))
    }

    // ---- Entry points ----

    pub fn parse_program(&mut self) -> Program {
        if self.options.is_module() {
            Program::Module(self.parse_module())
        } else {
            let start = self.current_pos();
            let body = self.parse_script();
            Program::Script(Script {
                span: self.span_since(start),
                body,
            })
        }
    }

    pub fn parse_module(&mut self) -> Module {
        let start = self.current_pos();
        let mut body = Vec::new();
        while !self.is_eof() {
            if self.peek() == TokenKind::Hashbang {
                self.advance();
                continue;
            }
            match self.peek() {
                TokenKind::Import => {
                    body.push(ModuleItem::Import(self.parse_import()));
                }
                TokenKind::Export => {
                    body.push(ModuleItem::Export(self.parse_export()));
                }
                _ => {
                    if let Some(stmt) = self.parse_stmt() {
                        body.push(ModuleItem::Stmt(stmt));
                    } else {
                        break;
                    }
                }
            }
        }
        Module {
            span: self.span_since(start),
            body,
        }
    }

    pub fn parse_script(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_eof() {
            if self.peek() == TokenKind::Hashbang {
                self.advance();
                continue;
            }
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                break;
            }
        }
        stmts
    }

    // ---- Expression forwarding ----

    pub fn alloc_expr(&mut self, expr: Expr) -> ExprRef {
        self.ast.alloc(expr)
    }

    pub fn parse_expr(&mut self) -> ExprRef {
        if self.bump_recursion().is_err() {
            return self.ast.alloc(Expr::Ident(Ident {
                name: String::new(),
                span: Span::ZERO,
                optional: false,
            }));
        }
        let result = expressions::parse_expr(self, 0);
        self.pop_recursion();
        result
    }

    pub fn parse_assignment_expr(&mut self) -> ExprRef {
        if self.bump_recursion().is_err() {
            return self.ast.alloc(Expr::Ident(Ident {
                name: String::new(),
                span: Span::ZERO,
                optional: false,
            }));
        }
        let result = expressions::parse_assign_expr(self);
        self.pop_recursion();
        result
    }

    pub fn parse_cond_expr(&mut self) -> ExprRef {
        if self.bump_recursion().is_err() {
            return self.ast.alloc(Expr::Ident(Ident {
                name: String::new(),
                span: Span::ZERO,
                optional: false,
            }));
        }
        let result = expressions::parse_cond_expr(self);
        self.pop_recursion();
        result
    }

    // ---- Statement forwarding ----

    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        if self.bump_recursion().is_err() {
            return Some(Stmt::Empty(EmptyStmt { span: Span::ZERO }));
        }
        let result = statements::parse_stmt(self);
        self.pop_recursion();
        result
    }

    pub fn parse_block(&mut self) -> BlockStmt {
        if self.bump_recursion().is_err() {
            return BlockStmt {
                span: Span::ZERO,
                stmts: Vec::new(),
            };
        }
        let result = statements::parse_block(self);
        self.pop_recursion();
        result
    }

    // ---- Declaration forwarding ----

    pub fn parse_decl(&mut self) -> Option<Decl> {
        declarations::parse_decl(self)
    }

    pub fn parse_var_stmt(&mut self) -> Stmt {
        declarations::parse_var_stmt(self)
    }

    pub fn parse_fn_decl(&mut self) -> FnDecl {
        declarations::parse_fn_decl(self)
    }

    pub fn parse_class_decl(&mut self) -> ClassDecl {
        declarations::parse_class_decl(self)
    }

    pub fn parse_fn_expr(&mut self) -> FnExpr {
        declarations::parse_fn_expr(self)
    }

    pub fn parse_class_expr(&mut self) -> ClassExpr {
        declarations::parse_class_expr(self)
    }

    // ---- Pattern forwarding ----

    pub fn parse_pat(&mut self) -> Pat {
        patterns::parse_pat(self)
    }

    pub fn parse_binding_pat(&mut self) -> Pat {
        patterns::parse_binding_pat(self)
    }

    pub fn parse_assignment_pat(&mut self) -> Pat {
        patterns::parse_assignment_pat(self)
    }

    // ---- Module forwarding ----

    pub fn parse_import(&mut self) -> ImportDecl {
        modules::parse_import(self)
    }

    pub fn parse_export(&mut self) -> ExportDecl {
        modules::parse_export(self)
    }
}
