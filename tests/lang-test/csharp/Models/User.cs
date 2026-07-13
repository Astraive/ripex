// ripex-lang-test: C# User model — class, properties, constructor, auto-props.
using System.Collections.Generic;

namespace Ripex.Models
{
    public class User
    {
        public string Name { get; set; }
        public string Email { get; set; }
        public List<string> Roles { get; set; } = new();

        public User(string name, string email)
        {
            Name = name;
            Email = email;
        }

        public virtual string Describe() => $"{Name} <{Email}>";

        public bool IsAdmin() => Roles.Contains("admin");
    }

    public class AdminUser : User
    {
        public AdminUser(string name, string email) : base(name, email) { }

        public override string Describe() => $"ADMIN {base.Describe()}";
    }
}
