use crate::js::ast::*;
use crate::js::config::ParserOptions;
use crate::js::parser;

fn parse(
    src: &str,
) -> (
    Program,
    Vec<crate::diagnostics::ParseError>,
    crate::arena::Arena<Expr>,
) {
    let options = ParserOptions::default();
    parser::parse_program(src, &options)
}

fn parse_module_src(
    src: &str,
) -> (
    Module,
    Vec<crate::diagnostics::ParseError>,
    crate::arena::Arena<Expr>,
) {
    let options = ParserOptions::module();
    parser::parse_module(src, &options)
}

#[test]
fn test_parse_empty_script() {
    let (program, _, _) = parse("");
    match program {
        Program::Script(s) => assert!(s.body.is_empty()),
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_empty_module() {
    let (module, _, _) = parse_module_src("");
    assert!(module.body.is_empty());
}

#[test]
fn test_parse_var_declaration() {
    let (program, _, _ast) = parse("var x = 42;");
    match program {
        Program::Script(script) => {
            assert_eq!(script.body.len(), 1);
            match &script.body[0] {
                Stmt::Decl(Decl::Var(var_decl)) => {
                    assert_eq!(var_decl.kind, VarKind::Var);
                    assert_eq!(var_decl.decls.len(), 1);
                    match &var_decl.decls[0].name {
                        Pat::Ident(bi) => assert_eq!(bi.id.name, "x"),
                        _ => panic!("expected ident pattern"),
                    }
                    assert!(var_decl.decls[0].init.is_some());
                }
                _ => panic!("expected var declaration"),
            }
        }
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_let_declaration() {
    let (program, _, _) = parse("let y = 10;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => {
                assert_eq!(var_decl.kind, VarKind::Let);
            }
            _ => panic!("expected let declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_const_declaration() {
    let (program, _, _) = parse("const z = true;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => {
                assert_eq!(var_decl.kind, VarKind::Const);
            }
            _ => panic!("expected const declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_function() {
    let (program, _, _) = parse("function foo() {}");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Fn(fn_decl)) => {
                assert_eq!(fn_decl.id.name, "foo");
            }
            _ => panic!("expected function declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_arrow_function() {
    let (program, _, ast) = parse("const add = (a, b) => a + b;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => {
                let init = var_decl.decls[0].init.unwrap();
                match &ast[init] {
                    Expr::Arrow(arrow) => {
                        assert!(!arrow.async_);
                        assert_eq!(arrow.params.len(), 2);
                    }
                    _ => panic!("expected arrow function"),
                }
            }
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_class() {
    let (program, _, _) = parse("class MyClass {}");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Class(class)) => {
                assert_eq!(class.id.name, "MyClass");
            }
            _ => panic!("expected class declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_class_with_extends() {
    let (program, _, _) = parse("class Child extends Parent {}");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Class(class)) => {
                assert_eq!(class.id.name, "Child");
                assert!(class.super_class.is_some());
            }
            _ => panic!("expected class declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_if_statement() {
    let (program, _, ast) = parse("if (true) { }");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::If(if_stmt) => match &ast[if_stmt.test] {
                Expr::Lit(Lit::Bool(b)) => assert!(b.value),
                _ => panic!("expected bool literal"),
            },
            _ => panic!("expected if statement"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_if_else() {
    let (program, _, _) = parse("if (x) { a; } else { b; }");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::If(if_stmt) => {
                assert!(if_stmt.alternate.is_some());
            }
            _ => panic!("expected if statement"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_for_statement() {
    let (program, _, _) = parse("for (;;) { }");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::For(for_stmt) => {
                assert!(for_stmt.init.is_none());
                assert!(for_stmt.test.is_none());
                assert!(for_stmt.update.is_none());
            }
            _ => panic!("expected for statement"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_while_statement() {
    let (program, _, ast) = parse("while (true) { break; }");
    match program {
        Program::Script(script) => {
            assert_eq!(script.body.len(), 1);
            match &script.body[0] {
                Stmt::While(while_stmt) => {
                    match &ast[while_stmt.test] {
                        Expr::Lit(Lit::Bool(b)) => assert!(b.value),
                        _ => panic!("expected bool literal"),
                    }
                    match &*while_stmt.body {
                        Stmt::Block(block) => {
                            assert_eq!(block.stmts.len(), 1);
                            match &block.stmts[0] {
                                Stmt::Break(b) => assert!(b.label.is_none()),
                                _ => panic!("expected break statement"),
                            }
                        }
                        _ => panic!("expected block statement"),
                    }
                }
                _ => panic!("expected while statement"),
            }
        }
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_try_catch() {
    let (program, _, _) = parse("try { } catch (e) { }");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Try(try_stmt) => {
                assert!(try_stmt.handler.is_some());
                assert!(try_stmt.finalizer.is_none());
            }
            _ => panic!("expected try statement"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_try_finally() {
    let (program, _, _) = parse("try { } finally { }");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Try(try_stmt) => {
                assert!(try_stmt.handler.is_none());
                assert!(try_stmt.finalizer.is_some());
            }
            _ => panic!("expected try statement"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_binary_expression() {
    let (program, _, ast) = parse("let x = a + b;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => {
                let init = var_decl.decls[0].init.unwrap();
                match &ast[init] {
                    Expr::Binary(bin) => {
                        assert_eq!(bin.op, BinaryOp::Plus);
                    }
                    _ => panic!("expected binary expression"),
                }
            }
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_unary_expression() {
    let (program, _, ast) = parse("let x = !true;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => {
                let init = var_decl.decls[0].init.unwrap();
                match &ast[init] {
                    Expr::Unary(unary) => {
                        assert_eq!(unary.op, UnaryOp::Not);
                    }
                    _ => panic!("expected unary expression"),
                }
            }
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_object_literal() {
    let (program, _, ast) = parse("let x = { a: 1, b: 2 };");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => {
                let init = var_decl.decls[0].init.unwrap();
                match &ast[init] {
                    Expr::Object(obj) => {
                        assert_eq!(obj.props.len(), 2);
                    }
                    _ => panic!("expected object literal"),
                }
            }
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_array_literal() {
    let (program, _, ast) = parse("let x = [1, 2, 3];");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => {
                let init = var_decl.decls[0].init.unwrap();
                match &ast[init] {
                    Expr::Array(arr) => {
                        assert_eq!(arr.elements.len(), 3);
                    }
                    _ => panic!("expected array literal"),
                }
            }
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_return_statement() {
    let (program, _, _) = parse("function f() { return 42; }");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Fn(fn_decl)) => match &fn_decl.body {
                Some(body) => match &body.stmts[0] {
                    Stmt::Return(ret) => {
                        assert!(ret.arg.is_some());
                    }
                    _ => panic!("expected return statement"),
                },
                None => panic!("expected function body"),
            },
            _ => panic!("expected function declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_switch_statement() {
    let (program, _, _) = parse("switch (x) { case 1: break; default: break; }");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Switch(switch) => {
                assert_eq!(switch.cases.len(), 2);
                assert!(switch.cases[0].test.is_some());
                assert!(switch.cases[1].test.is_none());
            }
            _ => panic!("expected switch statement"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_debugger() {
    let (program, _, _) = parse("debugger;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Debugger(_) => {}
            _ => panic!("expected debugger statement"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_empty_statement() {
    let (program, _, _) = parse(";");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Empty(_) => {}
            _ => panic!("expected empty statement"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_labeled_statement() {
    let (program, _, _) = parse("label: ;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Labelled(labeled) => {
                assert_eq!(labeled.label.name, "label");
            }
            _ => panic!("expected labeled statement"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_this_expression() {
    let (program, _, ast) = parse("let x = this;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => {
                let init = var_decl.decls[0].init.unwrap();
                match &ast[init] {
                    Expr::This(_) => {}
                    _ => panic!("expected this expression"),
                }
            }
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_destructuring() {
    let (program, _, _) = parse("let { a, b } = obj;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => match &var_decl.decls[0].name {
                Pat::Object(obj_pat) => {
                    assert_eq!(obj_pat.props.len(), 2);
                }
                _ => panic!("expected object pattern"),
            },
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_array_destructuring() {
    let (program, _, _) = parse("let [a, b] = arr;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => match &var_decl.decls[0].name {
                Pat::Array(arr_pat) => {
                    assert_eq!(arr_pat.elements.len(), 2);
                }
                _ => panic!("expected array pattern"),
            },
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_rest_destructuring() {
    let (program, _, _) = parse("let [a, ...rest] = arr;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => match &var_decl.decls[0].name {
                Pat::Array(arr_pat) => {
                    assert!(arr_pat.rest.is_some());
                }
                _ => panic!("expected array pattern with rest"),
            },
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_multi_expression_statement() {
    let (program, _, _) = parse("a; b; c;");
    match program {
        Program::Script(script) => {
            assert_eq!(script.body.len(), 3);
        }
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_boolean_literals() {
    let (program, _, _) = parse("let t = true; let f = false;");
    match program {
        Program::Script(script) => {
            assert_eq!(script.body.len(), 2);
        }
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_null_literal() {
    let (program, _, ast) = parse("let x = null;");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => {
                let init = var_decl.decls[0].init.unwrap();
                match &ast[init] {
                    Expr::Lit(Lit::Null(_)) => {}
                    _ => panic!("expected null literal"),
                }
            }
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_parse_import() {
    let (module, _, _) = parse_module_src("import { foo } from './bar';");
    match &module.body[0] {
        ModuleItem::Import(imp) => {
            assert_eq!(imp.source.value, "./bar");
            assert_eq!(imp.specifiers.len(), 1);
            match &imp.specifiers[0] {
                ImportSpecifier::Named(n) => {
                    assert_eq!(n.local.name, "foo");
                }
                _ => panic!("expected named import"),
            }
        }
        _ => panic!("expected import declaration"),
    }
}

#[test]
fn test_parse_export_default() {
    let (module, _, ast) = parse_module_src("export default 42;");
    match &module.body[0] {
        ModuleItem::Export(ExportDecl::Default(exp)) => match ast.get(exp.decl) {
            Expr::Lit(Lit::Num(n)) => {
                assert_eq!(n.value, 42.0);
            }
            _ => panic!("expected number literal"),
        },
        _ => panic!("expected export default"),
    }
}

#[test]
fn test_parse_export_named() {
    let (module, _, _) = parse_module_src("export { foo, bar };");
    match &module.body[0] {
        ModuleItem::Export(ExportDecl::Named(exp)) => {
            assert_eq!(exp.specifiers.len(), 2);
        }
        _ => panic!("expected named export"),
    }
}

#[test]
fn test_parse_export_all() {
    let (module, _, _) = parse_module_src("export * from './lib';");
    match &module.body[0] {
        ModuleItem::Export(ExportDecl::All(exp)) => {
            assert_eq!(exp.source.value, "./lib");
        }
        _ => panic!("expected export all"),
    }
}

#[test]
fn test_parse_new_expression() {
    let (program, _, ast) = parse("let x = new Foo();");
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(var_decl)) => {
                let init = var_decl.decls[0].init.unwrap();
                match &ast[init] {
                    Expr::New(new_expr) => match &ast[new_expr.callee] {
                        Expr::Ident(id) => assert_eq!(id.name, "Foo"),
                        _ => panic!("expected Foo ident"),
                    },
                    _ => panic!("expected new expression"),
                }
            }
            _ => panic!("expected variable declaration"),
        },
        _ => panic!("expected script"),
    }
}

fn parse_with_features(
    src: &str,
    features: &[&str],
) -> (
    Program,
    Vec<crate::diagnostics::ParseError>,
    crate::arena::Arena<Expr>,
) {
    let mut options = ParserOptions::default();
    for f in features {
        match *f {
            "explicit_resource_management" => options.features.explicit_resource_management = true,
            "import_attributes" => options.features.import_attributes = true,
            _ => {}
        }
    }
    parser::parse_program(src, &options)
}

fn parse_module_with_features(
    src: &str,
    features: &[&str],
) -> (
    Module,
    Vec<crate::diagnostics::ParseError>,
    crate::arena::Arena<Expr>,
) {
    let mut options = ParserOptions::module();
    for f in features {
        match *f {
            "explicit_resource_management" => options.features.explicit_resource_management = true,
            "import_attributes" => options.features.import_attributes = true,
            _ => {}
        }
    }
    parser::parse_module(src, &options)
}

#[test]
fn test_using_declaration() {
    let (program, _, _) =
        parse_with_features("using x = resource();", &["explicit_resource_management"]);
    match program {
        Program::Script(script) => {
            assert_eq!(script.body.len(), 1);
            match &script.body[0] {
                Stmt::Decl(Decl::Var(var_decl)) => {
                    assert_eq!(var_decl.kind, VarKind::Using);
                    assert!(!var_decl.await_);
                    assert_eq!(var_decl.decls.len(), 1);
                    match &var_decl.decls[0].name {
                        Pat::Ident(bi) => assert_eq!(bi.id.name, "x"),
                        _ => panic!("expected ident pattern"),
                    }
                }
                _ => panic!("expected using declaration"),
            }
        }
        _ => panic!("expected script"),
    }
}

#[test]
fn test_await_using_declaration() {
    let (program, _, _) = parse_with_features(
        "await using x = asyncResource();",
        &["explicit_resource_management"],
    );
    match program {
        Program::Script(script) => {
            assert_eq!(script.body.len(), 1);
            match &script.body[0] {
                Stmt::Decl(Decl::Var(var_decl)) => {
                    assert_eq!(var_decl.kind, VarKind::Using);
                    assert!(var_decl.await_);
                    assert_eq!(var_decl.decls.len(), 1);
                }
                _ => panic!("expected using declaration"),
            }
        }
        _ => panic!("expected script"),
    }
}

#[test]
fn test_using_multiple_declarators() {
    let (program, _, _) =
        parse_with_features("using x = a, y = b;", &["explicit_resource_management"]);
    match program {
        Program::Script(script) => {
            assert_eq!(script.body.len(), 1);
            match &script.body[0] {
                Stmt::Decl(Decl::Var(var_decl)) => {
                    assert_eq!(var_decl.kind, VarKind::Using);
                    assert_eq!(var_decl.decls.len(), 2);
                }
                _ => panic!("expected using declaration"),
            }
        }
        _ => panic!("expected script"),
    }
}

#[test]
fn test_hashbang_script() {
    let (program, _, _) = parse("#!/usr/bin/env node\nvar x = 1;");
    match program {
        Program::Script(script) => {
            assert_eq!(script.body.len(), 1);
            match &script.body[0] {
                Stmt::Decl(Decl::Var(_)) => {}
                _ => panic!("expected var declaration"),
            }
        }
        _ => panic!("expected script"),
    }
}

#[test]
fn test_hashbang_module() {
    let (module, _, _) = parse_module_src("#!/usr/bin/env node\nimport { x } from 'foo';");
    assert_eq!(module.body.len(), 1);
}

#[test]
fn test_import_assertions() {
    let (module, _, _) = parse_module_with_features(
        "import json from './foo.json' assert { type: 'json' };",
        &["import_attributes"],
    );
    assert_eq!(module.body.len(), 1);
    match &module.body[0] {
        ModuleItem::Import(import_decl) => {
            assert_eq!(import_decl.assertions.len(), 1);
            assert_eq!(import_decl.source.value, "./foo.json");
            match &import_decl.assertions[0].key {
                ImportAttributeKey::Ident(id) => assert_eq!(id.name, "type"),
                _ => panic!("expected ident key"),
            }
            assert_eq!(import_decl.assertions[0].value.value, "json");
        }
        _ => panic!("expected import declaration"),
    }
}

#[test]
fn test_import_assertions_multiple() {
    let (module, _, _) = parse_module_with_features(
        "import data from './data.json' assert { type: 'json', version: '2' };",
        &["import_attributes"],
    );
    match &module.body[0] {
        ModuleItem::Import(import_decl) => {
            assert_eq!(import_decl.assertions.len(), 2);
            assert_eq!(import_decl.source.value, "./data.json");
        }
        _ => panic!("expected import declaration"),
    }
}

#[test]
fn test_standard_import_attributes_with_syntax() {
    let (module, errors, _) = parse_module_with_features(
        "import data from './data.json' with { type: 'json', mode: 'strict' };",
        &["import_attributes"],
    );
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    match &module.body[0] {
        ModuleItem::Import(import_decl) => {
            assert_eq!(import_decl.source.value, "./data.json");
            assert_eq!(import_decl.assertions.len(), 2);
        }
        _ => panic!("expected import declaration"),
    }
}

#[test]
fn test_typescript_type_imports_and_reexports_preserve_facts() {
    let mut options = ParserOptions::module();
    crate::js::config::ParserPlugins::typescript().apply(&mut options);
    let source = r#"
        import type { User as UserModel } from "./models";
        import { type Config, run as execute } from "./runtime";
        export type { User as PublicUser } from "./models";
        export { type Config as PublicConfig, run as publicRun } from "./runtime";
        export type * from "./all-types";
    "#;
    let (program, errors, ast) = parser::parse_program(source, &options);
    assert!(errors.is_empty(), "parse errors: {errors:?}");

    let facts = crate::js::facts::extract_facts(&program, &ast);
    assert!(facts.imports.iter().any(|import| {
        import.kind == crate::facts::ImportKind::TypeImport
            && import.source == "./models"
            && import.imported_name.as_deref() == Some("User")
            && import.local_name.as_deref() == Some("UserModel")
            && import.is_type_only
    }));
    assert!(facts.imports.iter().any(|import| {
        import.kind == crate::facts::ImportKind::TypeImport
            && import.source == "./runtime"
            && import.imported_name.as_deref() == Some("Config")
            && import.is_type_only
    }));
    assert!(facts.imports.iter().any(|import| {
        import.kind == crate::facts::ImportKind::NamedImport
            && import.source == "./runtime"
            && import.imported_name.as_deref() == Some("run")
            && import.local_name.as_deref() == Some("execute")
            && !import.is_type_only
    }));
    assert!(facts.imports.iter().any(|import| {
        import.kind == crate::facts::ImportKind::TypeReExport
            && import.source == "./models"
            && import.specifiers[0].imported == "User"
            && import.specifiers[0].local == "PublicUser"
            && import.is_type_only
    }));
    assert!(facts.imports.iter().any(|import| {
        import.kind == crate::facts::ImportKind::ReExport
            && import.source == "./runtime"
            && import
                .specifiers
                .iter()
                .any(|specifier| specifier.imported == "run" && specifier.local == "publicRun")
            && !import.is_type_only
    }));
    assert!(facts.imports.iter().any(|import| {
        import.kind == crate::facts::ImportKind::TypeReExport
            && import.source == "./all-types"
            && import.is_star_import
            && import.is_type_only
    }));
}

#[test]
fn test_decorators_on_class() {
    let (program, errors, _) = parse("@sealed class Foo {}");
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Class(class)) => {
                assert_eq!(class.decorators.len(), 1);
            }
            _ => panic!("expected class declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_decorators_on_method_and_field() {
    let (program, errors, _) = parse("@logger class Foo { @bound foo() {} @readonly bar = 1; }");
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Class(class)) => {
                assert_eq!(class.body.len(), 2);
                match &class.body[0] {
                    ClassMember::Method(m) => assert_eq!(m.decorators.len(), 1),
                    _ => panic!("expected method"),
                }
                match &class.body[1] {
                    ClassMember::Prop(p) => assert_eq!(p.decorators.len(), 1),
                    _ => panic!("expected property"),
                }
            }
            _ => panic!("expected class declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_decorators_on_function() {
    let (program, errors, _) = parse("@wrap function foo() {}");
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Fn(f)) => assert_eq!(f.decorators.len(), 1),
            _ => panic!("expected function declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_pipeline_operator() {
    let (program, errors, ast) = parse("const y = x |> double |> inc;");
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(v)) => {
                let init = v.decls[0].init.expect("init");
                match &ast[init] {
                    Expr::Pipeline(p) => {
                        // outermost pipeline: input is `x |> double`, body is `inc`
                        assert!(matches!(ast[p.input], Expr::Pipeline(_)));
                        assert!(matches!(ast[p.body], Expr::Ident(_)));
                    }
                    other => panic!("expected pipeline, got {:?}", other),
                }
            }
            _ => panic!("expected var declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_record_literal() {
    let (program, errors, ast) = parse("const r = #{ a: 1, b };");
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(v)) => {
                let init = v.decls[0].init.expect("init");
                assert!(matches!(ast[init], Expr::Record(_)), "expected record");
            }
            _ => panic!("expected var declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_tuple_literal() {
    let (program, errors, ast) = parse("const t = #[1, 2, 3];");
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    match program {
        Program::Script(script) => match &script.body[0] {
            Stmt::Decl(Decl::Var(v)) => {
                let init = v.decls[0].init.expect("init");
                match &ast[init] {
                    Expr::Tuple(t) => assert_eq!(t.elements.len(), 3),
                    other => panic!("expected tuple, got {:?}", other),
                }
            }
            _ => panic!("expected var declaration"),
        },
        _ => panic!("expected script"),
    }
}

#[test]
fn test_malformed_string_escapes_no_panic() {
    for s in [
        r#""\uZZ""#,
        r#""\xZZ""#,
        r#""\u{""#,
        r#""\u{G}"#,
        r#""\u{110000}"#,
    ] {
        let _ = parse(s);
    }
}

#[test]
fn test_pipeline_calls_extracted() {
    let (program, _, ast) = parse("const y = x |> double |> inc;");
    let calls = crate::js::facts::extract_calls(&program, &ast);
    let names: Vec<&str> = calls.iter().map(|c| c.callee_text.as_str()).collect();
    assert!(
        names.contains(&"double") || names.contains(&"inc"),
        "pipeline body call not extracted: {:?}",
        names
    );
}

#[test]
fn test_template_literal_interpolations() {
    let (_, errors, _) = parse("const message = `a=${first(1)}, b=${second(2)}`;");
    assert!(errors.is_empty(), "parse errors: {errors:?}");
}

#[test]
fn test_class_accessors_and_static_block() {
    let source = "class Model { static { Model.count = 0; } get value() { return 1; } set value(next) { this._value = next; } }";
    let (_, errors, _) = parse(source);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
}

#[test]
fn test_parenthesized_conditional_expression() {
    let (_, errors, _) = parse("const max = (a, b) => (a > b ? a : b);");
    assert!(errors.is_empty(), "parse errors: {errors:?}");
}

#[test]
fn test_typescript_generic_class() {
    let mut options = ParserOptions::module();
    crate::js::config::ParserPlugins::typescript().apply(&mut options);
    let (_, errors, _) = parser::parse_program(
        "export class Repository<T extends Identifiable> { get(id: number): T | undefined { return this.items.get(id); } }",
        &options,
    );
    assert!(errors.is_empty(), "parse errors: {errors:?}");
}

#[test]
fn test_tsx_typed_destructuring_and_nested_elements() {
    let mut options = ParserOptions::module();
    crate::js::config::ParserPlugins::all_ts().apply(&mut options);
    let source = "function Header({ title, count = 0 }: Props) { return (<header><h1>{title}</h1><span>{count}</span></header>); }";
    let (_, errors, _) = parser::parse_program(source, &options);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
}

#[test]
fn test_typescript_optional_calls_and_destructuring_facts() {
    let mut options = ParserOptions::module();
    crate::js::config::ParserPlugins::all_ts().apply(&mut options);
    let source = r#"
        const { account: { id }, role = "guest", ...rest } = input;
        let [first, , { name }, ...tail] = values;
        const response = await client?.fetch<Response>?.("/api");
        const feature = await import("./feature.js");
        const output = value |> transform();
    "#;
    let (program, errors, ast) = parser::parse_program(source, &options);
    assert!(errors.is_empty(), "parse errors: {errors:?}");

    let facts = crate::js::facts::extract_facts(&program, &ast);
    let names: Vec<&str> = facts
        .variables
        .iter()
        .map(|variable| variable.name.as_str())
        .collect();
    for expected in [
        "id", "role", "rest", "first", "name", "tail", "response", "feature", "output",
    ] {
        assert!(
            names.contains(&expected),
            "missing destructured binding {expected}: {names:?}"
        );
    }
    assert!(facts
        .variables
        .iter()
        .find(|variable| variable.name == "first")
        .is_some_and(|variable| variable.is_mutable));

    let fetch = facts
        .calls
        .iter()
        .find(|call| call.callee_text == "client.fetch")
        .unwrap_or_else(|| {
            panic!(
                "optional generic call facts: {:#?}\narena: {ast:#?}",
                facts.calls
            )
        });
    assert!(fetch.is_optional);
    assert!(fetch.is_await);
    assert_eq!(
        fetch.type_args,
        vec![crate::facts::TypeKind::simple("Response")]
    );
    assert_eq!(
        facts
            .calls
            .iter()
            .filter(|call| call.callee_text == "transform")
            .count(),
        1,
        "pipeline calls must not be duplicated"
    );
    assert!(facts.imports.iter().any(|import| {
        import.kind == crate::facts::ImportKind::DynamicImport && import.source == "./feature.js"
    }));
}

#[test]
fn token_limit_is_reported_by_all_js_entrypoints() {
    let source = "x;".repeat(crate::limits::MAX_TOKENS / 2 + 2);
    assert!(source.len() < crate::limits::MAX_INPUT_SIZE);

    let options = ParserOptions::default();
    let (_, program_errors, _) = parser::parse_program(&source, &options);
    assert!(program_errors
        .iter()
        .any(|error| error.code == crate::diagnostics::DiagnosticCode::TokenLimitExceeded));

    let (_, module_errors, _) = parser::parse_module(&source, &options);
    assert!(module_errors
        .iter()
        .any(|error| error.code == crate::diagnostics::DiagnosticCode::TokenLimitExceeded));

    let (_, script_errors, _) = parser::parse_script(&source, &options);
    assert!(script_errors
        .iter()
        .any(|error| error.code == crate::diagnostics::DiagnosticCode::TokenLimitExceeded));
}

#[test]
fn deep_js_prefix_reports_recursion_limit() {
    let source = "!".repeat(crate::limits::MAX_RECURSION as usize + 64) + "x;";
    let options = ParserOptions::default();
    let (_, errors, _) = parser::parse_program(&source, &options);
    assert!(errors
        .iter()
        .any(|error| error.code == crate::diagnostics::DiagnosticCode::MaxRecursionExceeded));
}

#[test]
fn byte_limit_is_enforced_by_all_js_entrypoints() {
    let source = "x".repeat(crate::limits::MAX_INPUT_SIZE + 1);
    let options = ParserOptions::default();

    let (_, program_errors, _) = parser::parse_program(&source, &options);
    assert!(program_errors
        .iter()
        .any(|error| error.code == crate::diagnostics::DiagnosticCode::InputTooLarge));

    let (_, module_errors, _) = parser::parse_module(&source, &options);
    assert!(module_errors
        .iter()
        .any(|error| error.code == crate::diagnostics::DiagnosticCode::InputTooLarge));

    let (_, script_errors, _) = parser::parse_script(&source, &options);
    assert!(script_errors
        .iter()
        .any(|error| error.code == crate::diagnostics::DiagnosticCode::InputTooLarge));
}
