"""ripex-lang-test: Python Product model — enum, class var, method."""
from enum import Enum
from typing import Optional


class Category(Enum):
    ELECTRONICS = "electronics"
    CLOTHING = "clothing"


class Product:
    tax_rate: float = 0.1

    def __init__(self, id: int, name: str, price: float, category: Category) -> None:
        self.id = id
        self.name = name
        self.price = price
        self.category = category

    def calculate_tax(self, rate: Optional[float] = None) -> float:
        return self.price * (rate if rate is not None else self.tax_rate)
