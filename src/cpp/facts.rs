use crate::cpp::ast::*;
use crate::facts::*;
use crate::ExtractionResult;

/// C++ test-function heuristic. gtest `TEST(Suite, Name)` and `TEST_F` expand
/// to functions whose mangled names are not visible here, but Catch2/CppUnit
/// and hand-written tests conventionally name functions `test_*` / `Test_*`.
fn is_cpp_test_fn(name: &str) -> bool {
    name.starts_with("test_") || name.starts_with("Test_") || name.starts_with("TEST_")
}

pub fn extract_facts(program: &Program) -> ExtractionResult {
    let mut result = ExtractionResult::new();
    for decl in &program.decls {
        walk_decl(decl, &mut result, None);
    }
    result
}

pub fn extract_symbols(program: &Program) -> Vec<ParsedSymbol> {
    extract_facts(program).symbols
}

pub fn extract_imports(program: &Program) -> Vec<ParsedImport> {
    extract_facts(program).imports
}

pub fn extract_calls(program: &Program) -> Vec<ParsedCall> {
    extract_facts(program).calls
}

pub fn extract_variables(program: &Program) -> Vec<ParsedVariable> {
    extract_facts(program).variables
}

fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Int(v, _) => v.to_string(),
        Expr::UInt(v, _) => v.to_string(),
        Expr::Float(v, _) => v.to_string(),
        Expr::String(s, _) => format!("\"{}\"", s),
        Expr::Char(c, _) => format!("'{}'", c),
        Expr::Bool(b, _) => b.to_string(),
        Expr::NullPtr(_) => "nullptr".to_string(),
        Expr::Ident(s, _) => s.clone(),
        Expr::Call(callee, _, _) => expr_to_string(callee),
        Expr::Index(obj, _, _) => expr_to_string(obj),
        Expr::Member(obj, name, _) => format!("{}.{}", expr_to_string(obj), name),
        Expr::Arrow(obj, name, _) => format!("{}->{}", expr_to_string(obj), name),
        Expr::Deref(e, _) => format!("*{}", expr_to_string(e)),
        Expr::Ref(e, _) => format!("&{}", expr_to_string(e)),
        Expr::Paren(e, _) => format!("({})", expr_to_string(e)),
        Expr::This(_) => "this".to_string(),
        _ => "_".to_string(),
    }
}

fn expr_to_type_kind(expr: &Expr) -> TypeKind {
    match expr {
        Expr::Ident(s, _) => TypeKind::Simple(s.clone()),
        _ => TypeKind::Simple(expr_to_string(expr)),
    }
}

fn template_param_to_string(param: &TemplateParam) -> String {
    match param {
        TemplateParam::Type(name, _) => format!("typename {}", name),
        TemplateParam::Value(_, name, _, _) => name.clone(),
        TemplateParam::Template(name, _) => format!("template<...> {}", name),
    }
}

fn process_func_decl(
    fd: &FuncDecl,
    span: &crate::span::Span,
    result: &mut ExtractionResult,
    class_name: Option<&str>,
    template_params: &[String],
) {
    let destructor = fd.name.starts_with('~');
    let constructor = !destructor && class_name.is_some_and(|cn| fd.name == cn);
    let kind = if destructor {
        SymbolKind::Destructor
    } else if constructor {
        SymbolKind::Constructor
    } else {
        SymbolKind::Function
    };
    let name = if destructor {
        fd.name.trim_start_matches('~').to_string()
    } else {
        fd.name.clone()
    };
    let return_type = if constructor || destructor {
        None
    } else {
        Some(expr_to_string(&fd.return_type))
    };
    let storage = if fd.is_static {
        StorageClass::Static
    } else {
        StorageClass::Unknown
    };
    let mut sym = ParsedSymbol::builder(kind, &name)
        .lines(span.start.line, span.end.line)
        .return_type(return_type)
        .storage(storage)
        .virtual_(fd.is_virtual)
        .override_(fd.is_override)
        .static_(fd.is_static)
        .is_test(is_cpp_test_fn(&name))
        .build();
    sym.is_constexpr = fd.is_constexpr;
    sym.template_params = template_params.to_vec();
    if destructor {
        sym.is_destructor = true;
    }
    if constructor {
        sym.is_constructor = true;
    }
    result.symbols.push(sym);
}

fn walk_decl(decl: &Decl, result: &mut ExtractionResult, class_name: Option<&str>) {
    match decl {
        Decl::Func(fd, span) => {
            process_func_decl(fd, span, result, class_name, &[]);

            for param in &fd.params {
                if let Some(name) = &param.name {
                    let type_str = expr_to_string(&param.type_);
                    let type_kind = expr_to_type_kind(&param.type_);
                    result.variables.push(
                        ParsedVariable::builder(name, VarKind::Parameter)
                            .type_ann(Some(type_str))
                            .type_kind(type_kind)
                            .line(span.start.line)
                            .build(),
                    );
                }
            }

            if let Some(body) = &fd.body {
                walk_block(body, result, class_name);
            }
        }
        Decl::Var(vd, span) => {
            // A variable declared inside a class/struct scope is a member field.
            let kind = if class_name.is_some() {
                VarKind::Field
            } else if vd.is_static {
                VarKind::Static
            } else {
                VarKind::Let
            };
            let storage = if vd.is_static {
                StorageClass::Static
            } else if vd.is_extern {
                StorageClass::Extern
            } else {
                StorageClass::Local
            };
            let type_kind = expr_to_type_kind(&vd.type_);
            let mut var_builder = ParsedVariable::builder(&vd.name, kind)
                .line(span.start.line)
                .storage(storage)
                .type_kind(type_kind);
            if let Some(cn) = class_name {
                var_builder = var_builder.scope(Some(cn.to_string()), 0, 0);
            }
            let mut var = var_builder.build();
            var.is_static = vd.is_static;
            var.is_constexpr = vd.is_constexpr;
            var.is_extern = vd.is_extern;
            result.variables.push(var);
            if let Some(init) = &vd.init {
                walk_expr(init, result);
            }
        }
        Decl::Namespace(name, decls, span) => {
            result.symbols.push(
                ParsedSymbol::builder(SymbolKind::Namespace, name)
                    .lines(span.start.line, span.end.line)
                    .build(),
            );
            for inner in decls {
                walk_decl(inner, result, class_name);
            }
        }
        Decl::Using(path, span) => {
            result.imports.push(
                ParsedImport::builder(ImportKind::NamedImport, path)
                    .line(span.start.line)
                    .build(),
            );
        }
        Decl::UsingNamespace(path, span) => {
            result.imports.push(
                ParsedImport::builder(ImportKind::NamespaceImport, path)
                    .line(span.start.line)
                    .star(true)
                    .build(),
            );
        }
        Decl::Template(td, span) => {
            let params: Vec<String> = td.params.iter().map(template_param_to_string).collect();
            match &*td.decl {
                Decl::Func(fd, _) => {
                    process_func_decl(fd, span, result, class_name, &params);

                    if let Some(body) = &fd.body {
                        walk_block(body, result, class_name);
                    }
                }
                Decl::Class(cd, _) => {
                    let mut sym = ParsedSymbol::builder(SymbolKind::Class, &cd.name)
                        .lines(span.start.line, span.end.line)
                        .build();
                    sym.template_params = params;
                    sym.base_classes = cd.bases.iter().map(|b| b.name.clone()).collect();
                    result.symbols.push(sym);
                    walk_class_members(&cd.members, &cd.name, result);
                }
                _ => {
                    walk_decl(&td.decl, result, class_name);
                }
            }
        }
        Decl::Class(cd, span) => {
            let mut sym = ParsedSymbol::builder(SymbolKind::Class, &cd.name)
                .lines(span.start.line, span.end.line)
                .build();
            sym.base_classes = cd.bases.iter().map(|b| b.name.clone()).collect();
            sym.is_final = cd.is_final;
            result.symbols.push(sym);
            walk_class_members(&cd.members, &cd.name, result);
        }
        Decl::Struct(sd, span) => {
            result.symbols.push(
                ParsedSymbol::builder(SymbolKind::Struct, &sd.name)
                    .lines(span.start.line, span.end.line)
                    .build(),
            );
            for member in &sd.members {
                let b = ParsedVariable::builder(&member.name, VarKind::Field)
                    .type_kind(expr_to_type_kind(&member.type_))
                    .line(member.span.start.line)
                    .scope(Some(sd.name.clone()), 0, 0);
                result.variables.push(b.build());
            }
        }
        Decl::Enum(ed, span) => {
            result.symbols.push(
                ParsedSymbol::builder(SymbolKind::Enum, &ed.name)
                    .lines(span.start.line, span.end.line)
                    .build(),
            );
            for v in &ed.values {
                let b = ParsedVariable::builder(&v.name, VarKind::EnumMember)
                    .line(v.span.start.line)
                    .scope(Some(ed.name.clone()), 0, 0);
                result.variables.push(b.build());
            }
        }
        Decl::Typedef(td, span) => {
            result.symbols.push(
                ParsedSymbol::builder(SymbolKind::Type, &td.name)
                    .lines(span.start.line, span.end.line)
                    .build(),
            );
        }
        Decl::TypeAlias(name, _, span) => {
            result.symbols.push(
                ParsedSymbol::builder(SymbolKind::Type, name)
                    .lines(span.start.line, span.end.line)
                    .build(),
            );
        }
        Decl::StaticAssert(..) | Decl::Asm(..) => {}
    }
}

fn walk_class_members(members: &[ClassMember], class_name: &str, result: &mut ExtractionResult) {
    for member in members {
        if let ClassMember::Decl(decl, _) = member {
            walk_decl(decl, result, Some(class_name));
        }
    }
}

fn walk_block(block: &Block, result: &mut ExtractionResult, class_name: Option<&str>) {
    for stmt in &block.stmts {
        walk_stmt(stmt, result, class_name);
    }
}

fn walk_stmt(stmt: &Stmt, result: &mut ExtractionResult, class_name: Option<&str>) {
    match stmt {
        Stmt::Expr(expr, _) => walk_expr(expr, result),
        Stmt::Decl(decl, _) => walk_decl(decl, result, class_name),
        Stmt::If(cond, then, else_, _) => {
            walk_expr(cond, result);
            walk_stmt(then, result, class_name);
            if let Some(else_s) = else_ {
                walk_stmt(else_s, result, class_name);
            }
        }
        Stmt::Switch(expr, cases, _) => {
            walk_expr(expr, result);
            for case in cases {
                if let Some(e) = &case.expr {
                    walk_expr(e, result);
                }
                for s in &case.stmts {
                    walk_stmt(s, result, class_name);
                }
            }
        }
        Stmt::While(cond, body, _) => {
            walk_expr(cond, result);
            walk_stmt(body, result, class_name);
        }
        Stmt::Do(body, cond, _) => {
            walk_stmt(body, result, class_name);
            walk_expr(cond, result);
        }
        Stmt::For(init, cond, inc, body, _) => {
            if let Some(init_s) = init {
                walk_stmt(init_s, result, class_name);
            }
            if let Some(cond_e) = cond {
                walk_expr(cond_e, result);
            }
            if let Some(inc_s) = inc {
                walk_stmt(inc_s, result, class_name);
            }
            walk_stmt(body, result, class_name);
        }
        Stmt::RangeFor(decl_expr, expr, body, _) => {
            walk_stmt(decl_expr, result, class_name);
            walk_expr(expr, result);
            walk_stmt(body, result, class_name);
        }
        Stmt::Return(Some(expr), _) => walk_expr(expr, result),
        Stmt::Return(None, _)
        | Stmt::Break(_)
        | Stmt::Continue(_)
        | Stmt::Goto(..)
        | Stmt::Label(..) => {}
        Stmt::Try(body, catches, finally, _) => {
            walk_stmt(body, result, class_name);
            for catch in catches {
                walk_stmt(&catch.body, result, class_name);
            }
            if let Some(f) = finally {
                walk_stmt(f, result, class_name);
            }
        }
        Stmt::Throw(Some(expr), _) => walk_expr(expr, result),
        Stmt::Throw(None, _) => {}
        Stmt::Block(block, _) => walk_block(block, result, class_name),
        Stmt::Empty(_) => {}
    }
}

fn walk_expr(expr: &Expr, result: &mut ExtractionResult) {
    match expr {
        Expr::Call(callee, args, span) => {
            let (kind, callee_text, object) = match &**callee {
                Expr::Member(obj, name, _) => (
                    CallKind::MethodCall,
                    name.clone(),
                    Some(expr_to_string(obj)),
                ),
                Expr::Arrow(obj, name, _) => (
                    CallKind::SelectorCall,
                    name.clone(),
                    Some(expr_to_string(obj)),
                ),
                _ => (CallKind::FunctionCall, expr_to_string(callee), None),
            };

            let mut builder =
                ParsedCall::builder(kind, callee_text).pos(span.start.line, span.start.column);

            if let Some(obj) = object {
                builder = builder.object(obj);
            }

            result.calls.push(builder.build());

            for arg in args {
                walk_expr(arg, result);
            }
            walk_expr(callee, result);
        }
        Expr::Binary(lhs, _, rhs, _) => {
            walk_expr(lhs, result);
            walk_expr(rhs, result);
        }
        Expr::Unary(_, e, _) => walk_expr(e, result),
        Expr::Index(obj, idx, _) => {
            walk_expr(obj, result);
            walk_expr(idx, result);
        }
        Expr::Member(obj, _, _) => walk_expr(obj, result),
        Expr::Arrow(obj, _, _) => walk_expr(obj, result),
        Expr::Deref(e, _) => walk_expr(e, result),
        Expr::Ref(e, _) => walk_expr(e, result),
        Expr::Cast(e, t, _)
        | Expr::DynamicCast(e, t, _)
        | Expr::StaticCast(e, t, _)
        | Expr::ConstCast(e, t, _)
        | Expr::ReinterpretCast(e, t, _) => {
            walk_expr(e, result);
            walk_expr(t, result);
        }
        Expr::Sizeof(e, _) | Expr::Alignof(e, _) | Expr::Typeid(e, _) => walk_expr(e, result),
        Expr::Ternary(cond, then, else_, _) => {
            walk_expr(cond, result);
            walk_expr(then, result);
            walk_expr(else_, result);
        }
        Expr::Comma(exprs, _) => {
            for e in exprs {
                walk_expr(e, result);
            }
        }
        Expr::Lambda(lambda, _) => {
            if let Some(ret_type) = &lambda.return_type {
                walk_expr(ret_type, result);
            }
            walk_block(&lambda.body, result, None);
        }
        Expr::New(e, args, span) => {
            let type_name = expr_to_string(e);
            if !type_name.is_empty() {
                result.calls.push(
                    ParsedCall::builder(CallKind::ConstructorCall, &type_name)
                        .pos(span.start.line, span.start.column)
                        .build(),
                );
            }
            walk_expr(e, result);
            for arg in args {
                walk_expr(arg, result);
            }
        }
        Expr::Paren(e, _) => walk_expr(e, result),
        Expr::Delete(e, _) => walk_expr(e, result),
        Expr::Assign(lhs, rhs, _) => {
            walk_expr(lhs, result);
            walk_expr(rhs, result);
        }
        Expr::Template(callee, args, _) => {
            walk_expr(callee, result);
            for arg in args {
                walk_expr(arg, result);
            }
        }
        Expr::BraceInit(exprs, _) => {
            for e in exprs {
                walk_expr(e, result);
            }
        }
        Expr::Int(..)
        | Expr::UInt(..)
        | Expr::Float(..)
        | Expr::String(..)
        | Expr::Char(..)
        | Expr::Bool(..)
        | Expr::NullPtr(..)
        | Expr::Ident(..)
        | Expr::This(..)
        | Expr::Error(..) => {}
    }
}
