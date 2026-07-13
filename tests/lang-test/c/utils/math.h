// ripex-lang-test: C math header — function pointer typedef.
#ifndef RIPEX_MATH_H
#define RIPEX_MATH_H

typedef int (*binary_op)(int, int);

int add(int a, int b);
int sum(int* xs, int n);

#endif // RIPEX_MATH_H
