// ripex-lang-test: C# Product — record, enum, interface, method.
namespace Ripex.Models
{
    public enum Category
    {
        Electronics,
        Clothing
    }

    public interface IPricable
    {
        double PriceOf();
    }

    public record Product(int Id, string Name, double Price, Category Cat) : IPricable
    {
        public double PriceOf() => Price;
    }
}
