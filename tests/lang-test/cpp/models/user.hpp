// ripex-lang-test: C++ User header — class, access, virtual, smart ptr.
#ifndef RIPEX_USER_HPP
#define RIPEX_USER_HPP

#include <string>
#include <vector>
#include <memory>

class User {
public:
    User(std::string name, std::string email);
    virtual ~User() = default;

    std::string describe() const;
    bool is_admin() const;
    void add_role(const std::string& role);

protected:
    std::string name_;
    std::string email_;

private:
    std::vector<std::string> roles_;
};

using UserPtr = std::shared_ptr<User>;

#endif // RIPEX_USER_HPP
