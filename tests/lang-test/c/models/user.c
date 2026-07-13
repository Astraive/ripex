// ripex-lang-test: C User impl — malloc, impl, pointer arithmetic.
#include "user.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

User* user_new(const char* name, const char* email) {
    User* u = (User*)malloc(sizeof(User));
    u->name = strdup(name);
    u->email = strdup(email);
    u->roles = NULL;
    u->role_count = 0;
    return u;
}

void user_free(User* u) {
    free(u->name);
    free(u->email);
    free(u);
}

char* user_describe(User* u) {
    char* buf = (char*)malloc(256);
    snprintf(buf, 256, "%s <%s>", u->name, u->email);
    return buf;
}

int user_is_admin(User* u) {
    for (int i = 0; i < u->role_count; i++) {
        if (strcmp(u->roles[i], "admin") == 0) return 1;
    }
    return 0;
}
