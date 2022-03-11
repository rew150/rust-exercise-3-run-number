#include <stdio.h>

extern "C" {
    #include <stdio.h>
    fpos_t * allocate_fpos_t();
    void deallocate_fpos_t(fpos_t *ptr);
    void hello_world();
}

extern "C" fpos_t * allocate_fpos_t() {
    return new fpos_t;
}

extern "C" void deallocate_fpos_t(fpos_t *ptr) {
    delete ptr;
}

extern "C" void hello_world() {
    printf("Hello, world!");
}
