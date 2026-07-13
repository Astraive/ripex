"""ripex-lang-test: Python entry — bushy imports, call graph, f-strings."""
from src.models.user import User
from src.models.product import Product, Category
from src.utils.math_utils import add, squares
from src.utils.strings import greet, mask_email
from src.errors import ValidationError


def run_demo() -> None:
    alice = User("Alice", "alice@example.com", roles=["admin"])
    alice.password = "secret"
    print(greet(alice.name))
    print(alice.describe())
    print(f"admin? {alice.is_admin}")

    widget = Product(1, "Widget", 19.99, Category.ELECTRONICS)
    print(f"tax={widget.calculate_tax()}")

    try:
        raise ValidationError("boom")
    except ValidationError as e:
        print(f"caught: {e}")

    print(f"sum squares={sum(squares([-1, 2, 3]))}")
