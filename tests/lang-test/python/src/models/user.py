"""ripex-lang-test: Python User model — dataclass, classmethod, properties."""
from dataclasses import dataclass, field
from typing import List, Optional


@dataclass
class User:
    name: str
    email: str
    roles: List[str] = field(default_factory=list)
    _password: str = ""

    @property
    def password(self) -> str:
        return self._password

    @password.setter
    def password(self, value: str) -> None:
        self._password = value

    @classmethod
    def guest(cls) -> "User":
        return cls("Guest", "guest@example.com")

    def describe(self) -> str:
        return f"{self.name} <{self.email}>"

    @property
    def is_admin(self) -> bool:
        return "admin" in self.roles
