"""ripex-lang-test: Python config — typed constants, f-strings (gap: exprs dropped)."""
import os
from typing import Final, List

APP_NAME: Final[str] = "RipexLangTest"
VERSION: Final[str] = "1.0.0"
DEBUG: Final[bool] = os.environ.get("RIPEX_DEBUG", "false").lower() == "true"
MAX_RETRIES: Final[int] = 3
ALLOWED_ORIGINS: Final[List[str]] = ["http://localhost:3000"]


def get_db_url() -> str:
    return os.environ.get("DATABASE_URL", "sqlite:///ripex.db")


def report() -> str:
    # f-string with embedded expression — known silent drop in ripex
    return f"{APP_NAME} v{VERSION} (debug={DEBUG})"
