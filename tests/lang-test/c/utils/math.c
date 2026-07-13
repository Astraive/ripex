// ripex-lang-test: C math impl — arrays, function pointers.
#include "math.h"

int add(int a, int b) {
    return a + b;
}

int sum(int* xs, int n) {
    int total = 0;
    for (int i = 0; i < n; i++) {
        total += xs[i];
    }
    return total;
}
