// ripex-lang-test: C++ User impl — ctor init list, range-for.
#include "user.hpp"

User::User(std::string name, std::string email)
    : name_(std::move(name)), email_(std::move(email)) {}

std::string User::describe() const {
    return name_ + " <" + email_ + ">";
}

void User::add_role(const std::string& role) {
    roles_.push_back(role);
}

bool User::is_admin() const {
    for (const auto& r : roles_) {
        if (r == "admin") return true;
    }
    return false;
}
