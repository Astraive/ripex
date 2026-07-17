"""ripex-lang-test: Python async service — async/await, context manager."""
import asyncio
from typing import List, Literal


async def fetch_json(url: str) -> dict[str, str]:
    await asyncio.sleep(0)
    return {"url": url}


async def gather_all(urls: List[str]) -> List[dict[str, str]]:
    return await asyncio.gather(*(fetch_json(u) for u in urls))


class Resource:
    def __enter__(self) -> "Resource":
        return self

    def __exit__(self, *exc: object) -> Literal[False]:
        return False
