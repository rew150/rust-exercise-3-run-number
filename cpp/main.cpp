#include <stdio.h>
#include <string.h>

extern "C" {
    #include <stdio.h>
    fpos_t * allocate_fpos_t();
    void deallocate_fpos_t(fpos_t *ptr);
    void copy_fpos_t(fpos_t *dst, const fpos_t *src);
    void hello_world();
}

extern "C" fpos_t * allocate_fpos_t() {
    return new fpos_t;
}

extern "C" void deallocate_fpos_t(fpos_t *ptr) {
    delete ptr;
}

extern "C" void copy_fpos_t(fpos_t *dst, const fpos_t *src) {
    memcpy(dst, src, sizeof(fpos_t));
}

extern "C" void hello_world() {
    printf("Hello, world!");
}
