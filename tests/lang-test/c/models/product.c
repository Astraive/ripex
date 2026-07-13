// ripex-lang-test: C Product impl — const, switch.
#include "product.h"

const double TAX_RATE = 0.1;

double product_calculate_tax(const Product* p, double rate) {
    if (rate <= 0.0) rate = TAX_RATE;
    return p->price * rate;
}
