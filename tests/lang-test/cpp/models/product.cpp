// ripex-lang-test: C++ Product impl — template definition.
#include "product.hpp"

template <typename T>
Product<T>::Product(int id, std::string name, double price, T category)
    : id_(id), name_(std::move(name)), price_(price), category_(category) {}

template <typename T>
double Product<T>::calculate_tax(double rate) const {
    return price_ * rate;
}

template class Product<Category>;
