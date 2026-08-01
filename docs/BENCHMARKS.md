# Ripex v0.3.0 parser evidence

This report is generated from checked-in source and curated gold cases.
It is evidence for the published structural contract, not a claim of compiler-level semantic equivalence.

Release references: [crates.io](https://crates.io/crates/ripex/0.3.0) ·
[docs.rs](https://docs.rs/ripex/0.3.0) ·
[GitHub release](https://github.com/Astraive/ripex/releases/tag/v0.3.0).

Environment: `windows/x86_64`; corpus source: `tests/lang-test`; benchmark iterations: `10`.

## Corpus coverage

| Language | Corpus files | Bytes | Complete parses | Parse success | Diagnostics | Panics | Hangs | Tree-sitter clean parses |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `c` | 8 | 3148 | 8 | 100.00% | 0 | 0 | 0 | 8/8 |
| `cpp` | 7 | 3792 | 7 | 100.00% | 0 | 0 | 0 | 6/7 |
| `csharp` | 6 | 3934 | 6 | 100.00% | 0 | 0 | 0 | 6/6 |
| `go` | 6 | 2979 | 6 | 100.00% | 0 | 0 | 0 | 6/6 |
| `javascript` | 8 | 7187 | 8 | 100.00% | 0 | 0 | 0 | 8/8 |
| `python` | 9 | 4823 | 9 | 100.00% | 0 | 0 | 0 | 9/9 |
| `rust` | 9 | 3464 | 9 | 100.00% | 0 | 0 | 0 | 9/9 |
| `typescript` | 3 | 1571 | 3 | 100.00% | 0 | 0 | 0 | 3/3 |

## Curated fact accuracy

Gold facts are independently listed in `examples/evidence/<language>.rs`; matching uses exact names, import sources, call names, and variable names.

| Language | Case IDs | Cases | Gold facts | Predicted facts | True positives | Precision | Recall |
|---|---|---:|---:|---:|---:|---:|---:|
| `c` | `c17_struct_call` | 1 | 11 | 11 | 11 | 100.00% | 100.00% |
| `cpp` | `cpp20_namespace_class_call` | 1 | 12 | 12 | 12 | 100.00% | 100.00% |
| `csharp` | `csharp-using-class-method-call-vars` | 1 | 10 | 10 | 10 | 100.00% | 100.00% |
| `go` | `go-basics` | 1 | 6 | 6 | 6 | 100.00% | 100.00% |
| `javascript` | `javascript-module-bindings` | 1 | 10 | 10 | 10 | 100.00% | 100.00% |
| `python` | `python_core_symbols_and_facts` | 1 | 14 | 14 | 14 | 100.00% | 100.00% |
| `rust` | `rust_struct_function_facts` | 1 | 10 | 10 | 10 | 100.00% | 100.00% |
| `typescript` | `typescript_interface_function_constants` | 1 | 12 | 12 | 12 | 100.00% | 100.00% |

### Gold mismatches

No gold mismatches detected.


## Malformed-input behavior

Ripex exercised **17** curated malformed inputs: **15** produced a diagnostic or incomplete status, **0** panicked, and **0** exceeded the two-second watchdog.

Each language module owns at least two malformed variants; malformed input is measured separately from the valid fact oracle.

## Throughput and allocation evidence

Throughput includes parse plus best-effort fact extraction. Ripex memory is peak allocator bytes observed during the measured loop. Tree-sitter parsing uses native allocations that this Rust allocator probe cannot observe, so its allocator column is reported as zero and must not be interpreted as total memory. This keeps the measurement cross-platform and explicit. Tree-sitter is parse-only.

| Language | Corpus bytes | Ripex MB/s | Ripex peak alloc bytes | Tree-sitter MB/s | Tree-sitter observed Rust alloc bytes |
|---|---:|---:|---:|---:|---:|
| `c` | 3148 | 3.75 | 85622 | 2.42 | 0 |
| `cpp` | 3792 | 3.91 | 53014 | 2.19 | 0 |
| `csharp` | 3934 | 3.52 | 55817 | 1.49 | 0 |
| `go` | 2979 | 2.97 | 48439 | 1.78 | 0 |
| `javascript` | 7187 | 4.08 | 206629 | 2.65 | 0 |
| `python` | 4823 | 3.67 | 58511 | 1.98 | 0 |
| `rust` | 3464 | 3.31 | 52897 | 2.26 | 0 |
| `typescript` | 1571 | 3.48 | 72388 | 2.10 | 0 |

## Interpretation and limits

- Corpus measurements use the repository's checked-in language-test sources; the table records their exact file and byte counts.
- Precision/recall applies to the curated gold cases, not to every fact in every corpus file. Expanding gold coverage is the path to stronger accuracy claims.
- Tree-sitter comparison measures parser acceptance and parse throughput; it is not treated as an independent semantic oracle.
- Compiler conformance remains a separate gate because structural parsing cannot establish type checking, linking, macro, SDK, or project semantics.
