use crate::csharp::ast::expr::{Expr, LambdaBody, LambdaExpr};
use crate::csharp::ast::stmt::*;
use crate::csharp::ast::Program;
use crate::facts as f;
use crate::ExtractionResult;
use crate::{
    CallKind, ImportKind, ParsedCall, ParsedImport, ParsedSymbol, ParsedVariable, StorageClass,
    SymbolKind, TypeKind, VarKind,
};

/// C# test-method heuristic. The AST does not retain `[Fact]`/`[TestMethod]`
/// attributes, so we use the naming conventions common in xUnit/NUnit/MSTest:
/// methods named `Test*`, `Should*`, or the BDD-style `Given_*`/`When_*`/`Then_*`.
fn is_csharp_test_fn(name: &str) -> bool {
    name.starts_with("Test")
        || name.starts_with("Should")
        || name.starts_with("Given_")
        || name.starts_with("When_")
        || name.starts_with("Then_")
}

fn map_vis(v: &crate::csharp::ast::stmt::Visibility) -> f::Visibility {
    use crate::csharp::ast::stmt::Visibility::*;
    match v {
        Public => f::Visibility::Public,
        Private => f::Visibility::Private,
        Protected => f::Visibility::Protected,
        Internal => f::Visibility::Internal,
        _ => f::Visibility::Unknown,
    }
}

fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name, _) => name.clone(),
        Expr::Member(obj, name, _) => format!("{}.{}", expr_to_string(obj), name),
        Expr::Call(callee, args, _) => {
            let callee_str = expr_to_string(callee);
            let args_str: Vec<String> = args.iter().map(expr_to_string).collect();
            format!("{}({})", callee_str, args_str.join(", "))
        }
        Expr::String(s, _) => format!("\"{}\"", s),
        Expr::Int(i, _) => i.to_string(),
        Expr::UInt(u, _) => u.to_string(),
        Expr::Long(i, _) => i.to_string(),
        Expr::ULong(u, _) => u.to_string(),
        Expr::Float(f, _) => f.to_string(),
        Expr::Double(f, _) => f.to_string(),
        Expr::Decimal(f, _) => f.to_string(),
        Expr::Bool(b, _) => b.to_string(),
        Expr::Char(c, _) => format!("'{}'", c),
        Expr::Null(_) => "null".to_string(),
        Expr::Paren(inner, _) => format!("({})", expr_to_string(inner)),
        Expr::Array(items, _) => {
            let items_str: Vec<String> = items.iter().map(expr_to_string).collect();
            format!("[{}]", items_str.join(", "))
        }
        Expr::New(ty, args, _) => {
            let args_str: Vec<String> = args.iter().map(expr_to_string).collect();
            format!("new {}({})", expr_to_string(ty), args_str.join(", "))
        }
        _ => "?".to_string(),
    }
}

fn expr_to_type(expr: &Expr) -> TypeKind {
    match expr {
        Expr::Ident(name, _) => TypeKind::Simple(name.clone()),
        Expr::Member(obj, name, _) => match expr_to_type(obj) {
            TypeKind::Simple(s) => TypeKind::Simple(format!("{}.{}", s, name)),
            _ => TypeKind::Simple(name.clone()),
        },
        _ => TypeKind::Unknown,
    }
}

fn make_signature(name: &str, params: &[ParamDecl]) -> String {
    let types: Vec<String> = params.iter().map(|p| expr_to_string(&p.type_)).collect();
    format!("{}({})", name, types.join(", "))
}

pub fn extract_facts(program: &Program) -> ExtractionResult {
    ExtractionResult {
        symbols: extract_symbols(program),
        imports: extract_imports(program),
        calls: extract_calls(program),
        variables: extract_variables(program),
    }
}

pub fn extract_symbols(program: &Program) -> Vec<ParsedSymbol> {
    let mut symbols = Vec::new();
    walk_decls(&program.decls, &mut symbols, &[]);
    symbols
}

fn walk_decls(decls: &[Decl], symbols: &mut Vec<ParsedSymbol>, parent_path: &[String]) {
    for decl in decls {
        match decl {
            Decl::Namespace(name, nested, span) => {
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Namespace, name)
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
                let mut path = parent_path.to_vec();
                path.push(name.clone());
                walk_decls(nested, symbols, &path);
            }
            Decl::Class(cls, span) => {
                let full_name = if parent_path.is_empty() {
                    cls.name.clone()
                } else {
                    format!("{}.{}", parent_path.join("."), cls.name)
                };
                let mut base_classes = Vec::new();
                if let Some(base) = &cls.base {
                    base_classes.push(expr_to_string(base));
                }
                for iface in &cls.interfaces {
                    base_classes.push(expr_to_string(iface));
                }
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Class, &full_name)
                        .visibility(map_vis(&cls.visibility))
                        .abstract_(cls.is_abstract)
                        .static_(cls.is_static)
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
                let mut path = parent_path.to_vec();
                path.push(cls.name.clone());
                walk_decls(&cls.members, symbols, &path);
            }
            Decl::Struct(sd, span) => {
                let full_name = if parent_path.is_empty() {
                    sd.name.clone()
                } else {
                    format!("{}.{}", parent_path.join("."), sd.name)
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Struct, &full_name)
                        .visibility(map_vis(&sd.visibility))
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
                let mut path = parent_path.to_vec();
                path.push(sd.name.clone());
                walk_decls(&sd.members, symbols, &path);
            }
            Decl::Interface(id, span) => {
                let full_name = if parent_path.is_empty() {
                    id.name.clone()
                } else {
                    format!("{}.{}", parent_path.join("."), id.name)
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Interface, &full_name)
                        .visibility(map_vis(&id.visibility))
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
                let mut path = parent_path.to_vec();
                path.push(id.name.clone());
                walk_decls(&id.members, symbols, &path);
            }
            Decl::Record(cls, span) => {
                let full_name = if parent_path.is_empty() {
                    cls.name.clone()
                } else {
                    format!("{}.{}", parent_path.join("."), cls.name)
                };
                let mut base_classes = Vec::new();
                if let Some(base) = &cls.base {
                    base_classes.push(expr_to_string(base));
                }
                for iface in &cls.interfaces {
                    base_classes.push(expr_to_string(iface));
                }
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Class, &full_name)
                        .visibility(map_vis(&cls.visibility))
                        .abstract_(cls.is_abstract)
                        .static_(cls.is_static)
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
                let mut path = parent_path.to_vec();
                path.push(cls.name.clone());
                walk_decls(&cls.members, symbols, &path);
            }
            Decl::Enum(ed, span) => {
                let full_name = if parent_path.is_empty() {
                    ed.name.clone()
                } else {
                    format!("{}.{}", parent_path.join("."), ed.name)
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Enum, &full_name)
                        .visibility(map_vis(&ed.visibility))
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
            }
            Decl::Delegate(dd, span) => {
                let full_name = if parent_path.is_empty() {
                    dd.name.clone()
                } else {
                    format!("{}.{}", parent_path.join("."), dd.name)
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Delegate, &full_name)
                        .visibility(map_vis(&dd.visibility))
                        .return_type(Some(expr_to_string(&dd.return_type)))
                        .signature(make_signature(&full_name, &dd.params))
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
                for param in &dd.params {
                    symbols.push(
                        ParsedSymbol::builder(SymbolKind::Variable, &param.name)
                            .lines(span.start.line, span.end.line)
                            .build(),
                    );
                }
            }
            Decl::Event(ed, span) => {
                let member_name = if let Some(parent) = parent_path.last() {
                    format!("{}.{}", parent, ed.name)
                } else {
                    ed.name.clone()
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Event, &member_name)
                        .visibility(map_vis(&ed.visibility))
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
            }
            Decl::Property(pd, span) => {
                let member_name = if let Some(parent) = parent_path.last() {
                    format!("{}.{}", parent, pd.name)
                } else {
                    pd.name.clone()
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Property, &member_name)
                        .visibility(map_vis(&pd.visibility))
                        .return_type(Some(expr_to_string(&pd.type_)))
                        .type_kind(expr_to_type(&pd.type_))
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
            }
            Decl::Method(fd, span) => {
                let member_name = if let Some(parent) = parent_path.last() {
                    format!("{}.{}", parent, fd.name)
                } else {
                    fd.name.clone()
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Method, &member_name)
                        .visibility(map_vis(&fd.visibility))
                        .static_(fd.is_static)
                        .virtual_(fd.is_virtual)
                        .override_(fd.is_override)
                        .abstract_(fd.is_abstract)
                        .return_type(Some(expr_to_string(&fd.return_type)))
                        .signature(make_signature(&member_name, &fd.params))
                        .type_kind(expr_to_type(&fd.return_type))
                        .lines(span.start.line, span.end.line)
                        .is_test(is_csharp_test_fn(&fd.name))
                        .build(),
                );
                for param in &fd.params {
                    symbols.push(
                        ParsedSymbol::builder(SymbolKind::Variable, &param.name)
                            .visibility(map_vis(&fd.visibility))
                            .lines(span.start.line, span.end.line)
                            .type_kind(expr_to_type(&param.type_))
                            .build(),
                    );
                }
            }
            Decl::Constructor(cd, span) => {
                let class_name = parent_path.last().cloned().unwrap_or_default();
                let member_name = if class_name.is_empty() {
                    class_name
                } else {
                    format!(
                        "{}.{}",
                        parent_path[..parent_path.len() - 1].join("."),
                        class_name
                    )
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Constructor, &member_name)
                        .visibility(map_vis(&cd.visibility))
                        .constructor(true)
                        .static_(cd.is_static)
                        .signature(make_signature(&member_name, &cd.params))
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
                for param in &cd.params {
                    symbols.push(
                        ParsedSymbol::builder(SymbolKind::Variable, &param.name)
                            .lines(span.start.line, span.end.line)
                            .build(),
                    );
                }
            }
            Decl::Destructor(_, span) => {
                let class_name = parent_path.last().cloned().unwrap_or_default();
                let member_name = format!("~{}", class_name);
                let full_name = if parent_path.len() > 1 {
                    format!(
                        "{}.{}",
                        parent_path[..parent_path.len() - 1].join("."),
                        member_name
                    )
                } else {
                    member_name
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Destructor, &full_name)
                        .destructor(true)
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
            }
            Decl::Operator(op, span) => {
                let member_name = if let Some(parent) = parent_path.last() {
                    format!("{}.operator {}", parent, op.op)
                } else {
                    format!("operator {}", op.op)
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Method, &member_name)
                        .return_type(Some(expr_to_string(&op.return_type)))
                        .signature(make_signature(&member_name, &op.params))
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
            }
            Decl::Conversion(cd, span) => {
                let conv_kind = if cd.is_explicit {
                    "explicit"
                } else {
                    "implicit"
                };
                let member_name = if let Some(parent) = parent_path.last() {
                    format!("{}.operator {}", parent, conv_kind)
                } else {
                    format!("operator {}", conv_kind)
                };
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Method, &member_name)
                        .return_type(Some(expr_to_string(&cd.return_type)))
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
            }
            Decl::Field(..) | Decl::Using(..) | Decl::UsingStatic(..) | Decl::ExternAlias(..) => {}
        }
    }
}

pub fn extract_imports(program: &Program) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    walk_imports(&program.decls, &mut imports);
    imports
}

fn walk_imports(decls: &[Decl], imports: &mut Vec<ParsedImport>) {
    for decl in decls {
        match decl {
            Decl::Using(ud, span) => {
                let mut import = ParsedImport::builder(ImportKind::NamedImport, &ud.namespace)
                    .line(span.start.line);
                if let Some(alias) = &ud.alias {
                    import = import.local(alias);
                }
                imports.push(import.build());
            }
            Decl::UsingStatic(path, span) => {
                imports.push(
                    ParsedImport::builder(ImportKind::NamespaceImport, path)
                        .line(span.start.line)
                        .build(),
                );
            }
            Decl::Namespace(_, nested, _) => walk_imports(nested, imports),
            _ => {}
        }
    }
}

pub fn extract_calls(program: &Program) -> Vec<ParsedCall> {
    let mut calls = Vec::new();
    walk_decls_for_calls(&program.decls, &mut calls);
    calls
}

fn walk_decls_for_calls(decls: &[Decl], calls: &mut Vec<ParsedCall>) {
    for decl in decls {
        match decl {
            Decl::Namespace(_, nested, _) => walk_decls_for_calls(nested, calls),
            Decl::Class(cls, _) => walk_decls_for_calls(&cls.members, calls),
            Decl::Struct(sd, _) => walk_decls_for_calls(&sd.members, calls),
            Decl::Interface(id, _) => walk_decls_for_calls(&id.members, calls),
            Decl::Record(cls, _) => walk_decls_for_calls(&cls.members, calls),
            Decl::Method(fd, _) => {
                if let Some(body) = &fd.body {
                    walk_block_for_calls(body, calls);
                }
            }
            Decl::Constructor(cd, _) => {
                if let Some(body) = &cd.body {
                    walk_block_for_calls(body, calls);
                }
            }
            Decl::Destructor(dd, _) => {
                if let Some(body) = &dd.body {
                    walk_block_for_calls(body, calls);
                }
            }
            Decl::Property(pd, _) => {
                if let Some(getter) = &pd.getter {
                    walk_stmt_for_calls(getter, calls);
                }
                if let Some(setter) = &pd.setter {
                    walk_stmt_for_calls(setter, calls);
                }
            }
            Decl::Field(fd, _) => {
                if let Some(init) = &fd.init {
                    walk_expr_for_calls(init, calls);
                }
            }
            Decl::Operator(op, _) => {
                if let Some(body) = &op.body {
                    walk_block_for_calls(body, calls);
                }
            }
            Decl::Conversion(cd, _) => {
                if let Some(body) = &cd.body {
                    walk_block_for_calls(body, calls);
                }
            }
            Decl::Enum(ed, _) => {
                for member in &ed.members {
                    if let Some(value) = &member.value {
                        walk_expr_for_calls(value, calls);
                    }
                }
            }
            Decl::Delegate(..)
            | Decl::Event(..)
            | Decl::Using(..)
            | Decl::UsingStatic(..)
            | Decl::ExternAlias(..) => {}
        }
    }
}

fn walk_block_for_calls(block: &Block, calls: &mut Vec<ParsedCall>) {
    for stmt in &block.stmts {
        walk_stmt_for_calls(stmt, calls);
    }
}

fn walk_stmt_for_calls(stmt: &Stmt, calls: &mut Vec<ParsedCall>) {
    match stmt {
        Stmt::Expr(expr, _) => walk_expr_for_calls(expr, calls),
        Stmt::Decl(decl, _) => walk_single_decl_for_calls(decl, calls),
        Stmt::If(cond, then, else_, _) => {
            walk_expr_for_calls(cond, calls);
            walk_stmt_for_calls(then, calls);
            if let Some(else_stmt) = else_ {
                walk_stmt_for_calls(else_stmt, calls);
            }
        }
        Stmt::Switch(expr, cases, _) => {
            walk_expr_for_calls(expr, calls);
            for case in cases {
                for stmt in &case.stmts {
                    walk_stmt_for_calls(stmt, calls);
                }
            }
        }
        Stmt::While(cond, body, _) => {
            walk_expr_for_calls(cond, calls);
            walk_stmt_for_calls(body, calls);
        }
        Stmt::Do(body, cond, _) => {
            walk_stmt_for_calls(body, calls);
            walk_expr_for_calls(cond, calls);
        }
        Stmt::For(init, cond, update, body, _) => {
            if let Some(init) = init {
                walk_stmt_for_calls(init, calls);
            }
            if let Some(cond) = cond {
                walk_expr_for_calls(cond, calls);
            }
            if let Some(update) = update {
                walk_stmt_for_calls(update, calls);
            }
            walk_stmt_for_calls(body, calls);
        }
        Stmt::Foreach(_, expr, body, _) => {
            walk_expr_for_calls(expr, calls);
            walk_stmt_for_calls(body, calls);
        }
        Stmt::Return(expr, _) => {
            if let Some(expr) = expr {
                walk_expr_for_calls(expr, calls);
            }
        }
        Stmt::YieldReturn(expr, _) => walk_expr_for_calls(expr, calls),
        Stmt::Throw(expr, _) => {
            if let Some(expr) = expr {
                walk_expr_for_calls(expr, calls);
            }
        }
        Stmt::YieldBreak(_)
        | Stmt::Break(_)
        | Stmt::Continue(_)
        | Stmt::Empty(_)
        | Stmt::Goto(..)
        | Stmt::GotoDefault(_) => {}
        Stmt::GotoCase(expr, _) => walk_expr_for_calls(expr, calls),
        Stmt::Try(body, catches, finally, _) => {
            walk_stmt_for_calls(body, calls);
            for c in catches {
                walk_stmt_for_calls(&c.body, calls);
            }
            if let Some(finally) = finally {
                walk_stmt_for_calls(finally, calls);
            }
        }
        Stmt::Checked(body, _) | Stmt::Unchecked(body, _) | Stmt::Unsafe(body, _) => {
            walk_stmt_for_calls(body, calls);
        }
        Stmt::Lock(expr, body, _) | Stmt::Using(expr, body, _) | Stmt::Fixed(expr, body, _) => {
            walk_expr_for_calls(expr, calls);
            walk_stmt_for_calls(body, calls);
        }
        Stmt::Block(block, _) => walk_block_for_calls(block, calls),
        Stmt::Label(_, _) => {}
        Stmt::LocalFunc(fd, _) => {
            if let Some(body) = &fd.body {
                walk_block_for_calls(body, calls);
            }
        }
    }
}

fn walk_single_decl_for_calls(decl: &Decl, calls: &mut Vec<ParsedCall>) {
    match decl {
        Decl::Method(fd, _) => {
            if let Some(body) = &fd.body {
                walk_block_for_calls(body, calls);
            }
        }
        Decl::Constructor(cd, _) => {
            if let Some(body) = &cd.body {
                walk_block_for_calls(body, calls);
            }
        }
        Decl::Destructor(dd, _) => {
            if let Some(body) = &dd.body {
                walk_block_for_calls(body, calls);
            }
        }
        Decl::Property(pd, _) => {
            if let Some(getter) = &pd.getter {
                walk_stmt_for_calls(getter, calls);
            }
            if let Some(setter) = &pd.setter {
                walk_stmt_for_calls(setter, calls);
            }
        }
        Decl::Field(fd, _) => {
            if let Some(init) = &fd.init {
                walk_expr_for_calls(init, calls);
            }
        }
        Decl::Operator(op, _) => {
            if let Some(body) = &op.body {
                walk_block_for_calls(body, calls);
            }
        }
        Decl::Conversion(cd, _) => {
            if let Some(body) = &cd.body {
                walk_block_for_calls(body, calls);
            }
        }
        _ => {}
    }
}

fn walk_expr_for_calls(expr: &Expr, calls: &mut Vec<ParsedCall>) {
    match expr {
        Expr::Call(callee, args, span) => {
            let (callee_text, object) = match callee.as_ref() {
                Expr::Ident(name, _) => (name.clone(), None),
                Expr::Member(obj, name, _) => (name.clone(), Some(expr_to_string(obj))),
                _ => (expr_to_string(callee), None),
            };
            let kind = if object.is_some() {
                CallKind::MethodCall
            } else {
                CallKind::FunctionCall
            };
            let mut builder =
                ParsedCall::builder(kind, &callee_text).pos(span.start.line, span.start.column);
            if let Some(obj) = &object {
                builder = builder.object(obj);
            }
            calls.push(builder.build());
            walk_expr_for_calls(callee, calls);
            for arg in args {
                walk_expr_for_calls(arg, calls);
            }
        }
        Expr::New(ty, args, span) => {
            let type_name = expr_to_string(ty);
            calls.push(
                ParsedCall::builder(CallKind::ConstructorCall, &type_name)
                    .pos(span.start.line, span.start.column)
                    .build(),
            );
            walk_expr_for_calls(ty, calls);
            for arg in args {
                walk_expr_for_calls(arg, calls);
            }
        }
        Expr::Binary(l, _, r, _) => {
            walk_expr_for_calls(l, calls);
            walk_expr_for_calls(r, calls);
        }
        Expr::Unary(_, e, _) => walk_expr_for_calls(e, calls),
        Expr::Index(obj, idx, _) => {
            walk_expr_for_calls(obj, calls);
            walk_expr_for_calls(idx, calls);
        }
        Expr::Member(obj, _, _) => walk_expr_for_calls(obj, calls),
        Expr::NullConditional(obj, _, _) => walk_expr_for_calls(obj, calls),
        Expr::Conditional(c, t, f, _) => {
            walk_expr_for_calls(c, calls);
            walk_expr_for_calls(t, calls);
            walk_expr_for_calls(f, calls);
        }
        Expr::NullCoalesce(l, r, _) => {
            walk_expr_for_calls(l, calls);
            walk_expr_for_calls(r, calls);
        }
        Expr::Paren(inner, _) => walk_expr_for_calls(inner, calls),
        Expr::Assign(l, r, _) => {
            walk_expr_for_calls(l, calls);
            walk_expr_for_calls(r, calls);
        }
        Expr::IsPattern(v, p, _) => {
            walk_expr_for_calls(v, calls);
            walk_expr_for_calls(p, calls);
        }
        Expr::Await(inner, _) => walk_expr_for_calls(inner, calls),
        Expr::Throw(inner, _) => walk_expr_for_calls(inner, calls),
        Expr::Lambda(lam, _) => walk_lambda_for_calls(lam, calls),
        Expr::AnonymousMethod(_, block, _) => {
            for stmt in &block.stmts {
                walk_stmt_for_calls(stmt, calls);
            }
        }
        Expr::ObjectInit(_, inits, _) => {
            for init in inits {
                walk_expr_for_calls(&init.value, calls);
            }
        }
        Expr::CollectionInit(items, _) | Expr::Array(items, _) => {
            for item in items {
                walk_expr_for_calls(item, calls);
            }
        }
        Expr::SwitchExpr(_, arms, _) => {
            for arm in arms {
                walk_expr_for_calls(&arm.pattern, calls);
                walk_expr_for_calls(&arm.value, calls);
            }
        }
        _ => {}
    }
}

fn walk_lambda_for_calls(lam: &LambdaExpr, calls: &mut Vec<ParsedCall>) {
    match &lam.body {
        LambdaBody::Expr(expr) => walk_expr_for_calls(expr, calls),
        LambdaBody::Block(block) => {
            for stmt in &block.stmts {
                walk_stmt_for_calls(stmt, calls);
            }
        }
    }
}

pub fn extract_variables(program: &Program) -> Vec<ParsedVariable> {
    let mut variables = Vec::new();
    walk_variables(&program.decls, &mut variables);
    variables
}

fn walk_variables(decls: &[Decl], variables: &mut Vec<ParsedVariable>) {
    for decl in decls {
        match decl {
            Decl::Field(fd, _) => {
                variables.push(
                    ParsedVariable::builder(&fd.name, VarKind::Field)
                        .type_ann(Some(expr_to_string(&fd.type_)))
                        .type_kind(expr_to_type(&fd.type_))
                        .storage(if fd.is_static {
                            StorageClass::Static
                        } else {
                            StorageClass::Unknown
                        })
                        .line(fd.span.start.line)
                        .build(),
                );
            }
            Decl::Property(pd, _) => {
                variables.push(
                    ParsedVariable::builder(&pd.name, VarKind::Property)
                        .type_ann(Some(expr_to_string(&pd.type_)))
                        .type_kind(expr_to_type(&pd.type_))
                        .line(pd.span.start.line)
                        .build(),
                );
            }
            Decl::Enum(ed, _) => {
                for member in &ed.members {
                    variables.push(
                        ParsedVariable::builder(&member.name, VarKind::EnumMember)
                            .line(member.span.start.line)
                            .build(),
                    );
                }
            }
            Decl::Namespace(_, _, _)
            | Decl::Class(_, _)
            | Decl::Struct(_, _)
            | Decl::Interface(_, _)
            | Decl::Record(_, _) => {
                let inner = get_members(decl);
                walk_variables(inner, variables);
            }
            // Method-like declarations contribute parameters and body locals.
            Decl::Method(_, span) | Decl::Constructor(_, span) | Decl::Operator(_, span) => {
                let params = match decl {
                    Decl::Method(fd, _) => &fd.params,
                    Decl::Constructor(cd, _) => &cd.params,
                    Decl::Operator(op, _) => &op.params,
                    _ => unreachable!(),
                };
                for param in params {
                    variables.push(
                        ParsedVariable::builder(&param.name, VarKind::Parameter)
                            .type_ann(Some(expr_to_string(&param.type_)))
                            .type_kind(expr_to_type(&param.type_))
                            .line(span.start.line)
                            .build(),
                    );
                }
                let body = match decl {
                    Decl::Method(fd, _) => &fd.body,
                    Decl::Constructor(cd, _) => &cd.body,
                    Decl::Operator(op, _) => &op.body,
                    _ => unreachable!(),
                };
                if let Some(body) = body {
                    walk_block_for_variables(body, variables);
                }
            }
            Decl::Conversion(cd, span) => {
                let param = &cd.param;
                variables.push(
                    ParsedVariable::builder(&param.name, VarKind::Parameter)
                        .type_ann(Some(expr_to_string(&param.type_)))
                        .type_kind(expr_to_type(&param.type_))
                        .line(span.start.line)
                        .build(),
                );
                if let Some(body) = &cd.body {
                    walk_block_for_variables(body, variables);
                }
            }
            _ => {}
        }
    }
}

fn walk_block_for_variables(block: &Block, variables: &mut Vec<ParsedVariable>) {
    for stmt in &block.stmts {
        walk_stmt_for_variables(stmt, variables);
    }
}

fn walk_stmt_for_variables(stmt: &Stmt, variables: &mut Vec<ParsedVariable>) {
    match stmt {
        // Local variables inside a method body are stored by the parser as
        // `Decl::Field` (is_static = false). Emit them so find-references
        // works inside function bodies.
        Stmt::Decl(Decl::Field(f, _), _) if !f.name.is_empty() => {
            variables.push(
                ParsedVariable::builder(&f.name, VarKind::Var)
                    .type_ann(Some(expr_to_string(&f.type_)))
                    .type_kind(expr_to_type(&f.type_))
                    .line(f.span.start.line)
                    .build(),
            );
        }
        Stmt::Block(block, _) => walk_block_for_variables(block, variables),
        Stmt::If(_, then, else_, _) => {
            walk_stmt_for_variables(then, variables);
            if let Some(e) = else_ {
                walk_stmt_for_variables(e, variables);
            }
        }
        Stmt::While(_, body, _) | Stmt::Do(body, _, _) => walk_stmt_for_variables(body, variables),
        Stmt::For(init, _, _, body, _) => {
            if let Some(i) = init {
                walk_stmt_for_variables(i, variables);
            }
            walk_stmt_for_variables(body, variables);
        }
        Stmt::Foreach(_, _, body, _) => walk_stmt_for_variables(body, variables),
        Stmt::Try(body, catches, finally, _) => {
            walk_stmt_for_variables(body, variables);
            for c in catches {
                walk_stmt_for_variables(&c.body, variables);
            }
            if let Some(f) = finally {
                walk_stmt_for_variables(f, variables);
            }
        }
        Stmt::Checked(body, _)
        | Stmt::Unchecked(body, _)
        | Stmt::Unsafe(body, _)
        | Stmt::Lock(_, body, _)
        | Stmt::Using(_, body, _)
        | Stmt::Fixed(_, body, _) => walk_stmt_for_variables(body, variables),
        Stmt::LocalFunc(fd, _) => {
            if let Some(body) = &fd.body {
                walk_block_for_variables(body, variables);
            }
        }
        _ => {}
    }
}

fn get_members(decl: &Decl) -> &[Decl] {
    match decl {
        Decl::Namespace(_, nested, _) => nested,
        Decl::Class(cls, _) => &cls.members,
        Decl::Struct(sd, _) => &sd.members,
        Decl::Interface(id, _) => &id.members,
        Decl::Record(cls, _) => &cls.members,
        _ => &[],
    }
}
