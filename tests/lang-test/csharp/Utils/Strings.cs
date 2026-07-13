// ripex-lang-test: C# string utils — extension methods, string interpolation.
namespace Ripex.Utils
{
    public static class Strings
    {
        public static string Greet(string name) => $"Hello, {name}!";

        public static string MaskEmail(string email)
        {
            var parts = email.Split('@');
            return $"{parts[0][0]}***@{parts[1]}";
        }
    }

    public static class StringExtensions
    {
        public static string Truncate(this string s, int n = 10) =>
            s.Length > n ? s.Substring(0, n) + "…" : s;
    }
}
