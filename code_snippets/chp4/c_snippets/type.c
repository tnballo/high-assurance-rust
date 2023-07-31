#include <assert.h>     // assert
#include <stdio.h>      // puts
#include <string.h>     // strncpy, strlen
#include <stdlib.h>     // malloc

#define TYPE_NEW_USR 1 // New user, to be greeted
#define TYPE_CUR_USR 2 // Current user, increment visit count

struct user_record_t {
    int type;
    union {
        char *greeting;
        unsigned visit_count;
    };
};

int main() {
    struct user_record_t rec;

    rec.type = TYPE_NEW_USR;
    rec.greeting = "Hello!";

    // Logic error: should be `TYPE_CUR_USR`
    if (rec.type == TYPE_NEW_USR) {
        rec.visit_count += 1; // Type confusion, a type safety violation!
    }

    if (rec.type == TYPE_NEW_USR) {
        printf("%s\n", rec.greeting);
    }

    return 0;
}