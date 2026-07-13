"""ripex-lang-test: Python async service — async/await, context manager."""
import asyncio
from typing import List


async def fetch_json(url: str) -> dict:
    await asyncio.sleep(0)
    return {"url": url}


async def gather_all(urls: List[str]) -> List[dict]:
    return await asyncio.gather(*(fetch_json(u) for u in urls))


class Resource:
    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False
