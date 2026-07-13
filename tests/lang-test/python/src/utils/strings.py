"""ripex-lang-test: Python string utils — decorators, generators."""
import functools
from typing import Callable


def logged(func: Callable) -> Callable:
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        return func(*args, **kwargs)
    return wrapper


@logged
def greet(name: str) -> str:
    return f"Hello, {name}!"


def mask_email(email: str) -> str:
    user, domain = email.split("@")
    return f"{user[0]}***@{domain}"


def range_gen(start: int, stop: int):
    i = start
    while i < stop:
        yield i
        i += 1
