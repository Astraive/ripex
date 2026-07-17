use super::ast::*;

pub struct Codegen {
    output: String,
    indent: usize,
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
        }
    }
    pub fn generate(&mut self, p: &Program) -> String {
        self.output.clear();
        self.indent = 0;
        for d in &p.decls {
            self.decl(d);
        }
        self.output.clone()
    }
    fn ind(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent));
    }
    fn decl(&mut self, d: &Decl) {
        self.ind();
        match d {
            Decl::Func(f, _) => {
                if f.is_friend {
                    self.output.push_str("friend ");
                }
                if f.is_static {
                    self.output.push_str("static ");
                }
                if f.is_inline {
                    self.output.push_str("inline ");
                }
                if f.is_constexpr {
                    self.output.push_str("constexpr ");
                }
                self.ty(&f.return_type);
                self.output.push(' ');
                self.output.push_str(&f.name);
                self.output.push('(');
                for (i, p) in f.params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.ty(&p.type_);
                    if let Some(n) = &p.name {
                        self.output.push(' ');
                        self.output.push_str(n);
                    }
                    if let Some(v) = &p.default {
                        self.output.push_str(" = ");
                        self.expr(v);
                    }
                }
                self.output.push(')');
                if f.is_const {
                    self.output.push_str(" const");
                }
                if let Some(b) = &f.body {
                    self.output.push(' ');
                    self.block(b);
                } else {
                    self.output.push_str(";\n");
                }
            }
            Decl::Var(v, _) => {
                if v.is_extern {
                    self.output.push_str("extern ");
                }
                if v.is_static {
                    self.output.push_str("static ");
                }
                if v.is_constexpr {
                    self.output.push_str("constexpr ");
                } else if v.is_const {
                    self.output.push_str("const ");
                }
                self.ty(&v.type_);
                self.output.push(' ');
                self.output.push_str(&v.name);
                if let Some(x) = &v.init {
                    self.output.push_str(" = ");
                    self.expr(x);
                }
                self.output.push_str(";\n");
            }
            Decl::Namespace(n, v, _) => {
                self.output.push_str("namespace ");
                self.output.push_str(n);
                self.output.push_str(" {\n");
                self.indent += 1;
                for d in v {
                    self.decl(d);
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("}\n");
            }
            Decl::Using(n, _) => {
                if n == "template-instantiation" {
                    self.output
                        .push_str("using ripex_template_instantiation = int;\n");
                } else {
                    self.output.push_str(&format!("using {n};\n"));
                }
            }
            Decl::UsingNamespace(n, _) => self.output.push_str(&format!("using namespace {n};\n")),
            Decl::Template(v, _) => {
                self.output.push_str("template <");
                for (i, p) in v.params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    match p {
                        TemplateParam::Type(n, _) => self.output.push_str(&format!("typename {n}")),
                        TemplateParam::Value(t, n, d, _) => {
                            self.ty(t);
                            self.output.push(' ');
                            self.output.push_str(n);
                            if let Some(d) = d {
                                self.output.push_str(" = ");
                                self.expr(d);
                            }
                        }
                        TemplateParam::Template(n, _) => {
                            self.output.push_str(&format!("typename {n}"))
                        }
                    }
                }
                self.output.push_str(">\n");
                self.decl(&v.decl);
            }
            Decl::Class(v, _) => {
                self.output.push_str("class ");
                self.output.push_str(&v.name);
                if !v.bases.is_empty() {
                    self.output.push_str(" : ");
                    for (i, b) in v.bases.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.output.push_str(match b.access {
                            AccessSpec::Public => "public ",
                            AccessSpec::Private => "private ",
                            AccessSpec::Protected => "protected ",
                        });
                        if b.is_virtual {
                            self.output.push_str("virtual ");
                        }
                        self.output.push_str(&b.name);
                    }
                }
                self.output.push_str(" {\n");
                self.indent += 1;
                for m in &v.members {
                    match m {
                        ClassMember::Decl(d, _) => self.decl(d),
                        ClassMember::Access(a, _) => {
                            self.indent -= 1;
                            self.ind();
                            self.output.push_str(match a {
                                AccessSpec::Public => "public:\n",
                                AccessSpec::Private => "private:\n",
                                AccessSpec::Protected => "protected:\n",
                            });
                            self.indent += 1;
                        }
                    }
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("};\n");
            }
            Decl::Struct(v, _) => {
                self.output.push_str("struct ");
                self.output.push_str(&v.name);
                self.output.push_str(" {\n");
                self.indent += 1;
                for m in &v.members {
                    self.ind();
                    self.ty(&m.type_);
                    self.output.push(' ');
                    self.output.push_str(&m.name);
                    self.output.push_str(";\n");
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("};\n");
            }
            Decl::Enum(v, _) => {
                self.output.push_str("enum ");
                self.output.push_str(&v.name);
                self.output.push_str(" { ");
                for (i, x) in v.values.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&x.name);
                    if let Some(e) = &x.value {
                        self.output.push_str(" = ");
                        self.expr(e);
                    }
                }
                self.output.push_str(" };\n");
            }
            Decl::Typedef(v, _) => {
                self.output.push_str("typedef ");
                self.ty(&v.type_);
                self.output.push(' ');
                self.output.push_str(&v.name);
                self.output.push_str(";\n");
            }
            Decl::TypeAlias(n, t, _) => {
                self.output.push_str("using ");
                self.output.push_str(n);
                self.output.push_str(" = ");
                self.ty(t);
                self.output.push_str(";\n");
            }
            Decl::StaticAssert(e, m, _) => {
                self.output.push_str("static_assert(");
                self.expr(e);
                if !m.is_empty() {
                    self.output.push_str(", ");
                    self.output.push_str(&format!("{m:?}"));
                }
                self.output.push_str(");\n");
            }
            Decl::Asm(v, _) => self.output.push_str(&format!("asm({v:?});\n")),
        }
    }
    fn block(&mut self, b: &Block) {
        self.output.push_str("{\n");
        self.indent += 1;
        for s in &b.stmts {
            self.stmt(s);
        }
        self.indent -= 1;
        self.ind();
        self.output.push_str("}\n");
    }
    fn stmt(&mut self, s: &Stmt) {
        self.ind();
        match s {
            Stmt::Expr(e, _) => {
                self.expr(e);
                self.output.push_str(";\n");
            }
            Stmt::Decl(d, _) => {
                self.output
                    .truncate(self.output.trim_end_matches(' ').len());
                self.decl(d);
            }
            Stmt::Return(e, _) => {
                self.output.push_str("return");
                if let Some(e) = e {
                    self.output.push(' ');
                    self.expr(e);
                }
                self.output.push_str(";\n");
            }
            Stmt::Block(b, _) => self.block(b),
            Stmt::If(c, t, e, _) => {
                self.output.push_str("if (");
                self.expr(c);
                self.output.push_str(") ");
                self.stub_body(t);
                if e.is_some() {
                    self.output.push_str(" else {}");
                }
                self.output.push('\n');
            }
            Stmt::While(c, _, _) => {
                self.output.push_str("while (");
                self.expr(c);
                self.output.push_str(") {}\n");
            }
            Stmt::Break(_) => self.output.push_str("break;\n"),
            Stmt::Continue(_) => self.output.push_str("continue;\n"),
            Stmt::Throw(e, _) => {
                self.output.push_str("throw");
                if let Some(e) = e {
                    self.output.push(' ');
                    self.expr(e);
                }
                self.output.push_str(";\n");
            }
            Stmt::Empty(_) => self.output.push_str(";\n"),
            _ => self.output.push_str("{}\n"),
        }
    }
    fn stub_body(&mut self, s: &Stmt) {
        if let Stmt::Block(b, _) = s {
            self.block(b)
        } else {
            self.output.push_str("{}");
        }
    }
    fn ty(&mut self, e: &Expr) {
        match e {
            Expr::Unary(UnaryOp::Deref, v, _) | Expr::Deref(v, _) => {
                self.ty(v);
                self.output.push('*');
            }
            Expr::Unary(UnaryOp::Ref, v, _) | Expr::Ref(v, _) => {
                self.ty(v);
                self.output.push('&');
            }
            _ => self.expr(e),
        }
    }
    fn exprs(&mut self, v: &[Expr]) {
        for (i, e) in v.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.expr(e);
        }
    }
    fn expr(&mut self, e: &Expr) {
        match e {
            Expr::Int(v, _) => self.output.push_str(&v.to_string()),
            Expr::UInt(v, _) => self.output.push_str(&format!("{v}u")),
            Expr::Float(v, _) => self.output.push_str(&v.to_string()),
            Expr::String(v, _) => {
                if v.starts_with('"') {
                    self.output.push_str(v)
                } else {
                    self.output.push_str(&format!("{v:?}"))
                }
            }
            Expr::Char(v, _) => self.output.push_str(&format!("'{v}'")),
            Expr::Bool(v, _) => self.output.push_str(if *v { "true" } else { "false" }),
            Expr::NullPtr(_) => self.output.push_str("nullptr"),
            Expr::Ident(v, _) => self.output.push_str(v),
            Expr::This(_) => self.output.push_str("this"),
            Expr::Binary(l, o, r, _) => {
                self.output.push('(');
                self.expr(l);
                self.output.push(' ');
                self.output.push_str(binary_op(*o));
                self.output.push(' ');
                self.expr(r);
                self.output.push(')');
            }
            Expr::Call(f, a, _) => {
                self.expr(f);
                self.output.push('(');
                self.exprs(a);
                self.output.push(')');
            }
            Expr::Member(v, n, _) => {
                self.expr(v);
                self.output.push('.');
                self.output.push_str(n);
            }
            Expr::Arrow(v, n, _) => {
                self.expr(v);
                self.output.push_str("->");
                self.output.push_str(n);
            }
            Expr::Index(v, i, _) => {
                self.expr(v);
                self.output.push('[');
                self.expr(i);
                self.output.push(']');
            }
            Expr::Paren(v, _) => {
                self.output.push('(');
                self.expr(v);
                self.output.push(')');
            }
            Expr::Assign(l, r, _) => {
                self.expr(l);
                self.output.push_str(" = ");
                self.expr(r);
            }
            Expr::New(t, a, _) => {
                self.output.push_str("new ");
                self.ty(t);
                self.output.push('(');
                self.exprs(a);
                self.output.push(')');
            }
            Expr::Delete(v, _) => {
                self.output.push_str("delete ");
                self.expr(v);
            }
            Expr::BraceInit(v, _) => {
                self.output.push('{');
                self.exprs(v);
                self.output.push('}');
            }
            _ => self.output.push('0'),
        }
    }
}

fn binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
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
        BinaryOp::ModAssign => "%=",
        BinaryOp::AndAssign => "&=",
        BinaryOp::OrAssign => "|=",
        BinaryOp::XorAssign => "^=",
        BinaryOp::ShlAssign => "<<=",
        BinaryOp::ShrAssign => ">>=",
    }
}
