// ripex-lang-test: C++ Product — template class, enum class, constexpr.
#ifndef RIPEX_PRODUCT_HPP
#define RIPEX_PRODUCT_HPP

#include <string>

enum class Category { Electronics, Clothing };

template <typename T>
class Product {
public:
    Product(int id, std::string name, double price, T category);

    double calculate_tax(double rate = 0.1) const;

    constexpr int id() const { return id_; }

private:
    int id_;
    std::string name_;
    double price_;
    T category_;
};

#endif // RIPEX_PRODUCT_HPP
