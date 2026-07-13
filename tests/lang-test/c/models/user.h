// ripex-lang-test: C User header — struct, typedef, function decls.
#ifndef RIPEX_USER_H
#define RIPEX_USER_H

typedef struct User {
    char* name;
    char* email;
    char** roles;
    int role_count;
} User;

User* user_new(const char* name, const char* email);
void user_free(User* u);
char* user_describe(User* u);
int user_is_admin(User* u);

#endif // RIPEX_USER_H
