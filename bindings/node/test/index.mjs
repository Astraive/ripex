import test from "node:test";
import assert from "node:assert/strict";

import {
  parse,
  parseSync,
  detectLanguage,
  supportedLanguages,
} from "../index.js";

test("ESM loader exposes named exports and parses a JavaScript module", async () => {
  assert.equal(typeof parse, "function");
  assert.equal(typeof parseSync, "function");
  assert.equal(typeof detectLanguage, "function");
  assert.equal(typeof supportedLanguages, "function");

  const source = `// module comment
export const answer = 40 + 2;
`;
  const result = await parse(source, {
    language: "javascript",
    extension: "mjs",
    includeAstSummary: true,
  });

  assert.equal(result.language, "javascript");
  assert.equal(result.status, "complete");
  assert.equal(result.completeness, true);
  assert.equal(result.truncated, false);
  assert.ok(Array.isArray(result.diagnostics));
  assert.ok(Array.isArray(result.comments));
  assert.ok(Array.isArray(result.facts.symbols));
  assert.ok(Array.isArray(result.facts.imports));
  assert.ok(Array.isArray(result.facts.calls));
  assert.ok(Array.isArray(result.facts.variables));
  assert.ok(result.facts.symbols.length > 0);
  assert.ok(result.astSummary && typeof result.astSummary.kind === "string");
});
