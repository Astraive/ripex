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
            self.decl(d, None);
        }
        self.output.clone()
    }
    fn ind(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent));
    }
    fn vis(&mut self, v: Visibility) {
        self.output.push_str(match v {
            Visibility::Public => "public ",
            Visibility::Private => "private ",
            Visibility::Protected => "protected ",
            Visibility::Internal => "internal ",
            Visibility::ProtectedInternal => "protected internal ",
            Visibility::PrivateProtected => "private protected ",
            Visibility::None => "",
        });
    }
    fn decl(&mut self, d: &Decl, owner: Option<&str>) {
        self.ind();
        match d {
            Decl::Namespace(n, v, _) => {
                self.output.push_str("namespace ");
                self.output.push_str(n);
                self.output.push_str(" {\n");
                self.indent += 1;
                for d in v {
                    self.decl(d, None);
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("}\n");
            }
            Decl::Class(v, _) | Decl::Record(v, _) => {
                self.vis(v.visibility);
                if v.is_static {
                    self.output.push_str("static ");
                }
                if v.is_abstract {
                    self.output.push_str("abstract ");
                }
                if v.is_sealed {
                    self.output.push_str("sealed ");
                }
                self.output.push_str(if matches!(d, Decl::Record(..)) {
                    "record "
                } else {
                    "class "
                });
                self.output.push_str(&v.name);
                self.type_params(&v.type_params);
                self.output.push_str(" {\n");
                self.indent += 1;
                for m in &v.members {
                    self.decl(m, Some(&v.name));
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("}\n");
            }
            Decl::Struct(v, _) => {
                self.vis(v.visibility);
                self.output.push_str("struct ");
                self.output.push_str(&v.name);
                self.type_params(&v.type_params);
                self.output.push_str(" {\n");
                self.indent += 1;
                for m in &v.members {
                    self.decl(m, Some(&v.name));
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("}\n");
            }
            Decl::Interface(v, _) => {
                self.vis(v.visibility);
                self.output.push_str("interface ");
                self.output.push_str(&v.name);
                self.type_params(&v.type_params);
                self.output.push_str(" {\n");
                self.indent += 1;
                for m in &v.members {
                    self.decl(m, Some(&v.name));
                }
                self.indent -= 1;
                self.ind();
                self.output.push_str("}\n");
            }
            Decl::Enum(v, _) => {
                self.vis(v.visibility);
                self.output.push_str("enum ");
                self.output.push_str(&v.name);
                self.output.push_str(" { ");
                for (i, m) in v.members.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&m.name);
                    if let Some(x) = &m.value {
                        self.output.push_str(" = ");
                        self.expr(x);
                    }
                }
                self.output.push_str(" }\n");
            }
            Decl::Delegate(v, _) => {
                self.vis(v.visibility);
                self.output.push_str("delegate ");
                self.ty(&v.return_type);
                self.output.push(' ');
                self.output.push_str(&v.name);
                self.params(&v.params);
                self.output.push_str(";\n");
            }
            Decl::Event(v, _) => {
                self.vis(v.visibility);
                self.output.push_str("event ");
                self.ty(&v.type_);
                self.output.push(' ');
                self.output.push_str(&v.name);
                self.output.push_str(";\n");
            }
            Decl::Field(v, _) => {
                self.vis(v.visibility);
                if v.is_static {
                    self.output.push_str("static ");
                }
                if v.is_const {
                    self.output.push_str("const ");
                } else if v.is_readonly {
                    self.output.push_str("readonly ");
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
            Decl::Property(v, _) => {
                self.vis(v.visibility);
                self.ty(&v.type_);
                self.output.push(' ');
                self.output.push_str(&v.name);
                self.output.push_str(" { ");
                if v.getter.is_some() || v.is_auto {
                    self.output.push_str("get; ");
                }
                if v.setter.is_some() || v.is_auto {
                    self.output.push_str("set; ");
                }
                self.output.push('}');
                if let Some(x) = &v.init {
                    self.output.push_str(" = ");
                    self.expr(x);
                    self.output.push(';');
                }
                self.output.push('\n');
            }
            Decl::Method(v, _) => {
                self.vis(v.visibility);
                if v.is_static {
                    self.output.push_str("static ");
                }
                if v.is_async {
                    self.output.push_str("async ");
                }
                self.ty(&v.return_type);
                self.output.push(' ');
                self.output.push_str(&v.name);
                self.type_params(&v.type_params);
                self.params(&v.params);
                if let Some(b) = &v.body {
                    self.output.push(' ');
                    self.block(b);
                } else {
                    self.output.push_str(";\n");
                }
            }
            Decl::Constructor(v, _) => {
                self.vis(v.visibility);
                if v.is_static {
                    self.output.push_str("static ");
                }
                self.output.push_str(owner.unwrap_or("RipexType"));
                self.params(&v.params);
                self.output.push_str(" {\n");
                self.ind();
                self.output.push_str("}\n");
            }
            Decl::Destructor(_, _) => {
                self.output.push('~');
                self.output.push_str(owner.unwrap_or("RipexType"));
                self.output.push_str("() {}\n");
            }
            Decl::Using(v, _) => {
                self.output.push_str("using ");
                if let Some(a) = &v.alias {
                    self.output.push_str(a);
                    self.output.push_str(" = ");
                }
                self.output.push_str(&v.namespace);
                self.output.push_str(";\n");
            }
            Decl::UsingStatic(v, _) => self.output.push_str(&format!("using static {v};\n")),
            Decl::ExternAlias(v, _) => self.output.push_str(&format!("extern alias {v};\n")),
            Decl::Operator(v, _) => {
                self.vis(Visibility::Public);
                self.output.push_str("static ");
                self.ty(&v.return_type);
                self.output.push_str(" operator ");
                self.output.push_str(&v.op);
                self.params(&v.params);
                self.output.push_str(" {}\n");
            }
            Decl::Conversion(v, _) => {
                self.output.push_str(if v.is_explicit {
                    "explicit operator "
                } else {
                    "implicit operator "
                });
                self.ty(&v.return_type);
                self.params(std::slice::from_ref(&v.param));
                self.output.push_str(" {}\n");
            }
        }
    }
    fn type_params(&mut self, v: &[TypeParam]) {
        if v.is_empty() {
            return;
        }
        self.output.push('<');
        for (i, x) in v.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&x.name);
        }
        self.output.push('>');
    }
    fn params(&mut self, v: &[ParamDecl]) {
        self.output.push('(');
        for (i, p) in v.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            if p.is_this {
                self.output.push_str("this ");
            }
            if p.is_ref {
                self.output.push_str("ref ");
            }
            if p.is_out {
                self.output.push_str("out ");
            }
            if p.is_in {
                self.output.push_str("in ");
            }
            if p.is_params {
                self.output.push_str("params ");
            }
            self.ty(&p.type_);
            self.output.push(' ');
            self.output.push_str(&p.name);
            if let Some(x) = &p.default {
                self.output.push_str(" = ");
                self.expr(x);
            }
        }
        self.output.push(')');
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
                self.decl(d, None);
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
            _ => self.output.push_str(";\n"),
        }
    }
    fn ty(&mut self, e: &Expr) {
        self.expr(e)
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
            Expr::Long(v, _) => self.output.push_str(&format!("{v}L")),
            Expr::ULong(v, _) => self.output.push_str(&format!("{v}UL")),
            Expr::Float(v, _) => self.output.push_str(&format!("{v}f")),
            Expr::Double(v, _) => self.output.push_str(&v.to_string()),
            Expr::Decimal(v, _) => self.output.push_str(&format!("{v}m")),
            Expr::String(v, _) => {
                if v.starts_with('"') {
                    self.output.push_str(v)
                } else {
                    self.output.push_str(&format!("{v:?}"))
                }
            }
            Expr::Char(v, _) => self.output.push_str(&format!("'{v}'")),
            Expr::Bool(v, _) => self.output.push_str(if *v { "true" } else { "false" }),
            Expr::Null(_) => self.output.push_str("null"),
            Expr::Ident(v, _) => self.output.push_str(v),
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
            Expr::Array(v, _) => {
                self.output.push_str("new[] {");
                self.exprs(v);
                self.output.push('}');
            }
            Expr::Await(v, _) => {
                self.output.push_str("await ");
                self.expr(v);
            }
            Expr::Default(v, _) => {
                self.output.push_str("default(");
                self.ty(v);
                self.output.push(')');
            }
            Expr::Typeof(v, _) => {
                self.output.push_str("typeof(");
                self.ty(v);
                self.output.push(')');
            }
            Expr::Nameof(v, _) => {
                self.output.push_str("nameof(");
                self.expr(v);
                self.output.push(')');
            }
            _ => self.output.push_str("null"),
        }
    }
}
