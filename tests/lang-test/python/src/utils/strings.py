"""ripex-lang-test: Python string utils — decorators, generators."""
import functools
from collections.abc import Callable, Generator
from typing import ParamSpec, TypeVar

P = ParamSpec("P")
R = TypeVar("R")


def logged(func: Callable[P, R]) -> Callable[P, R]:
    @functools.wraps(func)
    def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
        return func(*args, **kwargs)
    return wrapper


@logged
def greet(name: str) -> str:
    return f"Hello, {name}!"


def mask_email(email: str) -> str:
    user, domain = email.split("@")
    return f"{user[0]}***@{domain}"


def range_gen(start: int, stop: int) -> Generator[int, None, None]:
    i = start
    while i < stop:
        yield i
        i += 1
