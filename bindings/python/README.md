# ripex

Native multi-language structural parsing and fact extraction for Python.

## Installation

```bash
pip install ripex
```

## Quick Start

```python
import ripex

source = """
def add(a: int, b: int) -> int:
    return a + b
"""

output = ripex.parse_sync(source, language="python")
print(f"Language: {output.language}, Status: {output.status}")

for symbol in output.facts.symbols:
    print(f"Symbol: {symbol.name} ({symbol.kind})")
```

## Supported Languages

- Python (`python`)
- JavaScript (`javascript`)
- TypeScript (`typescript`)
- Go (`go`)
- Rust (`rust`)
- C (`c`)
- C++ (`cpp`)
- C# (`csharp`)
