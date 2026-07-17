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
        for stmt in &program.stmts {
            self.emit_stmt(stmt);
        }
        self.output.clone()
    }

    fn push_indent(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent));
    }

    fn line(&mut self, text: &str) {
        self.push_indent();
        self.output.push_str(text);
        self.output.push('\n');
    }

    fn suite(&mut self, stmts: &[Stmt]) {
        self.indent += 1;
        if stmts.is_empty() {
            self.line("pass");
        } else {
            for stmt in stmts {
                self.emit_stmt(stmt);
            }
        }
        self.indent -= 1;
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        self.push_indent();
        match stmt {
            Stmt::Expr(expr, _) => self.emit_expr(expr),
            Stmt::Assign(left, right, _) => {
                self.emit_expr(left);
                self.output.push_str(" = ");
                self.emit_expr(right);
            }
            Stmt::AugAssign(left, op, right, _) => {
                self.emit_expr(left);
                self.output.push(' ');
                self.output.push_str(binary_op(*op));
                self.output.push_str("= ");
                self.emit_expr(right);
            }
            Stmt::AnnAssign(target, ty, value, _) => {
                self.emit_expr(target);
                self.output.push_str(": ");
                self.emit_expr(ty);
                if let Some(value) = value {
                    self.output.push_str(" = ");
                    self.emit_expr(value);
                }
            }
            Stmt::If(test, body, alternate, _) => {
                self.output.push_str("if ");
                self.emit_expr(test);
                self.output.push_str(":\n");
                self.suite(body);
                if !alternate.is_empty() {
                    self.line("else:");
                    self.suite(alternate);
                }
                return;
            }
            Stmt::While(test, body, alternate, _) => {
                self.output.push_str("while ");
                self.emit_expr(test);
                self.output.push_str(":\n");
                self.suite(body);
                if let Some(alternate) = alternate {
                    self.line("else:");
                    self.suite(alternate);
                }
                return;
            }
            Stmt::For(target, iter, body, alternate, _) => {
                self.output.push_str("for ");
                self.emit_expr(target);
                self.output.push_str(" in ");
                self.emit_expr(iter);
                self.output.push_str(":\n");
                self.suite(body);
                if let Some(alternate) = alternate {
                    self.line("else:");
                    self.suite(alternate);
                }
                return;
            }
            Stmt::With(items, body, _) => {
                self.output.push_str("with ");
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expr(&item.context);
                    if let Some(target) = &item.target {
                        self.output.push_str(" as ");
                        self.emit_expr(target);
                    }
                }
                self.output.push_str(":\n");
                self.suite(body);
                return;
            }
            Stmt::Match(subject, cases, _) => {
                self.output.push_str("match ");
                self.emit_expr(subject);
                self.output.push_str(":\n");
                self.indent += 1;
                if cases.is_empty() {
                    self.line("case _:");
                    self.indent += 1;
                    self.line("pass");
                    self.indent -= 1;
                } else {
                    for case in cases {
                        self.push_indent();
                        self.output.push_str("case ");
                        self.emit_pattern(&case.pattern);
                        if let Some(guard) = &case.guard {
                            self.output.push_str(" if ");
                            self.emit_expr(guard);
                        }
                        self.output.push_str(":\n");
                        self.suite(&case.body);
                    }
                }
                self.indent -= 1;
                return;
            }
            Stmt::Return(value, _) => {
                self.output.push_str("return");
                if let Some(value) = value {
                    self.output.push(' ');
                    self.emit_expr(value);
                }
            }
            Stmt::Yield(value, _) => {
                self.output.push_str("yield");
                if let Some(value) = value {
                    self.output.push(' ');
                    self.emit_expr(value);
                }
            }
            Stmt::Raise(value, cause, _) => {
                self.output.push_str("raise");
                if let Some(value) = value {
                    self.output.push(' ');
                    self.emit_expr(value);
                }
                if let Some(cause) = cause {
                    self.output.push_str(" from ");
                    self.emit_expr(cause);
                }
            }
            Stmt::Assert(test, message, _) => {
                self.output.push_str("assert ");
                self.emit_expr(test);
                if let Some(message) = message {
                    self.output.push_str(", ");
                    self.emit_expr(message);
                }
            }
            Stmt::Break(_) => self.output.push_str("break"),
            Stmt::Continue(_) => self.output.push_str("continue"),
            Stmt::Pass(_) | Stmt::Empty(_) => self.output.push_str("pass"),
            Stmt::Delete(expr, _) => {
                self.output.push_str("del ");
                self.emit_expr(expr);
            }
            Stmt::Global(names, _) => {
                self.output.push_str("global ");
                self.output.push_str(&names.join(", "));
            }
            Stmt::Nonlocal(names, _) => {
                self.output.push_str("nonlocal ");
                self.output.push_str(&names.join(", "));
            }
            Stmt::Import(aliases, _) => {
                self.output.push_str("import ");
                self.emit_aliases(aliases);
            }
            Stmt::ImportFrom(module, aliases, level, _) => {
                self.output.push_str("from ");
                self.output.push_str(&".".repeat(*level));
                if let Some(module) = module {
                    self.output.push_str(module);
                }
                self.output.push_str(" import ");
                self.emit_aliases(aliases);
            }
            Stmt::Try(body, handlers, alternate, finalizer, _) => {
                self.output.push_str("try:\n");
                self.suite(body);
                for handler in handlers {
                    self.push_indent();
                    self.output.push_str("except");
                    if let Some(ty) = &handler.type_ {
                        self.output.push(' ');
                        self.emit_expr(ty);
                    }
                    if let Some(name) = &handler.name {
                        self.output.push_str(" as ");
                        self.output.push_str(name);
                    }
                    self.output.push_str(":\n");
                    self.suite(&handler.body);
                }
                if let Some(alternate) = alternate {
                    self.line("else:");
                    self.suite(alternate);
                }
                if let Some(finalizer) = finalizer {
                    self.line("finally:");
                    self.suite(finalizer);
                }
                return;
            }
            Stmt::FuncDef(function, _) => {
                self.emit_function(function);
                return;
            }
            Stmt::ClassDef(class, _) => {
                self.emit_class(class);
                return;
            }
            Stmt::Async(inner, _) => {
                match inner.as_ref() {
                    Stmt::FuncDef(function, _) => self.emit_function_with_prefix(function, true),
                    Stmt::For(target, iter, body, alternate, _) => {
                        self.output.push_str("async for ");
                        self.emit_expr(target);
                        self.output.push_str(" in ");
                        self.emit_expr(iter);
                        self.output.push_str(":\n");
                        self.suite(body);
                        if let Some(alternate) = alternate {
                            self.line("else:");
                            self.suite(alternate);
                        }
                    }
                    Stmt::With(items, body, _) => {
                        self.output.push_str("async with ");
                        for (index, item) in items.iter().enumerate() {
                            if index > 0 {
                                self.output.push_str(", ");
                            }
                            self.emit_expr(&item.context);
                        }
                        self.output.push_str(":\n");
                        self.suite(body);
                    }
                    _ => {
                        self.output.push_str("async def __ripex_async__():\n");
                        self.indent += 1;
                        self.emit_stmt(inner);
                        self.indent -= 1;
                    }
                }
                return;
            }
            Stmt::Block(body, _) => {
                self.output.push_str("if True:\n");
                self.suite(body);
                return;
            }
        }
        self.output.push('\n');
    }

    fn emit_aliases(&mut self, aliases: &[Alias]) {
        for (index, alias) in aliases.iter().enumerate() {
            if index > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&alias.name);
            if let Some(asname) = &alias.asname {
                self.output.push_str(" as ");
                self.output.push_str(asname);
            }
        }
    }

    fn emit_function(&mut self, function: &FuncDef) {
        self.emit_function_with_prefix(function, function.is_async);
    }

    fn emit_function_with_prefix(&mut self, function: &FuncDef, is_async: bool) {
        for (index, decorator) in function.decorators.iter().enumerate() {
            if index > 0 {
                self.push_indent();
            }
            self.output.push('@');
            self.emit_expr(decorator);
            self.output.push('\n');
        }
        if !function.decorators.is_empty() {
            self.push_indent();
        }
        if is_async {
            self.output.push_str("async ");
        }
        self.output.push_str("def ");
        self.output.push_str(&function.name);
        self.output.push('(');
        let default_start = function.args.len().saturating_sub(function.defaults.len());
        for (index, arg) in function.args.iter().enumerate() {
            if index > 0 {
                self.output.push_str(", ");
            }
            self.emit_arg(arg);
            if index >= default_start {
                self.output.push_str(" = ");
                self.emit_expr(&function.defaults[index - default_start]);
            }
        }
        if let Some(vararg) = &function.vararg {
            if !function.args.is_empty() {
                self.output.push_str(", ");
            }
            self.output.push('*');
            self.emit_arg(vararg);
        }
        if let Some(kwarg) = &function.kwarg {
            if !function.args.is_empty() || function.vararg.is_some() {
                self.output.push_str(", ");
            }
            self.output.push_str("**");
            self.emit_arg(kwarg);
        }
        self.output.push(')');
        if let Some(returns) = &function.returns {
            self.output.push_str(" -> ");
            self.emit_expr(returns);
        }
        self.output.push_str(":\n");
        self.suite(&function.body);
    }

    fn emit_arg(&mut self, arg: &Arg) {
        self.output.push_str(&arg.name);
        if let Some(ty) = &arg.type_ann {
            self.output.push_str(": ");
            self.emit_expr(ty);
        }
    }

    fn emit_class(&mut self, class: &ClassDef) {
        for (index, decorator) in class.decorators.iter().enumerate() {
            if index > 0 {
                self.push_indent();
            }
            self.output.push('@');
            self.emit_expr(decorator);
            self.output.push('\n');
        }
        if !class.decorators.is_empty() {
            self.push_indent();
        }
        self.output.push_str("class ");
        self.output.push_str(&class.name);
        if !class.bases.is_empty() || !class.keywords.is_empty() {
            self.output.push('(');
            let mut first = true;
            for base in &class.bases {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.emit_expr(base);
            }
            for keyword in &class.keywords {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                if let Some(name) = &keyword.name {
                    self.output.push_str(name);
                    self.output.push('=');
                } else {
                    self.output.push_str("**");
                }
                self.emit_expr(&keyword.value);
            }
            self.output.push(')');
        }
        self.output.push_str(":\n");
        self.suite(&class.body);
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(literal, _) => self.emit_literal(literal),
            Expr::Ident(name, _) => self.output.push_str(name),
            Expr::Attribute(object, property, _) => {
                self.emit_expr(object);
                self.output.push('.');
                self.output.push_str(property);
            }
            Expr::Subscript(object, index, _) => {
                self.emit_expr(object);
                self.output.push('[');
                self.emit_expr(index);
                self.output.push(']');
            }
            Expr::Slice(lower, upper, step, _) => {
                if let Some(lower) = lower {
                    self.emit_expr(lower);
                }
                self.output.push(':');
                if let Some(upper) = upper {
                    self.emit_expr(upper);
                }
                if let Some(step) = step {
                    self.output.push(':');
                    self.emit_expr(step);
                }
            }
            Expr::Call(function, args, keywords, _) => {
                self.emit_expr(function);
                self.output.push('(');
                let mut first = true;
                for arg in args {
                    if !first {
                        self.output.push_str(", ");
                    }
                    first = false;
                    self.emit_expr(arg);
                }
                for keyword in keywords {
                    if !first {
                        self.output.push_str(", ");
                    }
                    first = false;
                    if let Some(name) = &keyword.name {
                        self.output.push_str(name);
                        self.output.push('=');
                    } else {
                        self.output.push_str("**");
                    }
                    self.emit_expr(&keyword.value);
                }
                self.output.push(')');
            }
            Expr::Binary(left, op, right, _) => {
                self.output.push('(');
                self.emit_expr(left);
                self.output.push(' ');
                self.output.push_str(binary_op(*op));
                self.output.push(' ');
                self.emit_expr(right);
                self.output.push(')');
            }
            Expr::Unary(op, value, _) => {
                self.output.push_str(unary_op(*op));
                self.emit_expr(value);
            }
            Expr::IfElse(test, consequent, alternate, _) => {
                self.output.push('(');
                self.emit_expr(consequent);
                self.output.push_str(" if ");
                self.emit_expr(test);
                self.output.push_str(" else ");
                self.emit_expr(alternate);
                self.output.push(')');
            }
            Expr::Lambda(params, body, _) => {
                self.output.push_str("lambda ");
                self.output.push_str(&params.join(", "));
                self.output.push_str(": ");
                self.emit_expr(body);
            }
            Expr::List(items, _) => self.emit_expr_list("[", "]", items),
            Expr::Tuple(items, _) => {
                self.emit_expr_list("(", ")", items);
                if items.len() == 1 {
                    let len = self.output.len();
                    self.output.insert(len - 1, ',');
                }
            }
            Expr::Dict(items, _) => {
                self.output.push('{');
                for (index, (key, value)) in items.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expr(key);
                    self.output.push_str(": ");
                    self.emit_expr(value);
                }
                self.output.push('}');
            }
            Expr::Set(items, _) => {
                if items.is_empty() {
                    self.output.push_str("set()");
                } else {
                    self.emit_expr_list("{", "}", items);
                }
            }
            Expr::ListComp(element, generators, _) => {
                self.output.push('[');
                self.emit_expr(element);
                self.emit_comprehensions(generators);
                self.output.push(']');
            }
            Expr::SetComp(element, generators, _) => {
                self.output.push('{');
                self.emit_expr(element);
                self.emit_comprehensions(generators);
                self.output.push('}');
            }
            Expr::DictComp(element, generators, _) => {
                self.output.push('{');
                self.emit_expr(element);
                self.output.push_str(": None");
                self.emit_comprehensions(generators);
                self.output.push('}');
            }
            Expr::Generator(element, generators, _) => {
                self.output.push('(');
                self.emit_expr(element);
                self.emit_comprehensions(generators);
                self.output.push(')');
            }
            Expr::Await(value, _) => {
                self.output.push_str("await ");
                self.emit_expr(value);
            }
            Expr::Yield(value, _) => {
                self.output.push_str("yield");
                if let Some(value) = value {
                    self.output.push(' ');
                    self.emit_expr(value);
                }
            }
            Expr::YieldFrom(value, _) => {
                self.output.push_str("yield from ");
                self.emit_expr(value);
            }
            Expr::Starred(value, _) => {
                self.output.push('*');
                self.emit_expr(value);
            }
            Expr::Walrus(target, value, _) => {
                self.output.push('(');
                self.emit_expr(target);
                self.output.push_str(" := ");
                self.emit_expr(value);
                self.output.push(')');
            }
            Expr::FString(parts, _) => {
                self.output.push_str("f\"");
                for part in parts {
                    match part {
                        FStringPart::Text(text, _) => self.output.push_str(&escape_fstring(text)),
                        FStringPart::Expr(value, _) => {
                            self.output.push('{');
                            self.emit_expr(value);
                            self.output.push('}');
                        }
                    }
                }
                self.output.push('"');
            }
            Expr::Compare(left, ops, comparators, _) => {
                self.output.push('(');
                self.emit_expr(left);
                for (op, comparator) in ops.iter().zip(comparators) {
                    self.output.push(' ');
                    self.output.push_str(compare_op(*op));
                    self.output.push(' ');
                    self.emit_expr(comparator);
                }
                self.output.push(')');
            }
            Expr::Paren(value, _) => {
                self.output.push('(');
                self.emit_expr(value);
                self.output.push(')');
            }
            Expr::Ellipsis(_) => self.output.push_str("..."),
            Expr::Match(subject, _, _) => self.emit_expr(subject),
            Expr::Error(_) => self.output.push_str("None"),
        }
    }

    fn emit_literal(&mut self, literal: &Literal) {
        match literal {
            Literal::Int(value, raw, _) => {
                if raw.is_empty() {
                    self.output.push_str(&value.to_string());
                } else {
                    self.output.push_str(raw);
                }
            }
            Literal::Float(value, raw, _) => {
                if raw.is_empty() {
                    self.output.push_str(&value.to_string());
                } else {
                    self.output.push_str(raw);
                }
            }
            Literal::Complex { text, imag, .. } => {
                if text.is_empty() {
                    self.output.push_str(&format!("{imag}j"));
                } else {
                    self.output.push_str(text);
                }
            }
            Literal::String(value, _, _) => self.output.push_str(&format!("{value:?}")),
            Literal::Bytes(value, _, _) => {
                self.output.push('b');
                self.output
                    .push_str(&format!("{:?}", String::from_utf8_lossy(value)));
            }
            Literal::Boolean(value, _) => {
                self.output.push_str(if *value { "True" } else { "False" })
            }
            Literal::None_(_) => self.output.push_str("None"),
            Literal::Ellipsis(_) => self.output.push_str("..."),
        }
    }

    fn emit_expr_list(&mut self, open: &str, close: &str, items: &[Expr]) {
        self.output.push_str(open);
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                self.output.push_str(", ");
            }
            self.emit_expr(item);
        }
        self.output.push_str(close);
    }

    fn emit_comprehensions(&mut self, generators: &[Comprehension]) {
        for generator in generators {
            self.output.push(' ');
            if generator.is_async {
                self.output.push_str("async ");
            }
            self.output.push_str("for ");
            self.emit_expr(&generator.target);
            self.output.push_str(" in ");
            self.emit_expr(&generator.iter);
            for condition in &generator.ifs {
                self.output.push_str(" if ");
                self.emit_expr(condition);
            }
        }
    }

    fn emit_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard(_) => self.output.push('_'),
            Pattern::Value(value, _) | Pattern::Capture(value, _) => self.output.push_str(value),
            Pattern::Literal(value, _) => self.emit_expr(value),
            Pattern::Sequence(items, _) => {
                self.output.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_pattern(item);
                }
                self.output.push(']');
            }
            Pattern::Mapping(items, rest, _) => {
                self.output.push('{');
                for (index, (key, value)) in items.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_pattern(key);
                    self.output.push_str(": ");
                    self.emit_pattern(value);
                }
                if let Some(rest) = rest {
                    if !items.is_empty() {
                        self.output.push_str(", ");
                    }
                    self.output.push_str("**");
                    self.emit_pattern(rest);
                }
                self.output.push('}');
            }
            Pattern::Class(name, positional, keywords, _) => {
                self.output.push_str(name);
                self.output.push('(');
                let mut first = true;
                for item in positional {
                    if !first {
                        self.output.push_str(", ");
                    }
                    first = false;
                    self.emit_pattern(item);
                }
                for (name, item) in keywords {
                    if !first {
                        self.output.push_str(", ");
                    }
                    first = false;
                    self.output.push_str(name);
                    self.output.push('=');
                    self.emit_pattern(item);
                }
                self.output.push(')');
            }
            Pattern::Or(items, _) => {
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(" | ");
                    }
                    self.emit_pattern(item);
                }
            }
            Pattern::As(inner, name, _) => {
                self.emit_pattern(inner);
                self.output.push_str(" as ");
                self.output.push_str(name);
            }
            Pattern::Guard(inner, guard, _) => {
                self.emit_pattern(inner);
                self.output.push_str(" if ");
                self.emit_expr(guard);
            }
            Pattern::Group(inner, _) => {
                self.output.push('(');
                self.emit_pattern(inner);
                self.output.push(')');
            }
        }
    }
}

fn binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::FloorDiv => "//",
        BinaryOp::Mod => "%",
        BinaryOp::Pow => "**",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::Shl => "<<",
        BinaryOp::Shr => ">>",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::MatMult => "@",
    }
}

fn unary_op(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Pos => "+",
        UnaryOp::Not => "not ",
        UnaryOp::Invert => "~",
    }
}

fn compare_op(op: CmpOp) -> &'static str {
    match op {
        CmpOp::Eq => "==",
        CmpOp::Ne => "!=",
        CmpOp::Lt => "<",
        CmpOp::Gt => ">",
        CmpOp::Le => "<=",
        CmpOp::Ge => ">=",
        CmpOp::Is => "is",
        CmpOp::IsNot => "is not",
        CmpOp::In => "in",
        CmpOp::NotIn => "not in",
    }
}

fn escape_fstring(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('{', "{{")
        .replace('}', "}}")
        .replace('\n', "\\n")
}
