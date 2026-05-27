#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void rpy_rt_init(void) {}

void rpy_rt_fini(void) {}

void rpy_panic(const char *msg, int64_t len) {
    (void)len;
    fprintf(stderr, "rpython panic: %.*s\n", (int)len, msg);
    abort();
}

void rpy_print_str(const char *ptr, int64_t len) {
    if (ptr == NULL) {
        return;
    }
    fwrite(ptr, 1, (size_t)len, stdout);
}

void rpy_print_int(int64_t i) {
    printf("%lld", (long long)i);
}

void rpy_print_bool(int8_t b) {
    fputs(b ? "true" : "false", stdout);
}

void rpy_print_newline(void) {
    putchar('\n');
}

int8_t rpy_str_eq(const char *a, int64_t alen, const char *b, int64_t blen) {
    if (alen != blen) {
        return 0;
    }
    if (a == NULL || b == NULL) {
        return alen == 0 ? 1 : 0;
    }
    return memcmp(a, b, (size_t)alen) == 0 ? 1 : 0;
}

void *rpy_alloc(int64_t size) {
    if (size <= 0) {
        return NULL;
    }
    return malloc((size_t)size);
}

void rpy_free(void *ptr) {
    free(ptr);
}
