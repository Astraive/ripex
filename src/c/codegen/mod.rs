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
    pub fn generate(&mut self, program: &Program) -> String {
        self.output.clear();
        self.indent = 0;
        for stmt in &program.decls {
            self.stmt(stmt);
        }
        self.output.clone()
    }
    fn ind(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent));
    }
    fn stmt(&mut self, stmt: &Stmt) {
        self.ind();
        match stmt {
            Stmt::Expr(e, _) => {
                self.expr(e);
                self.output.push_str(";\n");
            }
            Stmt::Decl(f, _) => {
                self.function(f);
            }
            Stmt::VarDecl(v, _) => {
                self.var(v);
                self.output.push_str(";\n");
            }
            Stmt::If(t, b, e, _) => {
                self.output.push_str("if (");
                self.expr(t);
                self.output.push_str(") ");
                self.body(b);
                if let Some(e) = e {
                    self.output.push_str(" else ");
                    self.body(e);
                }
                self.output.push('\n');
            }
            Stmt::Switch(v, c, _) => {
                self.output.push_str("switch (");
                self.expr(v);
                self.output.push_str(") {\n");
                self.indent += 1;
                for x in c {
                    self.ind();
                    if let Some(v) = &x.expr {
                        self.output.push_str("case ");
                        self.expr(v);
                    } else {
                        self.output.push_str("default");
                    }
                    self.output.push_str(":\n");
                    self.indent += 1;
                    for s in &x.stmts {
                        self.stmt(s);
                    }
                    self.indent -= 1;
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("}\n");
            }
            Stmt::While(t, b, _) => {
                self.output.push_str("while (");
                self.expr(t);
                self.output.push_str(") ");
                self.body(b);
                self.output.push('\n');
            }
            Stmt::Do(b, t, _) => {
                self.output.push_str("do ");
                self.body(b);
                self.output.push_str(" while (");
                self.expr(t);
                self.output.push_str(");\n");
            }
            Stmt::For(i, t, u, b, _) => {
                self.output.push_str("for (");
                if let Some(i) = i {
                    self.inline_stmt(i);
                }
                self.output.push_str("; ");
                if let Some(t) = t {
                    self.expr(t);
                }
                self.output.push_str("; ");
                if let Some(u) = u {
                    self.inline_stmt(u);
                }
                self.output.push_str(") ");
                self.body(b);
                self.output.push('\n');
            }
            Stmt::Return(v, _) => {
                self.output.push_str("return");
                if let Some(v) = v {
                    self.output.push(' ');
                    self.expr(v);
                }
                self.output.push_str(";\n");
            }
            Stmt::Break(_) => self.output.push_str("break;\n"),
            Stmt::Continue(_) => self.output.push_str("continue;\n"),
            Stmt::Goto(n, _) => self.output.push_str(&format!("goto {n};\n")),
            Stmt::Label(n, _) => self.output.push_str(&format!("{n}:\n")),
            Stmt::Block(b, _) => self.block(b),
            Stmt::Empty(_) => self.output.push_str(";\n"),
            Stmt::Preprocessor(v, _) => {
                self.preproc(v);
                self.output.push('\n');
            }
        }
    }
    fn body(&mut self, s: &Stmt) {
        match s {
            Stmt::Block(b, _) => {
                self.output.pop();
                self.block(b);
            }
            _ => {
                self.output.push_str("{\n");
                self.indent += 1;
                self.stmt(s);
                self.indent -= 1;
                self.ind();
                self.output.push('}');
            }
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
    fn inline_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Expr(e, _) => self.expr(e),
            Stmt::VarDecl(v, _) => self.var(v),
            _ => {}
        }
    }
    fn function(&mut self, f: &FuncDecl) {
        if let Some(v) = &f.storage_class {
            self.output.push_str(v);
            self.output.push(' ');
        }
        if f.is_inline {
            self.output.push_str("inline ");
        }
        self.type_expr(&f.return_type);
        self.output.push(' ');
        self.output.push_str(&f.name);
        self.output.push('(');
        for (i, p) in f.params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.type_expr(&p.type_);
            if let Some(n) = &p.name {
                self.output.push(' ');
                self.output.push_str(n);
            }
        }
        if f.is_variadic {
            if !f.params.is_empty() {
                self.output.push_str(", ");
            }
            self.output.push_str("...");
        }
        self.output.push(')');
        if let Some(b) = &f.body {
            self.output.push(' ');
            self.block(b);
        } else {
            self.output.push_str(";\n");
        }
    }
    fn var(&mut self, v: &VarDecl) {
        if let Some(s) = &v.storage_class {
            self.output.push_str(s);
            self.output.push(' ');
        }
        if v.is_const {
            self.output.push_str("const ");
        }
        self.type_expr(&v.type_);
        if !v.name.is_empty() {
            self.output.push(' ');
            self.output.push_str(&v.name);
        }
        if let Some(i) = &v.init {
            self.output.push_str(" = ");
            self.expr(i);
        }
    }
    fn preproc(&mut self, p: &PreprocDirective) {
        match p {
            PreprocDirective::Include(v, _) => self.output.push_str(&format!("#include {v}")),
            PreprocDirective::Define(n, v, _) => {
                self.output.push_str("#define ");
                self.output.push_str(n);
                if let Some(v) = v {
                    self.output.push(' ');
                    self.output.push_str(v);
                }
            }
            PreprocDirective::Undef(v, _) => self.output.push_str(&format!("#undef {v}")),
            PreprocDirective::Ifdef(v, _) => self.output.push_str(&format!("#ifdef {v}")),
            PreprocDirective::Ifndef(v, _) => self.output.push_str(&format!("#ifndef {v}")),
            PreprocDirective::If(v, _) => self.output.push_str(&format!("#if {v}")),
            PreprocDirective::Else(_) => self.output.push_str("#else"),
            PreprocDirective::Elif(v, _) => self.output.push_str(&format!("#elif {v}")),
            PreprocDirective::Endif(_) => self.output.push_str("#endif"),
            PreprocDirective::Error(v, _) => self.output.push_str(&format!("#error {v}")),
            PreprocDirective::Pragma(v, _) => self.output.push_str(&format!("#pragma {v}")),
            PreprocDirective::Line(v, _) => self.output.push_str(&format!("#line {v}")),
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
    fn type_expr(&mut self, value: &Expr) {
        match value {
            Expr::Unary(UnaryOp::Deref, inner, _) | Expr::Deref(inner, _) => {
                self.type_expr(inner);
                self.output.push_str(" *");
            }
            Expr::Unary(UnaryOp::Ref, inner, _) | Expr::Ref(inner, _) => {
                self.type_expr(inner);
                self.output.push_str(" *");
            }
            _ => self.expr(value),
        }
    }
    fn expr(&mut self, e: &Expr) {
        match e {
            Expr::Int(v, _) => self.output.push_str(&v.to_string()),
            Expr::UInt(v, _) => self.output.push_str(&format!("{v}u")),
            Expr::Float(v, _) => self.output.push_str(&v.to_string()),
            Expr::String(v, _) => {
                if v.starts_with('"') {
                    self.output.push_str(v);
                } else {
                    self.output.push_str(&format!("{v:?}"));
                }
            }
            Expr::Char(v, _) => self.output.push_str(&format!("'{v}'")),
            Expr::Ident(v, _) => self.output.push_str(v),
            Expr::Binary(l, o, r, _) => {
                self.output.push('(');
                self.expr(l);
                self.output.push(' ');
                self.output.push_str(bin(*o));
                self.output.push(' ');
                self.expr(r);
                self.output.push(')');
            }
            Expr::Unary(o, v, _) => match o {
                UnaryOp::PostInc | UnaryOp::PostDec => {
                    self.expr(v);
                    self.output.push_str(un(*o));
                }
                _ => {
                    self.output.push_str(un(*o));
                    self.expr(v);
                }
            },
            Expr::Call(f, a, _) => {
                self.expr(f);
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
            Expr::Deref(v, _) => {
                self.output.push('*');
                self.expr(v);
            }
            Expr::Ref(v, _) => {
                self.output.push('&');
                self.expr(v);
            }
            Expr::Cast(t, v, _) => {
                self.output.push('(');
                self.type_expr(t);
                self.output.push(')');
                self.expr(v);
            }
            Expr::Sizeof(v, _) => {
                self.output.push_str("sizeof(");
                self.expr(v);
                self.output.push(')');
            }
            Expr::Alignof(v, _) => {
                self.output.push_str("_Alignof(");
                self.expr(v);
                self.output.push(')');
            }
            Expr::Ternary(c, t, f, _) => {
                self.output.push('(');
                self.expr(c);
                self.output.push_str(" ? ");
                self.expr(t);
                self.output.push_str(" : ");
                self.expr(f);
                self.output.push(')');
            }
            Expr::Comma(v, _) => {
                self.output.push('(');
                self.exprs(v);
                self.output.push(')');
            }
            Expr::StmtExpr(v, _) => {
                self.output.push_str("({ ");
                for s in v {
                    self.inline_stmt(s);
                    self.output.push(';');
                }
                self.output.push_str(" })");
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
            Expr::StringConcat(v, _) => {
                for s in v {
                    self.output.push_str(&format!("{s:?}"));
                }
            }
            Expr::DeclSpec(v, _) => self.spec(v),
            Expr::Error(_) => self.output.push('0'),
        }
    }
    fn spec(&mut self, s: &DeclSpec) {
        match s {
            DeclSpec::Void => self.output.push_str("void"),
            DeclSpec::Char => self.output.push_str("char"),
            DeclSpec::Short => self.output.push_str("short"),
            DeclSpec::Int => self.output.push_str("int"),
            DeclSpec::Long => self.output.push_str("long"),
            DeclSpec::Float => self.output.push_str("float"),
            DeclSpec::Double => self.output.push_str("double"),
            DeclSpec::Signed => self.output.push_str("signed"),
            DeclSpec::Unsigned => self.output.push_str("unsigned"),
            DeclSpec::Struct(n, f) => {
                self.output.push_str("struct ");
                self.output.push_str(n);
                if let Some(f) = f {
                    self.output.push_str(" { ");
                    for x in f {
                        self.type_expr(&x.type_);
                        self.output.push(' ');
                        self.output.push_str(&x.name);
                        if let Some(b) = x.bitfield {
                            self.output.push_str(&format!(": {b}"));
                        }
                        self.output.push_str("; ");
                    }
                    self.output.push('}');
                }
            }
            DeclSpec::Union(n, _) => {
                self.output.push_str("union ");
                self.output.push_str(n);
            }
            DeclSpec::Enum(n, v) => {
                self.output.push_str("enum ");
                self.output.push_str(n);
                if let Some(v) = v {
                    self.output.push_str(" { ");
                    for (i, x) in v.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.output.push_str(&x.name);
                        if let Some(v) = &x.value {
                            self.output.push_str(" = ");
                            self.expr(v);
                        }
                    }
                    self.output.push_str(" }");
                }
            }
            DeclSpec::Typedef(v, n) => {
                self.output.push_str("typedef ");
                self.type_expr(v);
                self.output.push(' ');
                self.output.push_str(n);
            }
            DeclSpec::TypeName(n) => self.output.push_str(n),
            DeclSpec::Const => self.output.push_str("const"),
            DeclSpec::Volatile => self.output.push_str("volatile"),
            DeclSpec::Restrict => self.output.push_str("restrict"),
            DeclSpec::Extern => self.output.push_str("extern"),
            DeclSpec::Static => self.output.push_str("static"),
            DeclSpec::Register => self.output.push_str("register"),
            DeclSpec::Inline => self.output.push_str("inline"),
            DeclSpec::Atomic => self.output.push_str("_Atomic"),
            DeclSpec::Auto => self.output.push_str("auto"),
            DeclSpec::ThreadLocal => self.output.push_str("_Thread_local"),
        }
    }
}
fn bin(o: BinaryOp) -> &'static str {
    match o {
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
fn un(o: UnaryOp) -> &'static str {
    match o {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
        UnaryOp::BitNot => "~",
        UnaryOp::Deref => "*",
        UnaryOp::Ref => "&",
        UnaryOp::Plus => "+",
        UnaryOp::PreInc | UnaryOp::PostInc => "++",
        UnaryOp::PreDec | UnaryOp::PostDec => "--",
    }
}
