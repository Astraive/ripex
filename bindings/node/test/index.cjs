const test = require("node:test");
const assert = require("node:assert/strict");

const {
  parseSync,
  parse,
  detectLanguage,
  supportedLanguages,
} = require("../index.cjs");

const TYPESCRIPT_SOURCE = `// leading line comment
/* block comment */
import { helper as importedHelper } from "./helper";

export function add(a: number, b: number): number {
  return importedHelper(a + b);
}

const answer = add(1, 2);
`;

function highLevelEnvelope(result) {
  return {
    language: result.language,
    status: result.status,
    completeness: result.completeness,
    truncated: result.truncated,
    effectiveMode: result.effectiveMode,
  };
}

test("parseSync extracts the TypeScript envelope and all fact categories", () => {
  const result = parseSync(TYPESCRIPT_SOURCE, {
    language: "typescript",
    includeAstSummary: true,
  });

  assert.equal(result.language, "typescript");
  assert.equal(result.status, "complete");
  assert.equal(result.completeness, true);
  assert.equal(result.truncated, false);
  assert.equal(typeof result.effectiveMode, "string");
  assert.ok(result.facts && typeof result.facts === "object");
  assert.ok(Array.isArray(result.facts.symbols));
  assert.ok(Array.isArray(result.facts.imports));
  assert.ok(Array.isArray(result.facts.calls));
  assert.ok(Array.isArray(result.facts.variables));
  assert.ok(result.facts.symbols.length > 0, "the exported function should be a symbol");
  assert.ok(Array.isArray(result.diagnostics));
  assert.ok(Array.isArray(result.comments));
  assert.ok(
    result.comments.some(
      (comment) => comment.kind === "line" && comment.text.includes("leading"),
    ),
    "line comments should be exposed",
  );
  assert.ok(
    result.comments.some(
      (comment) => comment.kind === "block" && comment.text.includes("block"),
    ),
    "block comments should be exposed",
  );
  assert.ok(result.astSummary && typeof result.astSummary.kind === "string");
});

test("async parse uses the same high-level envelope as parseSync", async () => {
  const sync = parseSync(TYPESCRIPT_SOURCE, { language: "typescript" });
  const asyncResult = await parse(TYPESCRIPT_SOURCE, { language: "typescript" });

  assert.deepEqual(highLevelEnvelope(asyncResult), highLevelEnvelope(sync));
  assert.equal(asyncResult.facts.symbols.length, sync.facts.symbols.length);
  assert.equal(asyncResult.facts.imports.length, sync.facts.imports.length);
  assert.equal(asyncResult.facts.calls.length, sync.facts.calls.length);
  assert.equal(asyncResult.facts.variables.length, sync.facts.variables.length);
});

test("language detection and supported language discovery are canonical and sorted", () => {
  assert.equal(detectLanguage("src/index.ts"), "typescript");
  assert.equal(detectLanguage("header.h"), null);
  assert.deepEqual(supportedLanguages(), [
    "c",
    "cpp",
    "csharp",
    "go",
    "javascript",
    "python",
    "rust",
    "typescript",
  ]);
});

test("invalid language selectors throw synchronously and reject asynchronously", async () => {
  assert.throws(
    () => parseSync("const value = 1;", { language: "kotlin" }),
    /unknown|unsupported|unavailable|language/i,
  );
  await assert.rejects(
    () => parse("const value = 1;", { language: "kotlin" }),
    /unknown|unsupported|unavailable|language/i,
  );
});

test("malformed input is recovered with diagnostics instead of throwing", () => {
  const result = parseSync("export const = 1;", { language: "typescript" });

  assert.equal(result.status, "recovered");
  assert.equal(result.completeness, false);
  assert.ok(result.diagnostics.length > 0);
  assert.ok(result.diagnostics.every((diagnostic) => {
    return typeof diagnostic.code === "string"
      && typeof diagnostic.message === "string"
      && diagnostic.span
      && diagnostic.span.start
      && diagnostic.span.end;
  }));
});

test("input over the native one MiB limit returns a limit diagnostic", () => {
  const result = parseSync("x".repeat(1_048_577), { language: "javascript" });

  assert.equal(result.status, "limit_exceeded");
  assert.equal(result.truncated, true);
  assert.ok(
    result.diagnostics.some((diagnostic) => diagnostic.code === "input_too_large"),
    "the input_too_large diagnostic should identify the boundary failure",
  );
});
