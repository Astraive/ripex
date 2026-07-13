// ripex-lang-test: C service — callbacks, threads reference.
#include <pthread.h>

typedef void (*callback)(const char*);

void run_callback(callback cb, const char* msg) {
    if (cb) cb(msg);
}

static void log_msg(const char* msg) {
    // logging
}
