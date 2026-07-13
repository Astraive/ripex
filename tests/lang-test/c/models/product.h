// ripex-lang-test: C Product header — enum, struct, const.
#ifndef RIPEX_PRODUCT_H
#define RIPEX_PRODUCT_H

typedef enum { ELECTRONICS, CLOTHING } Category;

typedef struct Product {
    int id;
    char* name;
    double price;
    Category category;
} Product;

double product_calculate_tax(const Product* p, double rate);

#endif // RIPEX_PRODUCT_H
