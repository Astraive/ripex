// ripex-lang-test: C# math utils — static methods, LINQ, generics.
using System.Collections.Generic;
using System.Linq;

namespace Ripex.Utils
{
    public static class MathUtil
    {
        public static int Add(int a, int b) => a + b;

        public static IEnumerable<int> Squares(IEnumerable<int> xs) =>
            xs.Where(x => x > 0).Select(x => x * x);

        public static T Max<T>(T a, T b) where T : System.IComparable<T> =>
            a.CompareTo(b) > 0 ? a : b;

        public static int Sum(params int[] xs) => xs.Sum();
    }
}
