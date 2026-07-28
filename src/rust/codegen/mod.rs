use super::ast::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationError {
    UnsupportedNode(&'static str),
}

pub struct Codegen {
    output: String,
    indent: usize,
    error: Option<GenerationError>,
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            error: None,
        }
    }

    pub fn generate(
        &mut self,
        program: &Program,
    ) -> Result<String, GenerationError> {
        self.output.clear();
        self.indent = 0;
        self.error = None;
        for item in &program.items {
            self.item(item);
        }
        match self.error.take() {
            Some(error) => Err(error),
            None => Ok(self.output.clone()),
        }
    }

    fn unsupported(&mut self, node: &'static str) {
        if self.error.is_none() {
            self.error = Some(GenerationError::UnsupportedNode(node));
        }
    }

    fn ind(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent));
    }
    fn vis(&mut self, vis: &Visibility) {
        match vis {
            Visibility::Pub => self.output.push_str("pub "),
            Visibility::PubCrate => self.output.push_str("pub(crate) "),
            Visibility::PubSuper => self.output.push_str("pub(super) "),
            Visibility::PubIn(path) => self.output.push_str(&format!("pub(in {path}) ")),
            Visibility::Private => {}
        }
    }
    fn generics(&mut self, values: &[GenericParam]) {
        if values.is_empty() {
            return;
        }
        self.output.push('<');
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&value.name);
            if !value.bounds.is_empty() {
                self.output.push_str(": ");
                for (j, bound) in value.bounds.iter().enumerate() {
                    if j > 0 {
                        self.output.push_str(" + ");
                    }
                    self.expr(bound);
                }
            }
        }
        self.output.push('>');
    }
    fn item(&mut self, item: &Item) {
        self.ind();
        match item {
            Item::Fn(value, _) => {
                self.function(value);
                return;
            }
            Item::Struct(value, _) => {
                self.vis(&value.visibility);
                self.output.push_str("struct ");
                self.output.push_str(&value.name);
                self.generics(&value.generics);
                self.output.push_str(" {\n");
                self.indent += 1;
                for field in &value.fields {
                    self.ind();
                    self.vis(&field.visibility);
                    self.output.push_str(&field.name);
                    self.output.push_str(": ");
                    self.expr(&field.type_ann);
                    self.output.push_str(",\n");
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("}\n");
                return;
            }
            Item::Enum(value, _) => {
                self.vis(&value.visibility);
                self.output.push_str("enum ");
                self.output.push_str(&value.name);
                self.generics(&value.generics);
                self.output.push_str(" {\n");
                self.indent += 1;
                for variant in &value.variants {
                    self.ind();
                    self.output.push_str(&variant.name);
                    if !variant.fields.is_empty() {
                        self.output.push('(');
                        for (i, ty) in variant.fields.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            self.expr(ty);
                        }
                        self.output.push(')');
                    }
                    self.output.push_str(",\n");
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("}\n");
                return;
            }
            Item::Trait(value, _) => {
                self.vis(&value.visibility);
                self.output.push_str("trait ");
                self.output.push_str(&value.name);
                self.output.push_str(" {\n");
                self.indent += 1;
                for method in &value.methods {
                    self.ind();
                    self.function(method);
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("}\n");
                return;
            }
            Item::Impl(value, _) => {
                self.output.push_str("impl ");
                if let Some(name) = &value.trait_name {
                    self.output.push_str(name);
                    self.output.push_str(" for ");
                }
                self.expr(&value.type_name);
                self.output.push_str(" {\n");
                self.indent += 1;
                for method in &value.methods {
                    self.ind();
                    self.function(method);
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("}\n");
                return;
            }
            Item::Use(value, _) => {
                self.output.push_str("use ");
                self.use_path(&value.path);
                self.output.push(';');
            }
            Item::Mod(value, _) => {
                self.vis(&value.visibility);
                self.output.push_str("mod ");
                self.output.push_str(&value.name);
                if value.items.is_empty() {
                    self.output.push(';');
                } else {
                    self.output.push_str(" {\n");
                    self.indent += 1;
                    for item in &value.items {
                        self.item(item);
                    }
                    self.indent -= 1;
                    self.ind();
                    self.output.push('}');
                }
            }
            Item::Type(value, _) => {
                self.vis(&value.visibility);
                self.output.push_str("type ");
                self.output.push_str(&value.name);
                self.generics(&value.generics);
                self.output.push_str(" = ");
                self.expr(&value.type_);
                self.output.push(';');
            }
            Item::Static(value, _) => {
                self.vis(&value.visibility);
                self.output.push_str("static ");
                if value.mutable {
                    self.output.push_str("mut ");
                }
                self.output.push_str(&value.name);
                self.output.push_str(": ");
                self.expr(&value.type_);
                self.output.push_str(" = ");
                self.expr(&value.init);
                self.output.push(';');
            }
            Item::Const(value, _) => {
                self.vis(&value.visibility);
                self.output.push_str("const ");
                self.output.push_str(&value.name);
                if let Some(ty) = &value.type_ {
                    self.output.push_str(": ");
                    self.expr(ty);
                }
                self.output.push_str(" = ");
                self.expr(&value.init);
                self.output.push(';');
            }
            Item::Macro(value, _) => {
                self.output.push_str(&value.name);
                self.output.push_str("!{");
                self.output.push_str(&value.body);
                self.output.push('}');
            }
            Item::ExternCrate(name, _) => {
                self.output.push_str("extern crate ");
                self.output.push_str(name);
                self.output.push(';');
            }
        }
        self.output.push('\n');
    }
    fn function(&mut self, value: &FnDecl) {
        self.vis(&value.visibility);
        if value.is_async {
            self.output.push_str("async ");
        }
        if value.is_unsafe {
            self.output.push_str("unsafe ");
        }
        if value.is_extern {
            self.output.push_str("extern \"C\" ");
        }
        self.output.push_str("fn ");
        self.output.push_str(&value.name);
        self.generics(&value.generics);
        self.output.push('(');
        for (i, param) in value.params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.pattern(&param.pattern);
            if let Some(ty) = &param.type_ann {
                self.output.push_str(": ");
                self.expr(ty);
            }
        }
        self.output.push(')');
        if let Some(ty) = &value.return_type {
            self.output.push_str(" -> ");
            self.expr(ty);
        }
        if let Some(body) = &value.body {
            self.output.push(' ');
            self.block(body);
        } else {
            self.output.push_str(";\n");
        }
    }
    fn block(&mut self, value: &Block) {
        self.output.push_str("{\n");
        self.indent += 1;
        for stmt in &value.stmts {
            self.stmt(stmt);
        }
        self.indent -= 1;
        self.ind();
        self.output.push_str("}\n");
    }
    fn stmt(&mut self, stmt: &Stmt) {
        self.ind();
        match stmt {
            Stmt::Expr(value, _) => {
                self.expr(value);
                self.output.push(';');
            }
            Stmt::Let(value, _) => {
                self.output.push_str("let ");
                if value.mutable {
                    self.output.push_str("mut ");
                }
                self.pattern(&value.pattern);
                if let Some(ty) = &value.type_ann {
                    self.output.push_str(": ");
                    self.expr(ty);
                }
                if let Some(init) = &value.init {
                    self.output.push_str(" = ");
                    self.expr(init);
                }
                self.output.push(';');
            }
            Stmt::Item(item, _) => {
                self.output
                    .truncate(self.output.trim_end_matches(' ').len());
                self.item(item);
                return;
            }
            Stmt::Empty(_) => self.output.push(';'),
        }
        self.output.push('\n');
    }
    fn use_path(&mut self, path: &UsePath) {
        match path {
            UsePath::Simple(value, _) => self.output.push_str(value),
            UsePath::Glob(value, _) => {
                self.output.push_str(value);
                self.output.push_str("::*");
            }
            UsePath::Self_(value, _) => {
                self.output.push_str(value);
                self.output.push_str("::self");
            }
            UsePath::Nested(value, children, _) => {
                self.output.push_str(value);
                self.output.push_str("::{");
                for (i, child) in children.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.use_path(child);
                }
                self.output.push('}');
            }
        }
    }
    fn exprs(&mut self, values: &[Expr]) {
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.expr(value);
        }
    }
    fn expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Bool(v, _) => self.output.push_str(if *v { "true" } else { "false" }),
            Expr::Int(v, _) => self.output.push_str(&v.to_string()),
            Expr::Float(v, _) => self.output.push_str(&v.to_string()),
            Expr::String(v, _) => self.output.push_str(&format!("{v:?}")),
            Expr::Char(v, _) => self.output.push_str(&format!("{v:?}")),
            Expr::Ident(v, _) => self.output.push_str(v),
            Expr::Path(v, _) => self.output.push_str(&v.join("::")),
            Expr::Binary(l, o, r, _) => {
                self.output.push('(');
                self.expr(l);
                self.output.push(' ');
                self.output.push_str(bin(*o));
                self.output.push(' ');
                self.expr(r);
                self.output.push(')');
            }
            Expr::Unary(o, v, _) => {
                self.output.push_str(un(*o));
                self.expr(v);
            }
            Expr::Call(f, a, _) => {
                self.expr(f);
                self.output.push('(');
                self.exprs(a);
                self.output.push(')');
            }
            Expr::MethodCall(v, n, a, _) => {
                self.expr(v);
                self.output.push('.');
                self.output.push_str(n);
                self.output.push('(');
                self.exprs(a);
                self.output.push(')');
            }
            Expr::Index(v, i, _) => {
                self.expr(v);
                self.output.push('[');
                self.expr(i);
                self.output.push(']');
            }
            Expr::Field(v, n, _) => {
                self.expr(v);
                self.output.push('.');
                self.output.push_str(n);
            }
            Expr::Tuple(v, _) => {
                self.output.push('(');
                self.exprs(v);
                if v.len() == 1 {
                    self.output.push(',');
                }
                self.output.push(')');
            }
            Expr::Array(v, _) => {
                self.output.push('[');
                self.exprs(v);
                self.output.push(']');
            }
            Expr::Struct(n, f, rest, _) => {
                self.output.push_str(n);
                self.output.push_str(" { ");
                for (i, x) in f.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&x.name);
                    if let Some(v) = &x.value {
                        self.output.push_str(": ");
                        self.expr(v);
                    }
                }
                if let Some(rest) = rest {
                    if !f.is_empty() {
                        self.output.push_str(", ");
                    }
                    self.output.push_str("..");
                    self.expr(rest);
                }
                self.output.push_str(" }");
            }
            Expr::Closure(p, b, _) => {
                self.output.push('|');
                for (i, p) in p.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.pattern(p);
                }
                self.output.push_str("| ");
                self.expr(b);
            }
            Expr::Block(v, _) => self.block_inline(v),
            Expr::If(t, b, e, _) => {
                self.output.push_str("if ");
                self.expr(t);
                self.output.push(' ');
                self.block_inline(b);
                if let Some(e) = e {
                    self.output.push_str(" else ");
                    self.expr(e);
                }
            }
            Expr::Match(v, a, _) => {
                self.output.push_str("match ");
                self.expr(v);
                self.output.push_str(" { ");
                for arm in a {
                    for (i, p) in arm.patterns.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(" | ");
                        }
                        self.pattern(p);
                    }
                    if let Some(g) = &arm.guard {
                        self.output.push_str(" if ");
                        self.expr(g);
                    }
                    self.output.push_str(" => ");
                    self.expr(&arm.body);
                    self.output.push_str(", ");
                }
                self.output.push('}');
            }
            Expr::While(t, b, _) => {
                self.output.push_str("while ");
                self.expr(t);
                self.output.push(' ');
                self.block_inline(b);
            }
            Expr::Loop(b, _) => {
                self.output.push_str("loop ");
                self.block_inline(b);
            }
            Expr::For(p, v, b, _) => {
                self.output.push_str("for ");
                self.pattern(p);
                self.output.push_str(" in ");
                self.expr(v);
                self.output.push(' ');
                self.block_inline(b);
            }
            Expr::Return(v, _) => {
                self.output.push_str("return");
                if let Some(v) = v {
                    self.output.push(' ');
                    self.expr(v);
                }
            }
            Expr::Break(v, _) => {
                self.output.push_str("break");
                if let Some(v) = v {
                    self.output.push(' ');
                    self.expr(v);
                }
            }
            Expr::Continue(_) => self.output.push_str("continue"),
            Expr::Paren(v, _) => {
                self.output.push('(');
                self.expr(v);
                self.output.push(')');
            }
            Expr::Async(v, _) => {
                self.output.push_str("async ");
                self.expr(v);
            }
            Expr::Await(v, _) => {
                self.expr(v);
                self.output.push_str(".await");
            }
            Expr::Ref(v, m, _) => {
                self.output.push('&');
                if *m {
                    self.output.push_str("mut ");
                }
                self.expr(v);
            }
            Expr::Deref(v, _) => {
                self.output.push('*');
                self.expr(v);
            }
            Expr::Cast(v, t, _) => {
                self.expr(v);
                self.output.push_str(" as ");
                self.expr(t);
            }
            Expr::Error(_) => self.unsupported("error expression"),
        }
    }
    fn block_inline(&mut self, value: &Block) {
        self.output.push_str("{ ");
        for stmt in &value.stmts {
            match stmt {
                Stmt::Expr(v, _) => {
                    self.expr(v);
                    self.output.push_str("; ");
                }
                Stmt::Let(v, _) => {
                    self.output.push_str("let ");
                    self.pattern(&v.pattern);
                    if let Some(i) = &v.init {
                        self.output.push_str(" = ");
                        self.expr(i);
                    }
                    self.output.push_str("; ");
                }
            Stmt::Item(_, _) | Stmt::Empty(_) => self.unsupported("unsupported inline statement"),
            }
        }
        self.output.push('}');
    }
    fn pattern(&mut self, p: &Pattern) {
        match p {
            Pattern::Wildcard(_) => self.output.push('_'),
            Pattern::Ident(v, _) => self.output.push_str(v),
            Pattern::Lit(v, _) => self.expr(v),
            Pattern::Tuple(v, _) => {
                self.output.push('(');
                for (i, p) in v.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.pattern(p);
                }
                self.output.push(')');
            }
            Pattern::Struct(n, f, _) => {
                self.output.push_str(n);
                self.output.push_str(" { ");
                for (i, x) in f.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&x.name);
                    self.output.push_str(": ");
                    self.pattern(&x.pattern);
                }
                self.output.push_str(" }");
            }
            Pattern::Range(a, b, _) => {
                self.pattern(a);
                self.output.push_str("..=");
                self.pattern(b);
            }
            Pattern::Or(v, _) => {
                for (i, p) in v.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" | ");
                    }
                    self.pattern(p);
                }
            }
            Pattern::Ref(v, m, _) => {
                self.output.push_str("ref ");
                if *m {
                    self.output.push_str("mut ");
                }
                self.pattern(v);
            }
            Pattern::Slice(v, _) => {
                self.output.push('[');
                for (i, p) in v.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.pattern(p);
                }
                self.output.push(']');
            }
            Pattern::Rest(_) => self.output.push_str(".."),
        }
    }
}

fn bin(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Rem => "%",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::Le => "<=",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::Shl => "<<",
        BinaryOp::Shr => ">>",
        BinaryOp::Assign => "=",
        BinaryOp::AddAssign => "+=",
        BinaryOp::SubAssign => "-=",
        BinaryOp::MulAssign => "*=",
        BinaryOp::DivAssign => "/=",
        BinaryOp::RemAssign => "%=",
        BinaryOp::AndAssign => "&=",
        BinaryOp::OrAssign => "|=",
        BinaryOp::XorAssign => "^=",
        BinaryOp::ShlAssign => "<<=",
        BinaryOp::ShrAssign => ">>=",
        BinaryOp::Range => "..",
        BinaryOp::RangeInclusive => "..=",
        BinaryOp::Pipe => "|>",
    }
}
fn un(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
        UnaryOp::Deref => "*",
        UnaryOp::Ref => "&",
        UnaryOp::RefMut => "&mut ",
    }
}
