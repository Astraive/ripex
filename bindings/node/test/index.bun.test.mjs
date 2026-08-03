import { test, expect } from "bun:test";

import { parse, parseSync } from "../index.js";

test("Bun loads the ESM native binding and parses JavaScript", async () => {
  const source = "export const answer = 42;";
  const sync = parseSync(source, { language: "javascript", extension: "mjs" });
  const asyncResult = await parse(source, { language: "javascript", extension: "mjs" });

  expect(sync.language).toBe("javascript");
  expect(sync.status).toBe("complete");
  expect(sync.facts.symbols.length).toBeGreaterThan(0);
  expect(asyncResult.language).toBe(sync.language);
  expect(asyncResult.status).toBe(sync.status);
  expect(asyncResult.facts.symbols.length).toBe(sync.facts.symbols.length);
});
