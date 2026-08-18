import pytest
import asyncio
import ripex

# ---------------------------------------------------------------------------
# Sample multi-language source codes
# ---------------------------------------------------------------------------

JS_SOURCE = """// JS line comment
/* JS block comment */
import { compute } from "./math.js";

export class Calculator {
    calculate(val) {
        return compute(val);
    }
}

const instance = new Calculator();
const output = instance.calculate(42);
"""

TS_SOURCE = """// TS leading comment
import { helper as importedHelper } from "./helper";

export function add(a: number, b: number): number {
    return importedHelper(a + b);
}

const answer = add(1, 2);
"""

PY_SOURCE = """# Python comment
import os
from math import sqrt

class Geometry:
    def distance(self, x: float, y: float) -> float:
        return sqrt(x * x + y * y)

geo = Geometry()
res = geo.distance(3.0, 4.0)
"""

GO_SOURCE = """// Go package comment
package main

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
"""

RUST_SOURCE = """// Rust module comment
use std::sync::Arc;

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
"""

C_SOURCE = """/* C comment */
#include <stdio.h>

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
"""

CPP_SOURCE = """// C++ comment
#include <iostream>
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
"""

CSHARP_SOURCE = """// C# source file
using System;

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
"""


# ---------------------------------------------------------------------------
# Unit tests
# ---------------------------------------------------------------------------

def test_version() -> None:
    assert ripex.__version__ == "0.3.1"


def test_supported_languages() -> None:
    langs = ripex.supported_languages()
    assert langs == [
        "c",
        "cpp",
        "csharp",
        "go",
        "javascript",
        "python",
        "rust",
        "typescript",
    ]


def test_detect_language() -> None:
    assert ripex.detect_language("src/index.ts") == "typescript"
    assert ripex.detect_language("script.js") == "javascript"
    assert ripex.detect_language("main.py") == "python"
    assert ripex.detect_language("server.go") == "go"
    assert ripex.detect_language("lib.rs") == "rust"
    assert ripex.detect_language("file.c") == "c"
    assert ripex.detect_language("header.h") is None
    assert ripex.detect_language("Program.cs") == "csharp"
    assert ripex.detect_language("unknown.xyz") is None


def test_parse_sync_typescript() -> None:
    out = ripex.parse_sync(TS_SOURCE, language="typescript", include_ast_summary=True)
    assert out.language == "typescript"
    assert out.status == "complete"
    assert out.completeness is True
    assert out.truncated is False
    assert isinstance(out.effective_mode, str)

    # Facts
    assert len(out.facts.symbols) > 0
    assert len(out.facts.imports) > 0
    assert len(out.facts.calls) > 0
    assert len(out.facts.variables) > 0

    # Comments
    assert any(c.kind == "line" and "TS leading" in c.text for c in out.comments)

    # AST Summary
    assert out.ast_summary is not None
    assert out.ast_summary.top_level_nodes > 0

    # Helper methods on Facts
    syms = out.facts.find_symbols_by_name("add")
    assert len(syms) > 0
    assert syms[0].name == "add"

    calls = out.facts.find_calls_by_callee("importedHelper")
    assert len(calls) > 0

    imports = out.facts.find_imports_by_source("helper")
    assert len(imports) > 0

    # to_dict serialization
    d = out.to_dict()
    assert isinstance(d, dict)
    assert d["language"] == "typescript"
    assert d["status"] == "complete"
    assert "facts" in d
    assert "symbols" in d["facts"]


def test_parse_sync_all_languages() -> None:
    test_cases = [
        ("javascript", JS_SOURCE, {"extension": "js"}),
        ("typescript", TS_SOURCE, {}),
        ("python", PY_SOURCE, {}),
        ("go", GO_SOURCE, {}),
        ("rust", RUST_SOURCE, {}),
        ("c", C_SOURCE, {}),
        ("cpp", CPP_SOURCE, {}),
        ("csharp", CSHARP_SOURCE, {}),
    ]

    for lang, source, kwargs in test_cases:
        out = ripex.parse_sync(source, language=lang, include_ast_summary=True, **kwargs)
        assert out.language == lang, f"Expected language {lang}, got {out.language}"
        assert out.status == "complete", f"Failed for {lang}: status {out.status}, diagnostics: {out.diagnostics}"
        assert out.completeness is True
        assert len(out.facts.symbols) > 0, f"No symbols extracted for {lang}"
        assert out.ast_summary is not None


@pytest.mark.asyncio
async def test_parse_async() -> None:
    out = await ripex.parse(TS_SOURCE, language="typescript")
    assert out.language == "typescript"
    assert out.status == "complete"
    assert len(out.facts.symbols) > 0


def test_enum_options() -> None:
    out = ripex.parse_sync(TS_SOURCE, language=ripex.Language.TYPESCRIPT)
    assert out.language == "typescript"
    assert out.status == ripex.ParseStatus.COMPLETE


def test_invalid_language() -> None:
    with pytest.raises(ValueError, match=r"unsupported or unknown language"):
        ripex.parse_sync("const x = 1;", language="nonexistent")


def test_malformed_syntax_recovery() -> None:
    malformed = "export const = 1;"
    out = ripex.parse_sync(malformed, language="typescript")
    assert out.status == "recovered"
    assert out.completeness is False
    assert len(out.diagnostics) > 0
    diag = out.diagnostics[0]
    assert diag.code is not None
    assert diag.message is not None
    assert diag.span.start.line >= 1


def test_input_too_large_limit() -> None:
    large_input = "x" * (1_048_576 + 1)
    out = ripex.parse_sync(large_input, language="javascript")
    assert out.status == "limit_exceeded"
    assert out.truncated is True
    assert any(d.code == "input_too_large" for d in out.diagnostics)
