use crate::arena::Arena;
use crate::js::ast::*;

pub struct Printer {
    output: String,
    indent: usize,
    indent_size: usize,
}

impl Default for Printer {
    fn default() -> Self {
        Self::new()
    }
}

impl Printer {
    pub fn new() -> Self {
        Printer {
            output: String::new(),
            indent: 0,
            indent_size: 2,
        }
    }

    pub fn with_indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }

    fn indent_str(&self) -> String {
        " ".repeat(self.indent * self.indent_size)
    }

    fn push(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn push_indent(&mut self) {
        self.push(&self.indent_str());
    }

    fn push_line(&mut self, s: &str) {
        self.push_indent();
        self.push(s);
        self.push("\n");
    }

    pub fn print_program(&mut self, program: &Program, ast: &mut Arena<Expr>) -> String {
        self.output.clear();
        self.indent = 0;
        match program {
            Program::Script(script) => self.print_script(script, ast),
            Program::Module(module) => self.print_module(module, ast),
        }
        std::mem::take(&mut self.output)
    }

    pub fn print_script(&mut self, script: &Script, ast: &mut Arena<Expr>) {
        for stmt in &script.body {
            self.print_stmt(stmt, ast);
        }
    }

    pub fn print_module(&mut self, module: &Module, ast: &mut Arena<Expr>) {
        for item in &module.body {
            self.print_module_item(item, ast);
        }
    }

    fn print_module_item(&mut self, item: &ModuleItem, ast: &mut Arena<Expr>) {
        match item {
            ModuleItem::Stmt(stmt) => self.print_stmt(stmt, ast),
            ModuleItem::Decl(decl) => self.print_decl(decl, ast),
            ModuleItem::Import(imp) => self.print_import_decl(imp, ast),
            ModuleItem::Export(exp) => self.print_export_decl(exp, ast),
        }
    }

    fn print_import_decl(&mut self, imp: &ImportDecl, _ast: &Arena<Expr>) {
        self.push("import ");
        match imp.specifiers.len() {
            0 => {}
            _ => {
                let mut has_default = false;
                let mut has_namespace = false;
                let mut has_named = false;
                for spec in &imp.specifiers {
                    match spec {
                        ImportSpecifier::Default(_) => has_default = true,
                        ImportSpecifier::Namespace(_) => has_namespace = true,
                        ImportSpecifier::Named(_) => has_named = true,
                    }
                }
                if has_default || has_namespace || has_named {
                    if has_default {
                        for spec in &imp.specifiers {
                            if let ImportSpecifier::Default(d) = spec {
                                self.push(&d.local.name);
                                break;
                            }
                        }
                        if has_namespace || has_named {
                            self.push(", ");
                        }
                    }
                    if has_namespace {
                        for spec in &imp.specifiers {
                            if let ImportSpecifier::Namespace(ns) = spec {
                                self.push("* as ");
                                self.push(&ns.local.name);
                                break;
                            }
                        }
                        if has_named {
                            self.push(", ");
                        }
                    }
                    if has_named {
                        self.push("{ ");
                        let mut first = true;
                        for spec in &imp.specifiers {
                            if let ImportSpecifier::Named(n) = spec {
                                if !first {
                                    self.push(", ");
                                }
                                first = false;
                                if n.imported.name != n.local.name {
                                    self.push(&n.imported.name);
                                    self.push(" as ");
                                }
                                self.push(&n.local.name);
                            }
                        }
                        self.push(" }");
                    }
                    self.push(" from ");
                }
            }
        }
        self.push("\"");
        self.push(&imp.source.value);
        self.push("\";\n");
    }

    fn print_export_decl(&mut self, exp: &ExportDecl, ast: &mut Arena<Expr>) {
        match exp {
            ExportDecl::Named(n) => {
                if let Some(decl) = &n.decl {
                    self.push("export ");
                    self.print_decl(decl, ast);
                } else {
                    self.push("export { ");
                    for (i, spec) in n.specifiers.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.push(&spec.local.name);
                        if spec.exported.name != spec.local.name {
                            self.push(" as ");
                            self.push(&spec.exported.name);
                        }
                    }
                    self.push(" }");
                    if let Some(source) = &n.source {
                        self.push(" from \"");
                        self.push(&source.value);
                        self.push("\"");
                    }
                    self.push(";\n");
                }
            }
            ExportDecl::Default(d) => {
                self.push("export default ");
                self.print_expr(d.decl, ast);
                if d.has_assign {
                    self.push(";\n");
                }
            }
            ExportDecl::All(a) => {
                self.push("export * from \"");
                self.push(&a.source.value);
                self.push("\";\n");
            }
        }
    }

    fn print_stmt(&mut self, stmt: &Stmt, ast: &mut Arena<Expr>) {
        match stmt {
            Stmt::Block(s) => self.print_block_stmt(s, ast),
            Stmt::Empty(_) => self.push(";\n"),
            Stmt::Expr(s) => {
                self.print_expr(s.expr, ast);
                self.push(";\n");
            }
            Stmt::If(s) => {
                self.push("if (");
                self.print_expr(s.test, ast);
                self.push(") ");
                self.print_stmt(&s.consequent, ast);
                if let Some(alt) = &s.alternate {
                    self.push("else ");
                    self.print_stmt(alt, ast);
                }
            }
            Stmt::Switch(s) => {
                self.push("switch (");
                self.print_expr(s.discriminant, ast);
                self.push(") {\n");
                self.indent += 1;
                for case in &s.cases {
                    if let Some(test) = &case.test {
                        self.push_indent();
                        self.push("case ");
                        self.print_expr(*test, ast);
                        self.push(":\n");
                    } else {
                        self.push_line("default:");
                    }
                    self.indent += 1;
                    for stmt in &case.consequent {
                        self.print_stmt(stmt, ast);
                    }
                    self.indent -= 1;
                }
                self.indent -= 1;
                self.push_line("}");
            }
            Stmt::For(s) => {
                self.push("for (");
                if let Some(init) = &s.init {
                    match init {
                        ForInit::Expr(e) => self.print_expr(*e, ast),
                        ForInit::Decl(d) => self.print_decl(d, ast),
                    }
                }
                self.push("; ");
                if let Some(test) = &s.test {
                    self.print_expr(*test, ast);
                }
                self.push("; ");
                if let Some(update) = &s.update {
                    self.print_expr(*update, ast);
                }
                self.push(") ");
                self.print_stmt(&s.body, ast);
            }
            Stmt::ForIn(s) => {
                self.push("for (");
                match &s.left {
                    ForInit::Expr(e) => self.print_expr(*e, ast),
                    ForInit::Decl(d) => self.print_decl(d, ast),
                }
                self.push(" in ");
                self.print_expr(s.right, ast);
                self.push(") ");
                self.print_stmt(&s.body, ast);
            }
            Stmt::ForOf(s) => {
                self.push("for ");
                if s.await_ {
                    self.push("await ");
                }
                self.push("(");
                match &s.left {
                    ForInit::Expr(e) => self.print_expr(*e, ast),
                    ForInit::Decl(d) => self.print_decl(d, ast),
                }
                self.push(" of ");
                self.print_expr(s.right, ast);
                self.push(") ");
                self.print_stmt(&s.body, ast);
            }
            Stmt::While(s) => {
                self.push("while (");
                self.print_expr(s.test, ast);
                self.push(") ");
                self.print_stmt(&s.body, ast);
            }
            Stmt::DoWhile(s) => {
                self.push("do ");
                self.print_stmt(&s.body, ast);
                self.push("while (");
                self.print_expr(s.test, ast);
                self.push(");\n");
            }
            Stmt::Break(s) => {
                self.push("break");
                if let Some(label) = &s.label {
                    self.push(" ");
                    self.push(&label.name);
                }
                self.push(";\n");
            }
            Stmt::Continue(s) => {
                self.push("continue");
                if let Some(label) = &s.label {
                    self.push(" ");
                    self.push(&label.name);
                }
                self.push(";\n");
            }
            Stmt::Return(s) => {
                self.push("return");
                if let Some(arg) = &s.arg {
                    self.push(" ");
                    self.print_expr(*arg, ast);
                }
                self.push(";\n");
            }
            Stmt::Throw(s) => {
                self.push("throw ");
                self.print_expr(s.arg, ast);
                self.push(";\n");
            }
            Stmt::Try(s) => {
                self.push("try ");
                self.print_block_stmt(&s.block, ast);
                if let Some(handler) = &s.handler {
                    self.push("catch ");
                    if let Some(param) = &handler.param {
                        self.push("(");
                        self.print_pat(param, ast);
                        self.push(") ");
                    }
                    self.print_block_stmt(&handler.body, ast);
                }
                if let Some(finalizer) = &s.finalizer {
                    self.push("finally ");
                    self.print_block_stmt(finalizer, ast);
                }
            }
            Stmt::Debugger(_) => self.push_line("debugger;"),
            Stmt::Labelled(s) => {
                self.push(&s.label.name);
                self.push(": ");
                self.print_stmt(&s.body, ast);
            }
            Stmt::Decl(d) => self.print_decl(d, ast),
            Stmt::With(s) => {
                self.push("with (");
                self.print_expr(s.object, ast);
                self.push(") ");
                self.print_stmt(&s.body, ast);
            }
        }
    }

    fn print_block_stmt(&mut self, stmt: &BlockStmt, ast: &mut Arena<Expr>) {
        self.push("{\n");
        self.indent += 1;
        for s in &stmt.stmts {
            self.print_stmt(s, ast);
        }
        self.indent -= 1;
        self.push_line("}");
    }

    fn print_expr(&mut self, expr_ref: ExprRef, ast: &mut Arena<Expr>) {
        let expr = ast[expr_ref].clone();
        match &expr {
            Expr::Ident(id) => self.push(&id.name),
            Expr::Lit(lit) => self.print_lit(lit, ast),
            Expr::This(_) => self.push("this"),
            Expr::Super(_) => self.push("super"),
            Expr::Array(arr) => {
                self.push("[");
                for (i, elem) in arr.elements.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    if let Some(e) = elem {
                        self.print_expr(*e, ast);
                    }
                }
                self.push("]");
            }
            Expr::Object(obj) => {
                self.push("{\n");
                self.indent += 1;
                for prop in &obj.props {
                    self.push_indent();
                    match prop {
                        ObjProp::KeyValue(kv) => {
                            self.print_prop_name(&kv.key, ast);
                            self.push(": ");
                            self.print_expr(kv.value, ast);
                        }
                        ObjProp::Shorthand(id) => {
                            self.push(&id.name);
                        }
                        ObjProp::Method(m) => {
                            self.print_prop_name(&m.key, ast);
                            self.push("(");
                            for (i, param) in m.function.params.iter().enumerate() {
                                if i > 0 {
                                    self.push(", ");
                                }
                                self.print_pat(param, ast);
                            }
                            self.push(") ");
                            if let Some(body) = &m.function.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ObjProp::Getter(g) => {
                            self.push("get ");
                            self.print_prop_name(&g.key, ast);
                            self.push("() ");
                            if let Some(body) = &g.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ObjProp::Setter(s) => {
                            self.push("set ");
                            self.print_prop_name(&s.key, ast);
                            self.push("(");
                            self.print_pat(&s.param, ast);
                            self.push(") ");
                            if let Some(body) = &s.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ObjProp::Spread(sp) => {
                            self.push("...");
                            self.print_expr(sp.arg, ast);
                        }
                    }
                    self.push(",\n");
                }
                self.indent -= 1;
                self.push_indent();
                self.push("}");
            }
            Expr::Fn(f) => {
                self.push("function");
                if f.generator {
                    self.push("*");
                }
                if let Some(id) = &f.id {
                    self.push(" ");
                    self.push(&id.name);
                }
                self.push("(");
                for (i, param) in f.params.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.print_pat(param, ast);
                }
                self.push(") ");
                if let Some(body) = &f.body {
                    self.print_block_stmt(body, ast);
                } else {
                    self.push(";\n");
                }
            }
            Expr::Arrow(a) => {
                if a.async_ {
                    self.push("async ");
                }
                if a.params.len() == 1 && matches!(&a.params[0], Pat::Ident(_)) {
                    self.print_pat(&a.params[0], ast);
                } else {
                    self.push("(");
                    for (i, param) in a.params.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.print_pat(param, ast);
                    }
                    self.push(")");
                }
                self.push(" => ");
                match &a.body {
                    ArrowBody::Block(b) => self.print_block_stmt(b, ast),
                    ArrowBody::Expr(e) => self.print_expr(*e, ast),
                }
            }
            Expr::Class(c) => {
                self.push("class");
                if let Some(id) = &c.id {
                    self.push(" ");
                    self.push(&id.name);
                }
                if let Some(super_class) = &c.super_class {
                    self.push(" extends ");
                    self.print_expr(*super_class, ast);
                }
                self.push(" {\n");
                self.indent += 1;
                for member in &c.body {
                    self.push_indent();
                    match member {
                        ClassMember::Getter(g) => {
                            self.push("get ");
                            self.print_prop_name(&g.key, ast);
                            self.push("() ");
                            if let Some(body) = &g.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ClassMember::Setter(s) => {
                            self.push("set ");
                            self.print_prop_name(&s.key, ast);
                            self.push("(");
                            self.print_pat(&s.param, ast);
                            self.push(") ");
                            if let Some(body) = &s.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ClassMember::Method(m) => {
                            if m.is_static {
                                self.push("static ");
                            }
                            match m.kind {
                                MethodKind::Get => self.push("get "),
                                MethodKind::Set => self.push("set "),
                                MethodKind::Method => {}
                            }
                            self.print_prop_name(&m.key, ast);
                            self.push("(");
                            for (i, param) in m.function.params.iter().enumerate() {
                                if i > 0 {
                                    self.push(", ");
                                }
                                self.print_pat(param, ast);
                            }
                            self.push(") ");
                            if let Some(body) = &m.function.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ClassMember::Prop(p) => {
                            if p.is_static {
                                self.push("static ");
                            }
                            self.print_prop_name(&p.key, ast);
                            if let Some(value) = &p.value {
                                self.push(" = ");
                                self.print_expr(*value, ast);
                            }
                            self.push(";\n");
                        }
                        ClassMember::Ctor(c) => {
                            self.push("constructor(");
                            for (i, param) in c.params.iter().enumerate() {
                                if i > 0 {
                                    self.push(", ");
                                }
                                self.print_pat(param, ast);
                            }
                            self.push(") ");
                            if let Some(body) = &c.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ClassMember::StaticBlock(sb) => {
                            self.push("static ");
                            self.print_block_stmt(&sb.body, ast);
                        }
                        ClassMember::TSIndex(_) => {
                            self.push("[key: string]: any;\n");
                        }
                    }
                }
                self.indent -= 1;
                self.push_line("}");
            }
            Expr::New(n) => {
                self.push("new ");
                self.print_expr(n.callee, ast);
                self.push("(");
                for (i, arg) in n.args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.print_expr(*arg, ast);
                }
                self.push(")");
            }
            Expr::Call(c) => {
                self.print_expr(c.callee, ast);
                self.push("(");
                for (i, arg) in c.args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.print_expr(*arg, ast);
                }
                self.push(")");
            }
            Expr::OptionalCall(c) => {
                self.print_expr(c.callee, ast);
                self.push("?.(");
                for (i, arg) in c.args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.print_expr(*arg, ast);
                }
                self.push(")");
            }
            Expr::Member(m) => {
                self.print_expr(m.object, ast);
                if m.computed {
                    self.push("[");
                    self.print_expr(ast.alloc(Expr::clone(&m.property)), ast);
                    self.push("]");
                } else {
                    self.push(".");
                    self.print_expr(ast.alloc(Expr::clone(&m.property)), ast);
                }
            }
            Expr::OptionalMember(m) => {
                self.print_expr(m.object, ast);
                if m.computed {
                    self.push("?.[");
                    self.print_expr(ast.alloc(Expr::clone(&m.property)), ast);
                    self.push("]");
                } else {
                    self.push("?.");
                    self.print_expr(ast.alloc(Expr::clone(&m.property)), ast);
                }
            }
            Expr::Unary(u) => {
                self.push(match u.op {
                    UnaryOp::Minus => "-",
                    UnaryOp::Plus => "+",
                    UnaryOp::Not => "!",
                    UnaryOp::BitNot => "~",
                    UnaryOp::Typeof => "typeof ",
                    UnaryOp::Void => "void ",
                    UnaryOp::Delete => "delete ",
                });
                self.print_expr(u.arg, ast);
            }
            Expr::UnaryOp(u) => {
                self.push(match u.op {
                    UnaryOp::Minus => "-",
                    UnaryOp::Plus => "+",
                    UnaryOp::Not => "!",
                    UnaryOp::BitNot => "~",
                    UnaryOp::Typeof => "typeof ",
                    UnaryOp::Void => "void ",
                    UnaryOp::Delete => "delete ",
                });
                self.print_expr(u.arg, ast);
            }
            Expr::Binary(b) => {
                self.print_expr(b.left, ast);
                self.push(" ");
                self.push(match b.op {
                    BinaryOp::EqEq => "==",
                    BinaryOp::NotEq => "!=",
                    BinaryOp::EqEqEq => "===",
                    BinaryOp::NotEqEq => "!==",
                    BinaryOp::Lt => "<",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::Gt => ">",
                    BinaryOp::GtEq => ">=",
                    BinaryOp::LShift => "<<",
                    BinaryOp::RShift => ">>",
                    BinaryOp::RShift3 => ">>>",
                    BinaryOp::Plus => "+",
                    BinaryOp::Minus => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Pow => "**",
                    BinaryOp::BitAnd => "&",
                    BinaryOp::BitOr => "|",
                    BinaryOp::BitXor => "^",
                    BinaryOp::In => "in",
                    BinaryOp::Instanceof => "instanceof",
                    BinaryOp::StarStar => "**",
                });
                self.push(" ");
                self.print_expr(b.right, ast);
            }
            Expr::Logical(l) => {
                self.print_expr(l.left, ast);
                self.push(" ");
                self.push(match l.op {
                    LogicalOp::And => "&&",
                    LogicalOp::Or => "||",
                    LogicalOp::Nullish => "??",
                });
                self.push(" ");
                self.print_expr(l.right, ast);
            }
            Expr::Conditional(c) => {
                self.print_expr(c.test, ast);
                self.push(" ? ");
                self.print_expr(c.consequent, ast);
                self.push(" : ");
                self.print_expr(c.alternate, ast);
            }
            Expr::Assignment(a) => {
                self.print_expr(a.left, ast);
                self.push(" ");
                self.push(match a.op {
                    AssignOp::Assign => "=",
                    AssignOp::AddAssign => "+=",
                    AssignOp::SubAssign => "-=",
                    AssignOp::MulAssign => "*=",
                    AssignOp::DivAssign => "/=",
                    AssignOp::ModAssign => "%=",
                    AssignOp::PowAssign => "**=",
                    AssignOp::LShiftAssign => "<<=",
                    AssignOp::RShiftAssign => ">>=",
                    AssignOp::RShift3Assign => ">>>=",
                    AssignOp::BitAndAssign => "&=",
                    AssignOp::BitOrAssign => "|=",
                    AssignOp::BitXorAssign => "^=",
                    AssignOp::AndAssign => "&&=",
                    AssignOp::OrAssign => "||=",
                    AssignOp::NullishAssign => "??=",
                });
                self.push(" ");
                self.print_expr(a.right, ast);
            }
            Expr::Sequence(s) => {
                for (i, e) in s.expressions.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.print_expr(*e, ast);
                }
            }
            Expr::Update(u) => {
                if u.prefix {
                    self.push(match u.op {
                        UpdateOp::PlusPlus => "++",
                        UpdateOp::MinusMinus => "--",
                    });
                    self.print_expr(u.arg, ast);
                } else {
                    self.print_expr(u.arg, ast);
                    self.push(match u.op {
                        UpdateOp::PlusPlus => "++",
                        UpdateOp::MinusMinus => "--",
                    });
                }
            }
            Expr::Await(a) => {
                self.push("await ");
                self.print_expr(a.arg, ast);
            }
            Expr::Yield(y) => {
                self.push("yield");
                if y.delegate {
                    self.push("*");
                }
                if let Some(arg) = &y.arg {
                    self.push(" ");
                    self.print_expr(*arg, ast);
                }
            }
            Expr::Spread(s) => {
                self.push("...");
                self.print_expr(s.arg, ast);
            }
            Expr::Template(t) => {
                self.push("`");
                for (i, quasi) in t.quasis.iter().enumerate() {
                    self.push(&quasi.value);
                    if i < t.expressions.len() {
                        self.push("${");
                        self.print_expr(t.expressions[i], ast);
                        self.push("}");
                    }
                }
                self.push("`");
            }
            Expr::TaggedTemplate(t) => {
                self.print_expr(t.tag, ast);
                self.push("`");
                for (i, quasi) in t.template.quasis.iter().enumerate() {
                    self.push(&quasi.value);
                    if i < t.template.expressions.len() {
                        self.push("${");
                        self.print_expr(t.template.expressions[i], ast);
                        self.push("}");
                    }
                }
                self.push("`");
            }
            Expr::MetaProperty(m) => {
                self.push(&m.meta);
                self.push(".");
                self.push(&m.property);
            }
            Expr::Import(i) => {
                self.push("import(");
                self.print_expr(i.source, ast);
                self.push(")");
            }
            Expr::Parenthesized(p) => {
                self.push("(");
                self.print_expr(p.expr, ast);
                self.push(")");
            }
            Expr::Chain(c) => {
                self.print_expr(c.expr, ast);
            }
            Expr::JSXElement(el) => self.print_jsx_element(el, ast),
            Expr::JSXFragment(frag) => self.print_jsx_fragment(frag, ast),
            Expr::TSAs(e) => {
                self.print_expr(e.expr, ast);
                self.push(" as ");
                self.print_type_ann(&e.type_ann);
            }
            Expr::TSSatisfies(e) => {
                self.print_expr(e.expr, ast);
                self.push(" satisfies ");
                self.print_type_ann(&e.type_ann);
            }
            Expr::TSTypeAssertion(e) => {
                self.push("<");
                self.print_type_ann(&e.type_ann);
                self.push(">");
                self.print_expr(e.expr, ast);
            }
            Expr::TSNonNull(e) => {
                self.print_expr(e.expr, ast);
                self.push("!");
            }
            Expr::TSInst(e) => {
                self.print_expr(e.expr, ast);
                self.push("<");
                for (i, arg) in e.type_params.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.print_type_ann(arg);
                }
                self.push(">");
            }
            Expr::PrivateName(p) => {
                self.push("#");
                self.push(&p.name.name);
            }
            Expr::Invalid(_) => self.push("<invalid>"),
            Expr::Record(r) => {
                self.push("#{\n");
                self.indent += 1;
                for prop in &r.props {
                    self.push_indent();
                    match prop {
                        ObjProp::KeyValue(kv) => {
                            self.print_prop_name(&kv.key, ast);
                            self.push(": ");
                            self.print_expr(kv.value, ast);
                        }
                        ObjProp::Shorthand(id) => {
                            self.push(&id.name);
                        }
                        ObjProp::Method(m) => {
                            self.print_prop_name(&m.key, ast);
                            self.push("(");
                            for (i, param) in m.function.params.iter().enumerate() {
                                if i > 0 {
                                    self.push(", ");
                                }
                                self.print_pat(param, ast);
                            }
                            self.push(") ");
                            if let Some(body) = &m.function.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ObjProp::Getter(g) => {
                            self.push("get ");
                            self.print_prop_name(&g.key, ast);
                            self.push("() ");
                            if let Some(body) = &g.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ObjProp::Setter(s) => {
                            self.push("set ");
                            self.print_prop_name(&s.key, ast);
                            self.push("(");
                            self.print_pat(&s.param, ast);
                            self.push(") ");
                            if let Some(body) = &s.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ObjProp::Spread(sp) => {
                            self.push("...");
                            self.print_expr(sp.arg, ast);
                        }
                    }
                    self.push(",\n");
                }
                self.indent -= 1;
                self.push_indent();
                self.push("}");
            }
            Expr::Tuple(t) => {
                self.push("#[");
                for (i, el) in t.elements.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    if let Some(e) = el {
                        self.print_expr(*e, ast)
                    }
                }
                self.push("]");
            }
            Expr::Pipeline(p) => {
                self.print_expr(p.input, ast);
                self.push(" |> ");
                self.print_expr(p.body, ast);
            }
        }
    }

    fn print_jsx_element(&mut self, el: &JSXElement, ast: &mut Arena<Expr>) {
        self.push("<");
        self.print_jsx_name(&el.opening.name);
        for attr in &el.opening.attrs {
            self.push(" ");
            match attr {
                JSXAttr::Attr(a) => {
                    self.print_jsx_name(&a.name);
                    if let Some(value) = &a.value {
                        self.push("=");
                        match value {
                            JSXAttrVal::Str(s) => self.print_lit(&Lit::Str(s.clone()), ast),
                            JSXAttrVal::Expr(e) => {
                                self.push("{");
                                self.print_expr(*e, ast);
                                self.push("}");
                            }
                            JSXAttrVal::Element(el2) => self.print_jsx_element(el2, ast),
                            JSXAttrVal::Fragment(frag) => self.print_jsx_fragment(frag, ast),
                        }
                    }
                }
                JSXAttr::Spread(sp) => {
                    self.push("{...");
                    self.print_expr(sp.arg, ast);
                    self.push("}");
                }
            }
        }
        if el.opening.self_closing {
            self.push(" />");
        } else {
            self.push(">");
            for child in &el.children {
                match child {
                    JSXChild::Text(t) => self.push(t),
                    JSXChild::Expr(e) => {
                        self.push("{");
                        self.print_expr(*e, ast);
                        self.push("}");
                    }
                    JSXChild::Element(el2) => self.print_jsx_element(el2, ast),
                    JSXChild::Fragment(frag) => self.print_jsx_fragment(frag, ast),
                }
            }
            if let Some(closing) = &el.closing {
                self.push("</");
                self.print_jsx_name(&closing.name);
                self.push(">");
            }
        }
    }

    fn print_jsx_fragment(&mut self, frag: &JSXFragment, ast: &mut Arena<Expr>) {
        self.push("<>");
        for child in &frag.children {
            match child {
                JSXChild::Text(t) => self.push(t),
                JSXChild::Expr(e) => {
                    self.push("{");
                    self.print_expr(*e, ast);
                    self.push("}");
                }
                JSXChild::Element(el) => self.print_jsx_element(el, ast),
                JSXChild::Fragment(f) => self.print_jsx_fragment(f, ast),
            }
        }
        self.push("</>");
    }

    fn print_jsx_name(&mut self, name: &JSXName) {
        match name {
            JSXName::Ident(id) => self.push(&id.name),
            JSXName::Member(m) => {
                self.print_jsx_name(&m.object);
                self.push(".");
                self.push(&m.property.name);
            }
            JSXName::Namespace(ns) => {
                self.push(&ns.namespace.name);
                self.push(":");
                self.push(&ns.name.name);
            }
        }
    }

    fn print_lit(&mut self, lit: &Lit, ast: &mut Arena<Expr>) {
        match lit {
            Lit::Str(s) => {
                self.push("\"");
                self.push(&s.value);
                self.push("\"");
            }
            Lit::Num(n) => self.push(&n.raw),
            Lit::Bool(b) => self.push(if b.value { "true" } else { "false" }),
            Lit::Null(_) => self.push("null"),
            Lit::BigInt(b) => {
                self.push(&b.value);
                self.push("n");
            }
            Lit::RegExp(r) => {
                self.push("/");
                self.push(&r.pattern);
                self.push("/");
                self.push(&r.flags);
            }
            Lit::Template(t) => {
                self.push("`");
                for (i, quasi) in t.quasis.iter().enumerate() {
                    self.push(&quasi.value);
                    if i < t.expressions.len() {
                        self.push("${");
                        self.print_expr(t.expressions[i], ast);
                        self.push("}");
                    }
                }
                self.push("`");
            }
        }
    }

    fn print_type_ann(&mut self, ty: &TypeAnn) {
        match ty {
            TypeAnn::Any(_) => self.push("any"),
            TypeAnn::String(_) => self.push("string"),
            TypeAnn::Number(_) => self.push("number"),
            TypeAnn::Boolean(_) => self.push("boolean"),
            TypeAnn::Void(_) => self.push("void"),
            TypeAnn::Never(_) => self.push("never"),
            TypeAnn::Unknown(_) => self.push("unknown"),
            TypeAnn::Null(_) => self.push("null"),
            TypeAnn::Undefined(_) => self.push("undefined"),
            TypeAnn::Object(_) => self.push("object"),
            TypeAnn::Symbol(_) => self.push("symbol"),
            TypeAnn::BigInt(_) => self.push("bigint"),
            TypeAnn::Ident(id) => self.push(&id.name),
            TypeAnn::Array(a) => {
                self.print_type_ann(a);
                self.push("[]");
            }
            TypeAnn::Union(types) => {
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.push(" | ");
                    }
                    self.print_type_ann(t);
                }
            }
            TypeAnn::Intersection(types) => {
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.push(" & ");
                    }
                    self.print_type_ann(t);
                }
            }
            TypeAnn::Fn(params, ret) => {
                self.push("(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.print_type_ann(p);
                }
                self.push(") => ");
                self.print_type_ann(ret);
            }
            TypeAnn::Lit(l) => {
                self.push("\"");
                self.push(&l.value);
                self.push("\"");
            }
            TypeAnn::LitNum(n) => self.push(&n.raw),
            TypeAnn::LitBool(b) => self.push(if b.value { "true" } else { "false" }),
            TypeAnn::Generic(id, args) => {
                self.push(&id.name);
                if !args.is_empty() {
                    self.push("<");
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.print_type_ann(a);
                    }
                    self.push(">");
                }
            }
            TypeAnn::Tuple(types) => {
                self.push("[");
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.print_type_ann(t);
                }
                self.push("]");
            }
            TypeAnn::Rest(t) => {
                self.push("...");
                self.print_type_ann(t);
            }
            TypeAnn::Optional(t) => {
                self.print_type_ann(t);
                self.push("?");
            }
            TypeAnn::Readonly(t) => {
                self.push("readonly ");
                self.print_type_ann(t);
            }
            TypeAnn::KeyOf(t) => {
                self.push("keyof ");
                self.print_type_ann(t);
            }
            TypeAnn::Typeof(id) => {
                self.push("typeof ");
                self.push(&id.name);
            }
            TypeAnn::Infer(id) => {
                self.push("infer ");
                self.push(&id.name);
            }
            TypeAnn::Member(obj, prop) => {
                self.print_type_ann(obj);
                self.push(".");
                self.push(&prop.name);
            }
            TypeAnn::Paren(t) => {
                self.push("(");
                self.print_type_ann(t);
                self.push(")");
            }
            TypeAnn::Mapped(param, constraint) => {
                self.push("{\n");
                self.indent += 1;
                self.push_indent();
                self.push("[");
                self.push(&param.name);
                self.push(" in ");
                self.print_type_ann(constraint);
                self.push("]");
                self.push(": any;\n");
                self.indent -= 1;
                self.push_line("}");
            }
            TypeAnn::Conditional(check, extends, true_t, false_t) => {
                self.print_type_ann(check);
                self.push(" extends ");
                self.print_type_ann(extends);
                self.push(" ? ");
                self.print_type_ann(true_t);
                self.push(" : ");
                self.print_type_ann(false_t);
            }
            TypeAnn::This(_) => self.push("this"),
            TypeAnn::Pred(_, _) => self.push("/* pred */ any"),
            TypeAnn::Indexed(obj, idx) => {
                self.print_type_ann(obj);
                self.push("[");
                self.print_type_ann(idx);
                self.push("]");
            }
            TypeAnn::TsNull(_) => self.push("null"),
        }
    }

    fn print_prop_name(&mut self, name: &PropName, ast: &mut Arena<Expr>) {
        match name {
            PropName::Ident(id) => self.push(&id.name),
            PropName::Str(s) => {
                self.push("\"");
                self.push(&s.value);
                self.push("\"");
            }
            PropName::Num(n) => self.push(&n.raw),
            PropName::Computed(e) => {
                self.push("[");
                self.print_expr(*e, ast);
                self.push("]");
            }
        }
    }

    fn print_decl(&mut self, decl: &Decl, ast: &mut Arena<Expr>) {
        match decl {
            Decl::Var(d) => self.print_var_decl(d, ast),
            Decl::Fn(d) => {
                self.push("function");
                if d.generator {
                    self.push("*");
                }
                self.push(" ");
                self.push(&d.id.name);
                self.push("(");
                for (i, param) in d.params.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.print_pat(param, ast);
                }
                self.push(") ");
                if let Some(body) = &d.body {
                    self.print_block_stmt(body, ast);
                } else {
                    self.push(";\n");
                }
            }
            Decl::Class(d) => {
                self.push("class ");
                self.push(&d.id.name);
                if let Some(super_class) = &d.super_class {
                    self.push(" extends ");
                    self.print_expr(*super_class, ast);
                }
                self.push(" {\n");
                self.indent += 1;
                for member in &d.body {
                    self.push_indent();
                    match member {
                        ClassMember::Getter(g) => {
                            self.push("get ");
                            self.print_prop_name(&g.key, ast);
                            self.push("() ");
                            if let Some(body) = &g.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ClassMember::Setter(s) => {
                            self.push("set ");
                            self.print_prop_name(&s.key, ast);
                            self.push("(");
                            self.print_pat(&s.param, ast);
                            self.push(") ");
                            if let Some(body) = &s.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ClassMember::Method(m) => {
                            if m.is_static {
                                self.push("static ");
                            }
                            match m.kind {
                                MethodKind::Get => self.push("get "),
                                MethodKind::Set => self.push("set "),
                                MethodKind::Method => {}
                            }
                            self.print_prop_name(&m.key, ast);
                            self.push("(");
                            for (i, param) in m.function.params.iter().enumerate() {
                                if i > 0 {
                                    self.push(", ");
                                }
                                self.print_pat(param, ast);
                            }
                            self.push(") ");
                            if let Some(body) = &m.function.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ClassMember::Prop(p) => {
                            if p.is_static {
                                self.push("static ");
                            }
                            self.print_prop_name(&p.key, ast);
                            if let Some(value) = &p.value {
                                self.push(" = ");
                                self.print_expr(*value, ast);
                            }
                            self.push(";\n");
                        }
                        ClassMember::Ctor(c) => {
                            self.push("constructor(");
                            for (i, param) in c.params.iter().enumerate() {
                                if i > 0 {
                                    self.push(", ");
                                }
                                self.print_pat(param, ast);
                            }
                            self.push(") ");
                            if let Some(body) = &c.body {
                                self.print_block_stmt(body, ast);
                            } else {
                                self.push(";\n");
                            }
                        }
                        ClassMember::StaticBlock(sb) => {
                            self.push("static ");
                            self.print_block_stmt(&sb.body, ast);
                        }
                        ClassMember::TSIndex(_) => {
                            self.push("[key: string]: any;\n");
                        }
                    }
                }
                self.indent -= 1;
                self.push_line("}");
            }
            Decl::TsInterface(d) => {
                self.push("interface ");
                self.push(&d.id.name);
                self.push(" {\n");
                self.indent += 1;
                self.indent -= 1;
                self.push_line("}");
            }
            Decl::TsTypeAlias(d) => {
                self.push("type ");
                self.push(&d.id.name);
                self.push(" = ");
                self.print_type_ann(&d.type_ann);
                self.push(";\n");
            }
            Decl::TsEnum(d) => {
                self.push(if d.is_const { "const enum " } else { "enum " });
                self.push(&d.id.name);
                self.push(" {\n");
                self.indent += 1;
                for member in &d.members {
                    self.push_indent();
                    self.push(&member.id.name);
                    if let Some(init) = &member.init {
                        self.push(" = ");
                        self.print_expr(*init, ast);
                    }
                    self.push(",\n");
                }
                self.indent -= 1;
                self.push_line("}");
            }
            Decl::TsModule(d) => {
                self.push(if d.is_namespace {
                    "namespace "
                } else {
                    "module "
                });
                self.push(&d.id.name);
                self.push(" {\n");
                self.indent += 1;
                for stmt in &d.body {
                    self.print_stmt(stmt, ast);
                }
                self.indent -= 1;
                self.push_line("}");
            }
        }
    }

    fn print_var_decl(&mut self, decl: &VarDecl, ast: &mut Arena<Expr>) {
        self.push(match decl.kind {
            VarKind::Var => "var ",
            VarKind::Let => "let ",
            VarKind::Const => "const ",
            VarKind::Using => "using ",
        });
        for (i, d) in decl.decls.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            self.print_pat(&d.name, ast);
            if let Some(init) = &d.init {
                self.push(" = ");
                self.print_expr(*init, ast);
            }
        }
        self.push(";\n");
    }

    fn print_pat(&mut self, pat: &Pat, ast: &mut Arena<Expr>) {
        match pat {
            Pat::Ident(bi) => self.push(&bi.id.name),
            Pat::Object(op) => {
                self.push("{ ");
                for (i, prop) in op.props.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    match prop {
                        ObjectPatProp::KeyValue(kv) => {
                            self.print_prop_name(&kv.key, ast);
                            self.push(": ");
                            self.print_pat(&kv.value, ast);
                        }
                        ObjectPatProp::Shorthand(bi) => self.push(&bi.id.name),
                        ObjectPatProp::Rest(rp) => {
                            self.push("...");
                            self.print_pat(&rp.arg, ast);
                        }
                    }
                }
                if let Some(rest) = &op.rest {
                    if !op.props.is_empty() {
                        self.push(", ");
                    }
                    self.push("...");
                    self.print_pat(&rest.arg, ast);
                }
                self.push(" }");
            }
            Pat::Array(ap) => {
                self.push("[");
                for (i, elem) in ap.elements.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    if let Some(e) = elem {
                        self.print_pat(e, ast);
                    }
                }
                if let Some(rest) = &ap.rest {
                    if !ap.elements.is_empty() {
                        self.push(", ");
                    }
                    self.push("...");
                    self.print_pat(&rest.arg, ast);
                }
                self.push("]");
            }
            Pat::Rest(rp) => {
                self.push("...");
                self.print_pat(&rp.arg, ast);
            }
            Pat::Assign(ap) => {
                self.print_pat(&ap.left, ast);
                self.push(" = ");
                self.print_expr(ap.right, ast);
            }
            Pat::Expr(e) => self.print_expr(*e, ast),
            Pat::Invalid(_) => self.push("<invalid>"),
        }
    }
}
