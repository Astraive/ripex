"""ripex-lang-test: Python math — functions, type hints, walrus, comprehensions."""
from typing import List


def add(a: int, b: int = 1) -> int:
    return a + b


def factorial(n: int) -> int:
    result = 1
    for i in range(2, n + 1):
        result *= i
    return result


def squares(xs: List[int]) -> List[int]:
    return [x * x for x in xs if x > 0]


def process(data: List[int]) -> int:
    if (n := len(data)) > 100:
        print(f"large input: {n}")
    return n
