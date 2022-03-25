#include <stdio.h>

int undef_func() {
    int uninit_var;
    if (uninit_var > 0) {
        return 1;
    } else {
        return 0;
    }
}

int main() {
    printf("%d\n", undef_func());
}