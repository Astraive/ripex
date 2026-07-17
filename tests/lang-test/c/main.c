// ripex-lang-test: C entry — include graph, call graph.
#include "models/user.h"
#include "models/product.h"
#include "utils/math.h"

int main(void) {
    User* alice = user_new("Alice", "alice@example.com");
    char* desc = user_describe(alice);
    int admin = user_is_admin(alice);

    Product widget;
    widget.id = 1;
    widget.price = 19.99;
    widget.category = ELECTRONICS;
    double tax = product_calculate_tax(&widget, 0.0);

    int xs[] = {1, 2, 3};
    int s = sum(xs, 3);

    user_free(alice);
    return 0;
}
