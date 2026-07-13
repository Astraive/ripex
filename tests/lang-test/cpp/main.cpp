// ripex-lang-test: C++ entry — includes, namespaces, templates, lambda.
#include "models/user.hpp"
#include "models/product.hpp"
#include "utils/math.hpp"
#include "services/service.cpp"
#include <iostream>

int main() {
    User alice("Alice", "alice@example.com");
    alice.add_role("admin");
    std::cout << alice.describe() << std::endl;
    std::cout << "admin? " << alice.is_admin() << std::endl;

    Product<Category> widget(1, "Widget", 19.99, Category::Electronics);
    std::cout << "tax=" << widget.calculate_tax() << std::endl;

    std::vector<int> xs = {1, 2, 3};
    auto sq = squares(xs);
    std::cout << "sum=" << add(1, 2) << std::endl;

    auto results = ripex::services::fetch_all({"a", "b"});
    return 0;
}
