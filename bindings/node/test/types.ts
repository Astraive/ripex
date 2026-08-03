import {
  detectLanguage,
  parse,
  parseSync,
  supportedLanguages,
} from "../index.js";
import type {
  AstSummary,
  Comment,
  Diagnostic,
  Facts,
  ImportSpecifier,
  Language,
  ParsedCall,
  ParsedImport,
  ParsedParam,
  ParsedSymbol,
  ParsedVariable,
  ParseOptions,
  ParseOutput,
  Pos,
  Span,
  TypeKind,
  UsageSite,
} from "../index.js";

const options: ParseOptions = {
  language: "typescript",
  filename: "src/index.ts",
  extension: "ts",
  includeAstSummary: true,
};

const source = "export function add(a: number, b: number): number { return a + b; }";
const syncResult: ParseOutput = parseSync(source, options);
const asyncResult: Promise<ParseOutput> = parse(source, options);
const detected: Language | null = detectLanguage("src/index.ts");
const languages: Language[] = supportedLanguages();

function consumeTypeKind(typeKind: TypeKind): void {
  const kind: string = typeKind.kind;
  const name: string | null = typeKind.name;
  const items: TypeKind[] = typeKind.items;
  void kind;
  void name;
  for (const item of items) consumeTypeKind(item);
}

function consumePosition(position: Pos): void {
  const offset: number = position.offset;
  const line: number = position.line;
  const column: number = position.column;
  void offset;
  void line;
  void column;
}

function consumeSpan(span: Span): void {
  consumePosition(span.start);
  consumePosition(span.end);
}

function consumeDiagnostic(diagnostic: Diagnostic): void {
  const code: string = diagnostic.code;
  const message: string = diagnostic.message;
  consumeSpan(diagnostic.span);
  void code;
  void message;
}

function consumeComment(comment: Comment): void {
  const kind: "line" | "block" | "hashbang" = comment.kind;
  const text: string = comment.text;
  consumeSpan(comment.span);
  void kind;
  void text;
}

function consumeParam(param: ParsedParam): void {
  const name: string = param.name;
  const typeAnnotation: string | null = param.typeAnnotation;
  const defaultValue: string | null = param.defaultValue;
  void name;
  void typeAnnotation;
  void defaultValue;
}

function consumeSymbol(symbol: ParsedSymbol): void {
  const kind: string = symbol.kind;
  const name: string = symbol.name;
  const exported: boolean = symbol.exported;
  const visibility: string = symbol.visibility;
  const lineStart: number = symbol.lineStart;
  const lineEnd: number = symbol.lineEnd;
  const signature: string = symbol.signature;
  const isTest: boolean = symbol.isTest;
  const isAsync: boolean = symbol.isAsync;
  const returnType: string | null = symbol.returnType;
  const isConstructor: boolean = symbol.isConstructor;
  const isDestructor: boolean = symbol.isDestructor;
  const isVirtual: boolean = symbol.isVirtual;
  const isOverride: boolean = symbol.isOverride;
  const isAbstract: boolean = symbol.isAbstract;
  const isStatic: boolean = symbol.isStatic;
  const isConstexpr: boolean = symbol.isConstexpr;
  const isFinal: boolean = symbol.isFinal;
  const storageClass: string = symbol.storageClass;
  const templateParams: string[] = symbol.templateParams;
  const attributes: string[] = symbol.attributes;
  const baseClasses: string[] = symbol.baseClasses;
  const docString: string | null = symbol.docString;
  consumeTypeKind(symbol.typeKind);
  for (const param of symbol.params) consumeParam(param);
  void kind;
  void name;
  void exported;
  void visibility;
  void lineStart;
  void lineEnd;
  void signature;
  void isTest;
  void isAsync;
  void returnType;
  void isConstructor;
  void isDestructor;
  void isVirtual;
  void isOverride;
  void isAbstract;
  void isStatic;
  void isConstexpr;
  void isFinal;
  void storageClass;
  void templateParams;
  void attributes;
  void baseClasses;
  void docString;
}

function consumeImportSpecifier(specifier: ImportSpecifier): void {
  const imported: string = specifier.imported;
  const local: string = specifier.local;
  const kind: string = specifier.kind;
  void imported;
  void local;
  void kind;
}

function consumeImport(parsedImport: ParsedImport): void {
  const kind: string = parsedImport.kind;
  const source: string = parsedImport.source;
  const localName: string | null = parsedImport.localName;
  const importedName: string | null = parsedImport.importedName;
  const line: number = parsedImport.line;
  const isTypeOnly: boolean = parsedImport.isTypeOnly;
  const isReexport: boolean = parsedImport.isReexport;
  const specifiers: ImportSpecifier[] = parsedImport.specifiers;
  const isStarImport: boolean = parsedImport.isStarImport;
  const modulePath: string[] = parsedImport.modulePath;
  for (const specifier of specifiers) consumeImportSpecifier(specifier);
  void kind;
  void source;
  void localName;
  void importedName;
  void line;
  void isTypeOnly;
  void isReexport;
  void isStarImport;
  void modulePath;
}

function consumeCall(call: ParsedCall): void {
  const kind: string = call.kind;
  const calleeText: string = call.calleeText;
  const object: string | null = call.object;
  const line: number = call.line;
  const column: number = call.column;
  const isAwait: boolean = call.isAwait;
  const isOptional: boolean = call.isOptional;
  const typeArgs: TypeKind[] = call.typeArgs;
  for (const typeArg of typeArgs) consumeTypeKind(typeArg);
  void kind;
  void calleeText;
  void object;
  void line;
  void column;
  void isAwait;
  void isOptional;
}

function consumeUsage(usage: UsageSite): void {
  const line: number = usage.line;
  const column: number = usage.column;
  const usageKind: string = usage.usageKind;
  void line;
  void column;
  void usageKind;
}

function consumeVariable(variable: ParsedVariable): void {
  const name: string = variable.name;
  const kind: string = variable.kind;
  const typeAnnotation: string | null = variable.typeAnnotation;
  const isMutable: boolean = variable.isMutable;
  const lineDef: number = variable.lineDef;
  const scopeSymbol: string | null = variable.scopeSymbol;
  const scopeStart: number = variable.scopeStart;
  const scopeEnd: number = variable.scopeEnd;
  const usageSites: UsageSite[] = variable.usageSites;
  const storageClass: string = variable.storageClass;
  const isConstructor: boolean = variable.isConstructor;
  const isDestructor: boolean = variable.isDestructor;
  const isImported: boolean = variable.isImported;
  const isStatic: boolean = variable.isStatic;
  const isConstexpr: boolean = variable.isConstexpr;
  const isThreadLocal: boolean = variable.isThreadLocal;
  const isExtern: boolean = variable.isExtern;
  consumeTypeKind(variable.typeKind);
  for (const usage of usageSites) consumeUsage(usage);
  void name;
  void kind;
  void typeAnnotation;
  void isMutable;
  void lineDef;
  void scopeSymbol;
  void scopeStart;
  void scopeEnd;
  void storageClass;
  void isConstructor;
  void isDestructor;
  void isImported;
  void isStatic;
  void isConstexpr;
  void isThreadLocal;
  void isExtern;
}

function consumeFacts(facts: Facts): void {
  for (const symbol of facts.symbols) consumeSymbol(symbol);
  for (const parsedImport of facts.imports) consumeImport(parsedImport);
  for (const call of facts.calls) consumeCall(call);
  for (const variable of facts.variables) consumeVariable(variable);
}

function consumeAstSummary(astSummary: AstSummary | null): void {
  if (astSummary === null) return;
  const kind: string = astSummary.kind;
  const topLevelNodes: number = astSummary.topLevelNodes;
  const expressionNodes: number | null = astSummary.expressionNodes;
  void kind;
  void topLevelNodes;
  void expressionNodes;
}

function consumeOutput(output: ParseOutput): void {
  const language: Language = output.language;
  const status: "complete" | "recovered" | "limit_exceeded" | "failed" = output.status;
  const completeness: boolean = output.completeness;
  const truncated: boolean = output.truncated;
  const effectiveMode: string = output.effectiveMode;
  for (const diagnostic of output.diagnostics) consumeDiagnostic(diagnostic);
  for (const comment of output.comments) consumeComment(comment);
  consumeFacts(output.facts);
  consumeAstSummary(output.astSummary);
  void language;
  void status;
  void completeness;
  void truncated;
  void effectiveMode;
}

consumeOutput(syncResult);
void asyncResult;
void detected;
void languages;
