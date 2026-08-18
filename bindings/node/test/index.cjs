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

test("parses JavaScript source and extracts facts", () => {
  const jsSource = `// JS line comment
import { compute } from "./math.js";
export class Calculator {
  calculate(val) {
    return compute(val);
  }
}
const instance = new Calculator();
const output = instance.calculate(42);
`;
  const result = parseSync(jsSource, { language: "javascript", extension: "js", includeAstSummary: true });
  assert.equal(result.language, "javascript");
  assert.equal(result.status, "complete");
  assert.ok(result.facts.symbols.length > 0);
  assert.ok(result.facts.imports.length > 0);
  assert.ok(result.facts.calls.length > 0);
  assert.ok(result.facts.variables.length > 0);
  assert.ok(result.comments.some((c) => c.text.includes("JS line comment")));
  assert.ok(result.astSummary);
});

test("parses Python source and extracts facts", () => {
  const pySource = `import os
from math import sqrt

class Geometry:
    def distance(self, x: float, y: float) -> float:
        return sqrt(x * x + y * y)

geo = Geometry()
res = geo.distance(3.0, 4.0)
`;
  const result = parseSync(pySource, { language: "python", includeAstSummary: true });
  assert.equal(result.language, "python");
  assert.equal(result.status, "complete");
  assert.ok(result.facts.symbols.length > 0);
  assert.ok(result.facts.imports.length > 0);
  assert.ok(result.facts.calls.length > 0);
  assert.ok(result.facts.variables.length > 0);
  assert.ok(result.astSummary);
});

test("parses Go source and extracts facts", () => {
  const goSource = `package main

import "fmt"

type Service struct {
    Port int
}

func (s *Service) Start() {
    fmt.Println(s.Port)
}

func main() {
    svc := &Service{Port: 8080}
    svc.Start()
}
`;
  const result = parseSync(goSource, { language: "go", includeAstSummary: true });
  assert.equal(result.language, "go");
  assert.equal(result.status, "complete");
  assert.ok(result.facts.symbols.length > 0);
  assert.ok(result.facts.imports.length > 0);
  assert.ok(result.facts.calls.length > 0);
  assert.ok(result.facts.variables.length > 0);
  assert.ok(result.astSummary);
});

test("parses Rust source and extracts facts", () => {
  const rustSource = `use std::sync::Arc;

pub struct Worker {
    pub id: u32,
}

impl Worker {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
    pub fn run(&self) {
        println!("Worker {}", self.id);
    }
}

pub fn start() {
    let worker = Worker::new(1);
    worker.run();
}
`;
  const result = parseSync(rustSource, { language: "rust", includeAstSummary: true });
  assert.equal(result.language, "rust");
  assert.equal(result.status, "complete");
  assert.ok(result.facts.symbols.length > 0);
  assert.ok(result.facts.imports.length > 0);
  assert.ok(result.facts.calls.length > 0);
  assert.ok(result.facts.variables.length > 0);
  assert.ok(result.astSummary);
});

test("parses C source and extracts facts", () => {
  const cSource = `#include <stdio.h>

typedef struct {
    int id;
} Record;

int process(Record* rec) {
    printf("ID: %d\\n", rec->id);
    return 0;
}

int main() {
    Record r = { .id = 1 };
    return process(&r);
}
`;
  const result = parseSync(cSource, { language: "c", includeAstSummary: true });
  assert.equal(result.language, "c");
  assert.equal(result.status, "complete");
  assert.ok(result.facts.symbols.length > 0);
  assert.ok(result.facts.imports.length > 0);
  assert.ok(result.facts.calls.length > 0);
  assert.ok(result.facts.variables.length > 0);
  assert.ok(result.astSummary);
});

test("parses C++ source and extracts facts", () => {
  const cppSource = `#include <iostream>
#include <string>

namespace engine {
class Task {
public:
    void execute() {
        std::cout << "executing" << std::endl;
    }
};
}

int main() {
    engine::Task task;
    task.execute();
    return 0;
}
`;
  const result = parseSync(cppSource, { language: "cpp", includeAstSummary: true });
  assert.equal(result.language, "cpp");
  assert.equal(result.status, "complete");
  assert.ok(result.facts.symbols.length > 0);
  assert.ok(result.facts.imports.length > 0);
  assert.ok(result.facts.calls.length > 0);
  assert.ok(result.facts.variables.length > 0);
  assert.ok(result.astSummary);
});

test("parses C# source and extracts facts", () => {
  const csSource = `using System;

namespace Enterprise.App
{
    public class Worker
    {
        public void PerformWork()
        {
            Console.WriteLine("Working...");
        }
    }

    public class Program
    {
        public static void Main(string[] args)
        {
            var worker = new Worker();
            worker.PerformWork();
        }
    }
}
`;
  const result = parseSync(csSource, { language: "csharp", includeAstSummary: true });
  assert.equal(result.language, "csharp");
  assert.equal(result.status, "complete");
  assert.ok(result.facts.symbols.length > 0);
  assert.ok(result.facts.imports.length > 0);
  assert.ok(result.facts.calls.length > 0);
  assert.ok(result.facts.variables.length > 0);
  assert.ok(result.astSummary);
});
