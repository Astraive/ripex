// ripex-lang-test: C# entry — using directives, call graph, top-level statements.
using System;
using System.Linq;
using Ripex.Models;
using Ripex.Utils;
using Ripex.Services;

var alice = new User("Alice", "alice@example.com");
alice.Roles.Add("admin");
Console.WriteLine(Strings.Greet(alice.Name));
Console.WriteLine(alice.Describe());
Console.WriteLine($"admin? {alice.IsAdmin()}");

var widget = new Product(1, "Widget", 19.99, Category.Electronics);
Console.WriteLine($"tax={widget.PriceOf()}");

Console.WriteLine($"sum={MathUtil.Sum(MathUtil.Squares(new[] { -1, 2, 3 }).ToArray())}");

var svc = new AsyncService();
var results = await svc.FetchAllAsync(new[] { "a", "b" });
Console.WriteLine(alice.Name.Truncate(3));
