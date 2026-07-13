// ripex-lang-test: C++ math header — function template, lambda (gap: body dropped).
#ifndef RIPEX_MATH_HPP
#define RIPEX_MATH_HPP

#include <vector>
#include <algorithm>

template <typename T>
T add(T a, T b) {
    return a + b;
}

inline std::vector<int> squares(const std::vector<int>& xs) {
    std::vector<int> out;
    std::transform(xs.begin(), xs.end(), std::back_inserter(out),
                   [](int x) { return x * x; });
    return out;
}

#endif // RIPEX_MATH_HPP
