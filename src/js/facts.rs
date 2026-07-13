use super::ast::ImportSpecifier as AstImportSpecifier;
use super::ast::VarKind as AstVarKind;
use super::ast::{
    ClassMember, Decl, ExportDecl, Expr, ExprRef, ForInit, ImportDecl, Lit, ModuleItem, Pat,
    Program, PropName, Stmt, TypeAnn,
};
use super::parser::expressions::HasSpan;
use crate::arena::Arena;
use crate::facts::{
    CallKind, ImportKind, ImportSpecifier, ImportSpecifierKind, ParsedCall, ParsedImport,
    ParsedSymbol, ParsedVariable, SymbolKind, TypeKind, VarKind,
};
use crate::ExtractionResult;

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
        Expr::TSNonNull(e) => expr_name(e.expr, arena),
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

fn extract_pat_name(pat: &Pat) -> Option<&str> {
    match pat {
        Pat::Ident(bi) => Some(&bi.id.name),
        _ => None,
    }
}

fn extract_pat_type_kind(pat: &Pat) -> TypeKind {
    match pat {
        Pat::Ident(bi) => bi
            .type_ann
            .as_ref()
            .map_or(TypeKind::Unknown, convert_type_ann),
        _ => TypeKind::Unknown,
    }
}

fn pat_span(pat: &Pat) -> crate::span::Span {
    match pat {
        Pat::Ident(b) => b.span,
        Pat::Object(o) => o.span,
        Pat::Array(a) => a.span,
        Pat::Rest(r) => r.span,
        Pat::Assign(a) => a.span,
        Pat::Expr(_) => crate::span::Span::ZERO,
        Pat::Invalid(i) => i.span,
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
        Decl::Var(_) => {}
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

pub fn extract_imports(program: &Program, _arena: &Arena<Expr>) -> Vec<ParsedImport> {
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
    imports
}

fn process_import_decl(imp: &ImportDecl, imports: &mut Vec<ParsedImport>) {
    let source = imp.source.value.clone();
    let line = imp.span.start.line;
    for spec in &imp.specifiers {
        match spec {
            AstImportSpecifier::Default(d) => {
                imports.push(
                    ParsedImport::builder(ImportKind::DefaultImport, &source)
                        .local(&d.local.name)
                        .line(line)
                        .build(),
                );
            }
            AstImportSpecifier::Named(n) => {
                let specifiers = vec![ImportSpecifier {
                    imported: n.imported.name.clone(),
                    local: n.local.name.clone(),
                    kind: ImportSpecifierKind::Named,
                }];
                imports.push(
                    ParsedImport::builder(ImportKind::NamedImport, &source)
                        .local(&n.local.name)
                        .imported(&n.imported.name)
                        .line(line)
                        .specifiers(specifiers)
                        .build(),
                );
            }
            AstImportSpecifier::Namespace(ns) => {
                imports.push(
                    ParsedImport::builder(ImportKind::NamespaceImport, &source)
                        .local(&ns.local.name)
                        .line(line)
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
                        kind: ImportSpecifierKind::Named,
                    })
                    .collect();
                imports.push(
                    ParsedImport::builder(ImportKind::ReExport, &source.value)
                        .line(named.span.start.line)
                        .reexport(true)
                        .specifiers(specifiers)
                        .build(),
                );
            }
        }
        ExportDecl::All(all) => {
            imports.push(
                ParsedImport::builder(ImportKind::SideEffectImport, &all.source.value)
                    .line(all.span.start.line)
                    .reexport(true)
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
    for (_id, expr) in arena.iter() {
        push_calls_for_expr(expr, arena, &mut calls);
    }
    calls
}

/// Emits call facts for a single expression node. Used both while walking the
/// arena and to recurse into pipeline bodies.
fn push_calls_for_expr(expr: &Expr, arena: &Arena<Expr>, calls: &mut Vec<ParsedCall>) {
    match expr {
        Expr::Call(call) => {
            let (kind, callee_text) = resolve_callee(call.callee, arena);
            let callee_obj = resolve_callee_object(call.callee, arena);
            calls.push(
                ParsedCall::builder(kind, callee_text)
                    .object(callee_obj.unwrap_or_default())
                    .pos(call.span.start.line, call.span.start.column)
                    .build(),
            );
        }
        Expr::OptionalCall(oc) => {
            let (kind, callee_text) = resolve_callee(oc.callee, arena);
            let callee_obj = resolve_callee_object(oc.callee, arena);
            calls.push(
                ParsedCall::builder(kind, callee_text)
                    .object(callee_obj.unwrap_or_default())
                    .pos(oc.span.start.line, oc.span.start.column)
                    .build(),
            );
        }
        Expr::New(new) => {
            let callee_text = expr_name(new.callee, arena).unwrap_or_else(|| "unknown".into());
            calls.push(
                ParsedCall::builder(CallKind::ConstructorCall, callee_text)
                    .pos(new.span.start.line, new.span.start.column)
                    .build(),
            );
        }
        Expr::TaggedTemplate(tagged) => {
            let callee_text = expr_name(tagged.tag, arena).unwrap_or_else(|| "unknown".into());
            calls.push(
                ParsedCall::builder(CallKind::FunctionCall, callee_text)
                    .pos(tagged.span.start.line, tagged.span.start.column)
                    .build(),
            );
        }
        // A pipeline's body is the function applied to the input. The applied
        // function is a call target (e.g. `x |> double`), so emit it as a call
        // and also walk it in case it is itself a nested call (`x |> foo()`).
        Expr::Pipeline(p) => {
            if let Some(name) = expr_name(p.body, arena) {
                calls.push(
                    ParsedCall::builder(CallKind::FunctionCall, &name)
                        .pos(
                            arena[p.body].span().start.line,
                            arena[p.body].span().start.column,
                        )
                        .build(),
                );
            }
            push_calls_for_expr(&arena[p.body], arena, calls);
        }
        _ => {}
    }
}

fn resolve_callee_object(callee: ExprRef, arena: &Arena<Expr>) -> Option<String> {
    match &arena[callee] {
        Expr::Member(m) => expr_name(m.object, arena),
        _ => None,
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
        let is_mutable = matches!(v.kind, AstVarKind::Var);
        let vk = match v.kind {
            AstVarKind::Var => VarKind::Var,
            AstVarKind::Let => VarKind::Let,
            AstVarKind::Const => VarKind::Const,
            AstVarKind::Using => VarKind::Let,
        };
        for declarator in &v.decls {
            if let Some(name) = extract_pat_name(&declarator.name) {
                let type_kind = extract_pat_type_kind(&declarator.name);
                variables.push(
                    ParsedVariable::builder(name, vk)
                        .mutable(is_mutable)
                        .line(declarator.span.start.line)
                        .type_kind(type_kind)
                        .build(),
                );
            }
        }
    }
}

fn extract_params_as_variables(params: &[Pat], variables: &mut Vec<ParsedVariable>) {
    for pat in params {
        if let Some(name) = extract_pat_name(pat) {
            let type_kind = extract_pat_type_kind(pat);
            variables.push(
                ParsedVariable::builder(name, VarKind::Parameter)
                    .line(pat_span(pat).start.line)
                    .type_kind(type_kind)
                    .build(),
            );
        }
    }
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
