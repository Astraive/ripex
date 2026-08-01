use super::ast::ImportSpecifier as AstImportSpecifier;
use super::ast::VarKind as AstVarKind;
use super::ast::{
    BindingIdent, ClassMember, Decl, ExportDecl, Expr, ExprRef, ForInit, ImportDecl, Lit,
    ModuleItem, ObjectPatProp, Pat, Program, PropName, Stmt, TypeAnn,
};
use super::parser::expressions::HasSpan;
use crate::arena::Arena;
use crate::facts::{
    CallKind, ImportKind, ImportSpecifier, ImportSpecifierKind, ParsedCall, ParsedImport,
    ParsedSymbol, ParsedVariable, SymbolKind, TypeKind, VarKind,
};
use crate::ExtractionResult;
use std::collections::HashSet;

fn expr_name(expr_ref: ExprRef, arena: &Arena<Expr>) -> Option<String> {
    match &arena[expr_ref] {
        Expr::Ident(id) => Some(id.name.clone()),
        Expr::Lit(Lit::Str(s)) => Some(s.value.clone()),
        Expr::This(_) => Some("this".into()),
        Expr::Super(_) => Some("super".into()),
        Expr::Member(m) => {
            let obj = expr_name(m.object, arena).unwrap_or_default();
            let prop = member_prop_name(&m.property, arena);
            if obj.is_empty() {
                prop
            } else {
                Some(format!("{}.{}", obj, prop.unwrap_or_default()))
            }
        }
        Expr::PrivateName(p) => Some(p.name.name.clone()),
        Expr::TSAs(e) => expr_name(e.expr, arena),
        Expr::TSSatisfies(e) => expr_name(e.expr, arena),
        Expr::TSTypeAssertion(e) => expr_name(e.expr, arena),
        Expr::TSNonNull(e) => expr_name(e.expr, arena),
        Expr::TSInst(e) => expr_name(e.expr, arena),
        Expr::Parenthesized(e) => expr_name(e.expr, arena),
        Expr::Chain(e) => expr_name(e.expr, arena),
        _ => None,
    }
}

fn member_prop_name(expr: &Expr, _arena: &Arena<Expr>) -> Option<String> {
    match expr {
        Expr::Ident(id) => Some(id.name.clone()),
        Expr::Lit(Lit::Str(s)) => Some(s.value.clone()),
        Expr::PrivateName(p) => Some(p.name.name.clone()),
        _ => None,
    }
}

fn prop_name_to_string(pn: &PropName, arena: &Arena<Expr>) -> String {
    match pn {
        PropName::Ident(id) => id.name.clone(),
        PropName::Str(s) => s.value.clone(),
        PropName::Num(n) => n.raw.clone(),
        PropName::Computed(r) => expr_name(*r, arena).unwrap_or_else(|| "[expr]".into()),
    }
}

fn resolve_callee(callee: ExprRef, arena: &Arena<Expr>) -> (CallKind, String) {
    match &arena[callee] {
        Expr::Ident(id) => (CallKind::FunctionCall, id.name.clone()),
        Expr::Super(_) => (CallKind::SuperCall, "super".into()),
        Expr::Member(m) => {
            let obj = expr_name(m.object, arena).unwrap_or_default();
            let prop = member_prop_name(&m.property, arena).unwrap_or_default();
            let callee_text = if obj.is_empty() {
                prop.clone()
            } else {
                format!("{}.{}", obj, prop)
            };
            (CallKind::MethodCall, callee_text)
        }
        Expr::OptionalMember(m) => {
            let obj = expr_name(m.object, arena).unwrap_or_default();
            let prop = member_prop_name(&m.property, arena).unwrap_or_default();
            let callee_text = if obj.is_empty() {
                prop.clone()
            } else {
                format!("{}.{}", obj, prop)
            };
            (CallKind::MethodCall, callee_text)
        }
        Expr::TSAs(e) => resolve_callee(e.expr, arena),
        Expr::TSSatisfies(e) => resolve_callee(e.expr, arena),
        Expr::TSTypeAssertion(e) => resolve_callee(e.expr, arena),
        Expr::TSNonNull(e) => resolve_callee(e.expr, arena),
        Expr::TSInst(e) => resolve_callee(e.expr, arena),
        Expr::Parenthesized(e) => resolve_callee(e.expr, arena),
        Expr::Chain(e) => resolve_callee(e.expr, arena),
        _ => (CallKind::FunctionCall, "unknown".into()),
    }
}

fn expr_span(expr: &Expr) -> crate::span::Span {
    match expr {
        Expr::Ident(e) => e.span,
        Expr::Lit(Lit::Str(e)) => e.span,
        Expr::Lit(Lit::Num(e)) => e.span,
        Expr::Lit(Lit::Bool(e)) => e.span,
        Expr::Lit(Lit::Null(e)) => e.span,
        Expr::Lit(Lit::BigInt(e)) => e.span,
        Expr::Lit(Lit::RegExp(e)) => e.span,
        Expr::Lit(Lit::Template(e)) => e.span,
        Expr::This(e) => e.span,
        Expr::Super(e) => e.span,
        Expr::Array(e) => e.span,
        Expr::Object(e) => e.span,
        Expr::Fn(e) => e.span,
        Expr::Arrow(e) => e.span,
        Expr::Class(e) => e.span,
        Expr::New(e) => e.span,
        Expr::Call(e) => e.span,
        Expr::OptionalCall(e) => e.span,
        Expr::Member(e) => e.span,
        Expr::OptionalMember(e) => e.span,
        Expr::Unary(e) => e.span,
        Expr::UnaryOp(e) => e.span,
        Expr::Binary(e) => e.span,
        Expr::Logical(e) => e.span,
        Expr::Conditional(e) => e.span,
        Expr::Assignment(e) => e.span,
        Expr::Sequence(e) => e.span,
        Expr::Update(e) => e.span,
        Expr::Await(e) => e.span,
        Expr::Yield(e) => e.span,
        Expr::Spread(e) => e.span,
        Expr::Template(e) => e.span,
        Expr::TaggedTemplate(e) => e.span,
        Expr::MetaProperty(e) => e.span,
        Expr::Import(e) => e.span,
        Expr::JSXElement(e) => e.span,
        Expr::JSXFragment(e) => e.span,
        Expr::TSAs(e) => e.span,
        Expr::TSSatisfies(e) => e.span,
        Expr::TSTypeAssertion(e) => e.span,
        Expr::TSNonNull(e) => e.span,
        Expr::TSInst(e) => e.span,
        Expr::Parenthesized(e) => e.span,
        Expr::PrivateName(e) => e.span,
        Expr::Record(e) => e.span,
        Expr::Tuple(e) => e.span,
        Expr::Pipeline(e) => e.span,
        Expr::Chain(e) => e.span,
        Expr::Invalid(e) => e.span,
    }
}

fn convert_type_ann(ann: &TypeAnn) -> TypeKind {
    match ann {
        TypeAnn::String(_) => TypeKind::simple("string"),
        TypeAnn::Number(_) => TypeKind::simple("number"),
        TypeAnn::Boolean(_) => TypeKind::simple("boolean"),
        TypeAnn::Void(_) => TypeKind::Void,
        TypeAnn::Never(_) => TypeKind::Never,
        TypeAnn::Unknown(_) | TypeAnn::Any(_) => TypeKind::Unknown,
        TypeAnn::Null(_) | TypeAnn::TsNull(_) => TypeKind::simple("null"),
        TypeAnn::Undefined(_) => TypeKind::simple("undefined"),
        TypeAnn::Ident(id) => TypeKind::simple(&id.name),
        TypeAnn::Array(inner) => TypeKind::Array(Box::new(convert_type_ann(inner))),
        TypeAnn::Union(variants) => {
            TypeKind::Union(variants.iter().map(convert_type_ann).collect())
        }
        TypeAnn::Generic(ident, params) => TypeKind::Generic(
            ident.name.clone(),
            params.iter().map(convert_type_ann).collect(),
        ),
        TypeAnn::Fn(params, ret) => TypeKind::FnPtr(
            params.iter().map(convert_type_ann).collect(),
            Box::new(convert_type_ann(ret)),
        ),
        TypeAnn::Tuple(types) => TypeKind::Tuple(types.iter().map(convert_type_ann).collect()),
        TypeAnn::Optional(inner) => TypeKind::Optional(Box::new(convert_type_ann(inner))),
        _ => TypeKind::Unknown,
    }
}

// ─── symbols ───

pub fn extract_symbols(program: &Program, arena: &Arena<Expr>) -> Vec<ParsedSymbol> {
    let mut symbols = Vec::new();
    match program {
        Program::Script(s) => {
            walk_stmts_for_symbols(&s.body, false, arena, &mut symbols);
        }
        Program::Module(m) => {
            walk_module_items_for_symbols(&m.body, arena, &mut symbols);
        }
    }
    symbols
}

fn walk_module_items_for_symbols(
    items: &[ModuleItem],
    arena: &Arena<Expr>,
    symbols: &mut Vec<ParsedSymbol>,
) {
    for item in items {
        match item {
            ModuleItem::Stmt(stmt) => walk_stmt_for_symbols(stmt, false, arena, symbols),
            ModuleItem::Decl(decl) => {
                process_symbol_decl(decl, false, arena, symbols);
                walk_into_decl_body(decl, arena, symbols);
            }
            ModuleItem::Import(_) => {}
            ModuleItem::Export(e) => process_export_symbols(e, arena, symbols),
        }
    }
}

fn process_export_symbols(
    export: &ExportDecl,
    arena: &Arena<Expr>,
    symbols: &mut Vec<ParsedSymbol>,
) {
    match export {
        ExportDecl::Named(named) => {
            if let Some(decl) = &named.decl {
                process_symbol_decl(decl, true, arena, symbols);
                walk_into_decl_body(decl, arena, symbols);
            }
        }
        ExportDecl::Default(default) => {
            let span = expr_span(&arena[default.decl]);
            match &arena[default.decl] {
                Expr::Fn(f) => {
                    if let Some(id) = &f.id {
                        symbols.push(
                            ParsedSymbol::builder(SymbolKind::Function, &id.name)
                                .exported(true)
                                .lines(span.start.line, span.end.line)
                                .is_async(f.async_)
                                .build(),
                        );
                    }
                }
                Expr::Class(c) => {
                    if let Some(id) = &c.id {
                        let base_classes: Vec<String> = c
                            .super_class
                            .as_ref()
                            .and_then(|r| expr_name(*r, arena))
                            .map(|n| vec![n])
                            .unwrap_or_default();
                        let mut sym = ParsedSymbol::builder(SymbolKind::Class, &id.name)
                            .exported(true)
                            .lines(span.start.line, span.end.line)
                            .build();
                        sym.base_classes = base_classes;
                        symbols.push(sym);
                    }
                }
                Expr::Ident(id) => {
                    symbols.push(
                        ParsedSymbol::builder(SymbolKind::Variable, &id.name)
                            .exported(true)
                            .lines(span.start.line, span.end.line)
                            .build(),
                    );
                }
                _ => {}
            }
        }
        ExportDecl::All(_) => {}
    }
}

fn walk_stmts_for_symbols(
    stmts: &[Stmt],
    exported: bool,
    arena: &Arena<Expr>,
    symbols: &mut Vec<ParsedSymbol>,
) {
    for stmt in stmts {
        walk_stmt_for_symbols(stmt, exported, arena, symbols);
    }
}

fn walk_stmt_for_symbols(
    stmt: &Stmt,
    exported: bool,
    arena: &Arena<Expr>,
    symbols: &mut Vec<ParsedSymbol>,
) {
    match stmt {
        Stmt::Block(b) => walk_stmts_for_symbols(&b.stmts, exported, arena, symbols),
        Stmt::If(s) => {
            walk_stmt_for_symbols(&s.consequent, exported, arena, symbols);
            if let Some(alt) = &s.alternate {
                walk_stmt_for_symbols(alt, exported, arena, symbols);
            }
        }
        Stmt::Switch(s) => {
            for case in &s.cases {
                walk_stmts_for_symbols(&case.consequent, exported, arena, symbols);
            }
        }
        Stmt::For(s) => {
            if let Some(ForInit::Decl(d)) = &s.init {
                process_symbol_decl(d, exported, arena, symbols);
                walk_into_decl_body(d, arena, symbols);
            }
            walk_stmt_for_symbols(&s.body, exported, arena, symbols);
        }
        Stmt::ForIn(s) => {
            if let ForInit::Decl(d) = &s.left {
                process_symbol_decl(d, exported, arena, symbols);
                walk_into_decl_body(d, arena, symbols);
            }
            walk_stmt_for_symbols(&s.body, exported, arena, symbols);
        }
        Stmt::ForOf(s) => {
            if let ForInit::Decl(d) = &s.left {
                process_symbol_decl(d, exported, arena, symbols);
                walk_into_decl_body(d, arena, symbols);
            }
            walk_stmt_for_symbols(&s.body, exported, arena, symbols);
        }
        Stmt::While(s) => walk_stmt_for_symbols(&s.body, exported, arena, symbols),
        Stmt::DoWhile(s) => walk_stmt_for_symbols(&s.body, exported, arena, symbols),
        Stmt::Try(s) => {
            walk_stmts_for_symbols(&s.block.stmts, exported, arena, symbols);
            if let Some(h) = &s.handler {
                walk_stmts_for_symbols(&h.body.stmts, exported, arena, symbols);
            }
            if let Some(f) = &s.finalizer {
                walk_stmts_for_symbols(&f.stmts, exported, arena, symbols);
            }
        }
        Stmt::Labelled(s) => {
            walk_stmt_for_symbols(&s.body, exported, arena, symbols);
        }
        Stmt::With(s) => walk_stmt_for_symbols(&s.body, exported, arena, symbols),
        Stmt::Decl(decl) => {
            process_symbol_decl(decl, exported, arena, symbols);
            walk_into_decl_body(decl, arena, symbols);
        }
        _ => {}
    }
}

fn walk_into_decl_body(decl: &Decl, arena: &Arena<Expr>, symbols: &mut Vec<ParsedSymbol>) {
    match decl {
        Decl::Fn(f) => {
            if let Some(body) = &f.body {
                walk_stmts_for_symbols(&body.stmts, false, arena, symbols);
            }
        }
        Decl::Class(c) => {
            for member in &c.body {
                match member {
                    ClassMember::Method(m) => {
                        if let Some(body) = &m.function.body {
                            walk_stmts_for_symbols(&body.stmts, false, arena, symbols);
                        }
                    }
                    ClassMember::Ctor(ctor) => {
                        if let Some(body) = &ctor.body {
                            walk_stmts_for_symbols(&body.stmts, false, arena, symbols);
                        }
                    }
                    ClassMember::Getter(g) => {
                        if let Some(body) = &g.body {
                            walk_stmts_for_symbols(&body.stmts, false, arena, symbols);
                        }
                    }
                    ClassMember::Setter(s) => {
                        if let Some(body) = &s.body {
                            walk_stmts_for_symbols(&body.stmts, false, arena, symbols);
                        }
                    }
                    _ => {}
                }
            }
        }
        Decl::TsModule(m) => {
            walk_stmts_for_symbols(&m.body, false, arena, symbols);
        }
        _ => {}
    }
}

fn process_symbol_decl(
    decl: &Decl,
    exported: bool,
    arena: &Arena<Expr>,
    symbols: &mut Vec<ParsedSymbol>,
) {
    match decl {
        Decl::Fn(f) => {
            symbols.push(
                ParsedSymbol::builder(SymbolKind::Function, &f.id.name)
                    .exported(exported)
                    .lines(f.span.start.line, f.span.end.line)
                    .is_async(f.async_)
                    .is_test(is_js_test_fn(&f.id.name))
                    .build(),
            );
        }
        Decl::Class(c) => {
            let base_classes: Vec<String> = c
                .super_class
                .as_ref()
                .and_then(|r| expr_name(*r, arena))
                .map(|n| vec![n])
                .unwrap_or_default();
            let mut sym = ParsedSymbol::builder(SymbolKind::Class, &c.id.name)
                .exported(exported)
                .lines(c.span.start.line, c.span.end.line)
                .abstract_(c.abstract_)
                .build();
            sym.base_classes = base_classes;
            symbols.push(sym);

            for member in &c.body {
                process_class_member(member, &c.id.name, arena, symbols);
            }
        }
        Decl::TsInterface(i) => {
            let base_classes: Vec<String> = i
                .extends
                .iter()
                .filter_map(|ann| match ann {
                    TypeAnn::Ident(id) => Some(id.name.clone()),
                    _ => None,
                })
                .collect();
            let mut sym = ParsedSymbol::builder(SymbolKind::Interface, &i.id.name)
                .exported(exported)
                .lines(i.span.start.line, i.span.end.line)
                .build();
            sym.base_classes = base_classes;
            symbols.push(sym);
        }
        Decl::TsTypeAlias(t) => {
            symbols.push(
                ParsedSymbol::builder(SymbolKind::Type, &t.id.name)
                    .exported(exported)
                    .lines(t.span.start.line, t.span.end.line)
                    .build(),
            );
        }
        Decl::TsEnum(e) => {
            symbols.push(
                ParsedSymbol::builder(SymbolKind::Enum, &e.id.name)
                    .exported(exported)
                    .lines(e.span.start.line, e.span.end.line)
                    .build(),
            );
        }
        Decl::TsModule(m) => {
            symbols.push(
                ParsedSymbol::builder(SymbolKind::Module, &m.id.name)
                    .exported(exported)
                    .lines(m.span.start.line, m.span.end.line)
                    .build(),
            );
        }
        Decl::Var(var) => {
            for declarator in &var.decls {
                let Pat::Ident(binding) = &declarator.name else {
                    continue;
                };
                let kind = if var.kind == AstVarKind::Const {
                    SymbolKind::Constant
                } else {
                    SymbolKind::Variable
                };
                let mut symbol = ParsedSymbol::builder(kind, &binding.id.name)
                    .exported(exported)
                    .lines(declarator.span.start.line, declarator.span.end.line)
                    .build();
                if let Some(type_ann) = &binding.type_ann {
                    symbol.type_kind = convert_type_ann(type_ann);
                }
                symbols.push(symbol);
            }
        }
    }
}

fn process_class_member(
    member: &ClassMember,
    class_name: &str,
    arena: &Arena<Expr>,
    symbols: &mut Vec<ParsedSymbol>,
) {
    match member {
        ClassMember::Method(m) => {
            let method_name = prop_name_to_string(&m.key, arena);
            let name = format!("{}.{}", class_name, method_name);
            symbols.push(
                ParsedSymbol::builder(SymbolKind::Method, name)
                    .lines(m.span.start.line, m.span.end.line)
                    .is_async(m.function.async_)
                    .build(),
            );
        }
        ClassMember::Ctor(c) => {
            let name = format!("{}.{}", class_name, "(constructor)");
            symbols.push(
                ParsedSymbol::builder(SymbolKind::Constructor, name)
                    .constructor(true)
                    .lines(c.span.start.line, c.span.end.line)
                    .build(),
            );
        }
        ClassMember::Getter(g) => {
            let name = format!("{}.{}", class_name, prop_name_to_string(&g.key, arena));
            symbols.push(
                ParsedSymbol::builder(SymbolKind::Getter, name)
                    .lines(g.span.start.line, g.span.end.line)
                    .build(),
            );
        }
        ClassMember::Setter(s) => {
            let name = format!("{}.{}", class_name, prop_name_to_string(&s.key, arena));
            symbols.push(
                ParsedSymbol::builder(SymbolKind::Setter, name)
                    .lines(s.span.start.line, s.span.end.line)
                    .build(),
            );
        }
        _ => {}
    }
}

// ─── imports ───

pub fn extract_imports(program: &Program, arena: &Arena<Expr>) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    if let Program::Module(m) = program {
        for item in &m.body {
            match item {
                ModuleItem::Import(imp) => process_import_decl(imp, &mut imports),
                ModuleItem::Export(e) => process_export_for_imports(e, &mut imports),
                _ => {}
            }
        }
    }
    for (_, expression) in arena.iter() {
        if let Expr::Import(import) = expression {
            let source = expr_name(import.source, arena).unwrap_or_else(|| "<dynamic>".into());
            imports.push(
                ParsedImport::builder(ImportKind::DynamicImport, source)
                    .line(import.span.start.line)
                    .build(),
            );
        }
    }
    imports
}

fn process_import_decl(imp: &ImportDecl, imports: &mut Vec<ParsedImport>) {
    let source = imp.source.value.clone();
    let line = imp.span.start.line;
    if imp.specifiers.is_empty() {
        imports.push(
            ParsedImport::builder(ImportKind::SideEffectImport, source)
                .line(line)
                .build(),
        );
        return;
    }
    for spec in &imp.specifiers {
        match spec {
            AstImportSpecifier::Default(d) => {
                let kind = if imp.is_type_only {
                    ImportKind::TypeImport
                } else {
                    ImportKind::DefaultImport
                };
                imports.push(
                    ParsedImport::builder(kind, &source)
                        .local(&d.local.name)
                        .line(line)
                        .type_only(imp.is_type_only)
                        .build(),
                );
            }
            AstImportSpecifier::Named(n) => {
                let is_type_only = imp.is_type_only || n.is_type_only;
                let specifiers = vec![ImportSpecifier {
                    imported: n.imported.name.clone(),
                    local: n.local.name.clone(),
                    kind: if is_type_only {
                        ImportSpecifierKind::Type
                    } else {
                        ImportSpecifierKind::Named
                    },
                }];
                imports.push(
                    ParsedImport::builder(
                        if is_type_only {
                            ImportKind::TypeImport
                        } else {
                            ImportKind::NamedImport
                        },
                        &source,
                    )
                    .local(&n.local.name)
                    .imported(&n.imported.name)
                    .line(line)
                    .type_only(is_type_only)
                    .specifiers(specifiers)
                    .build(),
                );
            }
            AstImportSpecifier::Namespace(ns) => {
                let kind = if imp.is_type_only {
                    ImportKind::TypeImport
                } else {
                    ImportKind::NamespaceImport
                };
                imports.push(
                    ParsedImport::builder(kind, &source)
                        .local(&ns.local.name)
                        .line(line)
                        .type_only(imp.is_type_only)
                        .star(true)
                        .build(),
                );
            }
        }
    }
}

fn process_export_for_imports(export: &ExportDecl, imports: &mut Vec<ParsedImport>) {
    match export {
        ExportDecl::Named(named) => {
            if let Some(source) = &named.source {
                let specifiers: Vec<ImportSpecifier> = named
                    .specifiers
                    .iter()
                    .map(|s| ImportSpecifier {
                        imported: s.local.name.clone(),
                        local: s.exported.name.clone(),
                        kind: if named.is_type_only || s.is_type_only {
                            ImportSpecifierKind::Type
                        } else {
                            ImportSpecifierKind::Named
                        },
                    })
                    .collect();
                let is_type_only = named.is_type_only
                    || (!named.specifiers.is_empty()
                        && named
                            .specifiers
                            .iter()
                            .all(|specifier| specifier.is_type_only));
                imports.push(
                    ParsedImport::builder(
                        if is_type_only {
                            ImportKind::TypeReExport
                        } else {
                            ImportKind::ReExport
                        },
                        &source.value,
                    )
                    .line(named.span.start.line)
                    .reexport(true)
                    .type_only(is_type_only)
                    .specifiers(specifiers)
                    .build(),
                );
            }
        }
        ExportDecl::All(all) => {
            imports.push(
                ParsedImport::builder(
                    if all.is_type_only {
                        ImportKind::TypeReExport
                    } else {
                        ImportKind::ReExport
                    },
                    &all.source.value,
                )
                .line(all.span.start.line)
                .reexport(true)
                .type_only(all.is_type_only)
                .star(true)
                .build(),
            );
        }
        _ => {}
    }
}

// ─── calls ───

pub fn extract_calls(_program: &Program, arena: &Arena<Expr>) -> Vec<ParsedCall> {
    let mut calls = Vec::new();
    let awaited = arena
        .iter()
        .filter_map(|(_, expr)| match expr {
            Expr::Await(await_expr) => Some(await_expr.arg),
            _ => None,
        })
        .collect::<HashSet<_>>();
    for (id, expr) in arena.iter() {
        push_calls_for_expr(expr, arena, awaited.contains(&id), &mut calls);
    }
    calls
}

/// Emits a call fact for a single expression node. The expression arena owns
/// every nested expression, so this intentionally does not recurse: recursion
/// would duplicate facts for calls nested inside pipeline expressions.
fn push_calls_for_expr(
    expr: &Expr,
    arena: &Arena<Expr>,
    is_await: bool,
    calls: &mut Vec<ParsedCall>,
) {
    match expr {
        Expr::Call(call) => {
            let (kind, callee_text) = resolve_callee(call.callee, arena);
            let callee_obj = resolve_callee_object(call.callee, arena);
            let mut fact = ParsedCall::builder(kind, callee_text)
                .pos(call.span.start.line, call.span.start.column)
                .await_(is_await)
                .optional(callee_is_optional(call.callee, arena))
                .type_args(call_type_args(call.callee, arena));
            if let Some(object) = callee_obj {
                fact = fact.object(object);
            }
            if let Ok(fact) = fact.try_build() {
                calls.push(fact);
            }
        }
        Expr::OptionalCall(oc) => {
            let (kind, callee_text) = resolve_callee(oc.callee, arena);
            let callee_obj = resolve_callee_object(oc.callee, arena);
            let mut fact = ParsedCall::builder(kind, callee_text)
                .pos(oc.span.start.line, oc.span.start.column)
                .await_(is_await)
                .optional(true)
                .type_args(call_type_args(oc.callee, arena));
            if let Some(object) = callee_obj {
                fact = fact.object(object);
            }
            if let Ok(fact) = fact.try_build() {
                calls.push(fact);
            }
        }
        Expr::New(new) => {
            let callee_text = expr_name(new.callee, arena).unwrap_or_else(|| "unknown".into());
            if let Ok(fact) = ParsedCall::builder(CallKind::ConstructorCall, callee_text)
                .pos(new.span.start.line, new.span.start.column)
                .type_args(call_type_args(new.callee, arena))
                .try_build()
            {
                calls.push(fact);
            }
        }
        Expr::TaggedTemplate(tagged) => {
            let callee_text = expr_name(tagged.tag, arena).unwrap_or_else(|| "unknown".into());
            if let Ok(fact) = ParsedCall::builder(CallKind::FunctionCall, callee_text)
                .pos(tagged.span.start.line, tagged.span.start.column)
                .try_build()
            {
                calls.push(fact);
            }
        }
        // A pipeline's body is the function applied to the input. A concrete
        // call in the body has its own arena node, so emit a synthetic fact
        // only for a bare function/member target.
        Expr::Pipeline(p)
            if !matches!(
                arena[p.body],
                Expr::Call(_) | Expr::OptionalCall(_) | Expr::New(_) | Expr::TaggedTemplate(_)
            ) =>
        {
            let (kind, name) = resolve_callee(p.body, arena);
            if let Ok(fact) = ParsedCall::builder(kind, name)
                .pos(
                    arena[p.body].span().start.line,
                    arena[p.body].span().start.column,
                )
                .await_(is_await)
                .optional(callee_is_optional(p.body, arena))
                .type_args(call_type_args(p.body, arena))
                .try_build()
            {
                calls.push(fact);
            }
        }
        _ => {}
    }
}

fn resolve_callee_object(callee: ExprRef, arena: &Arena<Expr>) -> Option<String> {
    match &arena[callee] {
        Expr::Member(m) => expr_name(m.object, arena),
        Expr::OptionalMember(m) => expr_name(m.object, arena),
        Expr::TSAs(e) => resolve_callee_object(e.expr, arena),
        Expr::TSSatisfies(e) => resolve_callee_object(e.expr, arena),
        Expr::TSTypeAssertion(e) => resolve_callee_object(e.expr, arena),
        Expr::TSNonNull(e) => resolve_callee_object(e.expr, arena),
        Expr::TSInst(e) => resolve_callee_object(e.expr, arena),
        Expr::Parenthesized(e) => resolve_callee_object(e.expr, arena),
        Expr::Chain(e) => resolve_callee_object(e.expr, arena),
        _ => None,
    }
}

fn callee_is_optional(callee: ExprRef, arena: &Arena<Expr>) -> bool {
    match &arena[callee] {
        Expr::OptionalMember(_) => true,
        Expr::TSAs(e) => callee_is_optional(e.expr, arena),
        Expr::TSSatisfies(e) => callee_is_optional(e.expr, arena),
        Expr::TSTypeAssertion(e) => callee_is_optional(e.expr, arena),
        Expr::TSNonNull(e) => callee_is_optional(e.expr, arena),
        Expr::TSInst(e) => callee_is_optional(e.expr, arena),
        Expr::Parenthesized(e) => callee_is_optional(e.expr, arena),
        Expr::Chain(e) => callee_is_optional(e.expr, arena),
        _ => false,
    }
}

fn call_type_args(callee: ExprRef, arena: &Arena<Expr>) -> Vec<TypeKind> {
    match &arena[callee] {
        Expr::TSInst(e) => e.type_params.iter().map(convert_type_ann).collect(),
        Expr::TSAs(e) => call_type_args(e.expr, arena),
        Expr::TSSatisfies(e) => call_type_args(e.expr, arena),
        Expr::TSTypeAssertion(e) => call_type_args(e.expr, arena),
        Expr::TSNonNull(e) => call_type_args(e.expr, arena),
        Expr::Parenthesized(e) => call_type_args(e.expr, arena),
        Expr::Chain(e) => call_type_args(e.expr, arena),
        _ => Vec::new(),
    }
}

// ─── variables ───

pub fn extract_variables(program: &Program, arena: &Arena<Expr>) -> Vec<ParsedVariable> {
    let mut variables = Vec::new();
    match program {
        Program::Script(s) => {
            walk_stmts_for_variables(&s.body, arena, &mut variables);
        }
        Program::Module(m) => {
            walk_module_items_for_variables(&m.body, arena, &mut variables);
        }
    }
    variables
}

fn walk_module_items_for_variables(
    items: &[ModuleItem],
    arena: &Arena<Expr>,
    variables: &mut Vec<ParsedVariable>,
) {
    for item in items {
        match item {
            ModuleItem::Stmt(stmt) => walk_stmt_for_variables(stmt, arena, variables),
            ModuleItem::Decl(decl) => {
                process_var_decl(decl, arena, variables);
                walk_into_decl_for_variables(decl, arena, variables);
            }
            ModuleItem::Import(_) => {}
            ModuleItem::Export(e) => process_export_for_variables(e, arena, variables),
        }
    }
}

fn process_export_for_variables(
    export: &ExportDecl,
    arena: &Arena<Expr>,
    variables: &mut Vec<ParsedVariable>,
) {
    if let ExportDecl::Named(named) = export {
        if let Some(decl) = &named.decl {
            process_var_decl(decl, arena, variables);
            walk_into_decl_for_variables(decl, arena, variables);
        }
    }
}

fn walk_stmts_for_variables(
    stmts: &[Stmt],
    arena: &Arena<Expr>,
    variables: &mut Vec<ParsedVariable>,
) {
    for stmt in stmts {
        walk_stmt_for_variables(stmt, arena, variables);
    }
}

fn walk_stmt_for_variables(stmt: &Stmt, arena: &Arena<Expr>, variables: &mut Vec<ParsedVariable>) {
    match stmt {
        Stmt::Block(b) => walk_stmts_for_variables(&b.stmts, arena, variables),
        Stmt::If(s) => {
            walk_stmt_for_variables(&s.consequent, arena, variables);
            if let Some(alt) = &s.alternate {
                walk_stmt_for_variables(alt, arena, variables);
            }
        }
        Stmt::Switch(s) => {
            for case in &s.cases {
                walk_stmts_for_variables(&case.consequent, arena, variables);
            }
        }
        Stmt::For(s) => {
            if let Some(ForInit::Decl(d)) = &s.init {
                process_var_decl(d, arena, variables);
                walk_into_decl_for_variables(d, arena, variables);
            }
            walk_stmt_for_variables(&s.body, arena, variables);
        }
        Stmt::ForIn(s) => {
            if let ForInit::Decl(d) = &s.left {
                process_var_decl(d, arena, variables);
                walk_into_decl_for_variables(d, arena, variables);
            }
            walk_stmt_for_variables(&s.body, arena, variables);
        }
        Stmt::ForOf(s) => {
            if let ForInit::Decl(d) = &s.left {
                process_var_decl(d, arena, variables);
                walk_into_decl_for_variables(d, arena, variables);
            }
            walk_stmt_for_variables(&s.body, arena, variables);
        }
        Stmt::While(s) => walk_stmt_for_variables(&s.body, arena, variables),
        Stmt::DoWhile(s) => walk_stmt_for_variables(&s.body, arena, variables),
        Stmt::Try(s) => {
            walk_stmts_for_variables(&s.block.stmts, arena, variables);
            if let Some(h) = &s.handler {
                walk_stmts_for_variables(&h.body.stmts, arena, variables);
            }
            if let Some(f) = &s.finalizer {
                walk_stmts_for_variables(&f.stmts, arena, variables);
            }
        }
        Stmt::Labelled(s) => walk_stmt_for_variables(&s.body, arena, variables),
        Stmt::With(s) => walk_stmt_for_variables(&s.body, arena, variables),
        Stmt::Decl(decl) => {
            process_var_decl(decl, arena, variables);
            walk_into_decl_for_variables(decl, arena, variables);
        }
        _ => {}
    }
}

fn walk_into_decl_for_variables(
    decl: &Decl,
    arena: &Arena<Expr>,
    variables: &mut Vec<ParsedVariable>,
) {
    match decl {
        Decl::Fn(f) => {
            extract_params_as_variables(&f.params, variables);
            if let Some(body) = &f.body {
                walk_stmts_for_variables(&body.stmts, arena, variables);
            }
        }
        Decl::Class(c) => {
            for member in &c.body {
                match member {
                    ClassMember::Method(m) => {
                        extract_params_as_variables(&m.function.params, variables);
                        if let Some(body) = &m.function.body {
                            walk_stmts_for_variables(&body.stmts, arena, variables);
                        }
                    }
                    ClassMember::Ctor(ctor) => {
                        extract_params_as_variables(&ctor.params, variables);
                        if let Some(body) = &ctor.body {
                            walk_stmts_for_variables(&body.stmts, arena, variables);
                        }
                    }
                    ClassMember::Getter(g) => {
                        if let Some(body) = &g.body {
                            walk_stmts_for_variables(&body.stmts, arena, variables);
                        }
                    }
                    ClassMember::Setter(s) => {
                        extract_params_as_variables(std::slice::from_ref(&s.param), variables);
                        if let Some(body) = &s.body {
                            walk_stmts_for_variables(&body.stmts, arena, variables);
                        }
                    }
                    _ => {}
                }
            }
        }
        Decl::TsModule(m) => {
            walk_stmts_for_variables(&m.body, arena, variables);
        }
        _ => {}
    }
}

fn process_var_decl(decl: &Decl, _arena: &Arena<Expr>, variables: &mut Vec<ParsedVariable>) {
    if let Decl::Var(v) = decl {
        let is_mutable = !matches!(v.kind, AstVarKind::Const | AstVarKind::Using);
        let vk = match v.kind {
            AstVarKind::Var => VarKind::Var,
            AstVarKind::Let => VarKind::Let,
            AstVarKind::Const => VarKind::Const,
            AstVarKind::Using => VarKind::Let,
        };
        for declarator in &v.decls {
            extract_pattern_variables(&declarator.name, vk, is_mutable, variables);
        }
    }
}

fn extract_params_as_variables(params: &[Pat], variables: &mut Vec<ParsedVariable>) {
    for pat in params {
        extract_pattern_variables(pat, VarKind::Parameter, true, variables);
    }
}

/// Destructuring creates a binding for every identifier nested in its pattern.
/// Preserve those bindings rather than collapsing the whole pattern into an
/// anonymous variable, which is essential for rename/reference tools.
fn extract_pattern_variables(
    pat: &Pat,
    kind: VarKind,
    is_mutable: bool,
    variables: &mut Vec<ParsedVariable>,
) {
    match pat {
        Pat::Ident(binding) => push_binding_variable(binding, kind, is_mutable, variables),
        Pat::Object(object) => {
            for property in &object.props {
                match property {
                    ObjectPatProp::KeyValue(property) => {
                        extract_pattern_variables(&property.value, kind, is_mutable, variables);
                    }
                    ObjectPatProp::Shorthand(binding) => {
                        push_binding_variable(binding, kind, is_mutable, variables);
                    }
                    ObjectPatProp::Rest(rest) => {
                        extract_pattern_variables(&rest.arg, kind, is_mutable, variables);
                    }
                }
            }
            if let Some(rest) = &object.rest {
                extract_pattern_variables(&rest.arg, kind, is_mutable, variables);
            }
        }
        Pat::Array(array) => {
            for element in array.elements.iter().flatten() {
                extract_pattern_variables(element, kind, is_mutable, variables);
            }
            if let Some(rest) = &array.rest {
                extract_pattern_variables(&rest.arg, kind, is_mutable, variables);
            }
        }
        Pat::Rest(rest) => extract_pattern_variables(&rest.arg, kind, is_mutable, variables),
        Pat::Assign(assign) => {
            extract_pattern_variables(&assign.left, kind, is_mutable, variables);
        }
        Pat::Expr(_) | Pat::Invalid(_) => {}
    }
}

fn push_binding_variable(
    binding: &BindingIdent,
    kind: VarKind,
    is_mutable: bool,
    variables: &mut Vec<ParsedVariable>,
) {
    if binding.id.name.is_empty() {
        return;
    }
    variables.push(
        ParsedVariable::builder(&binding.id.name, kind)
            .mutable(is_mutable)
            .line(binding.span.start.line)
            .type_kind(
                binding
                    .type_ann
                    .as_ref()
                    .map_or(TypeKind::Unknown, convert_type_ann),
            )
            .build(),
    );
}

/// JS/TS test-function heuristic. Covers the common naming conventions:
/// `testFoo`, `test_foo`, `itWorks`, plus the bare `test`/`it` names used by
/// Jest/Vitest/Mocha. (Call-form tests like `it("name", () => {})` are not
/// function declarations and are handled separately by the call extractor.)
fn is_js_test_fn(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == "test"
        || lower == "it"
        || lower.starts_with("test")
        || lower.starts_with("it_")
        || lower.starts_with("itshould")
}

// ─── extract_facts ───

pub fn extract_facts(program: &Program, arena: &Arena<Expr>) -> ExtractionResult {
    ExtractionResult {
        symbols: extract_symbols(program, arena),
        imports: extract_imports(program, arena),
        calls: extract_calls(program, arena),
        variables: extract_variables(program, arena),
    }
}
