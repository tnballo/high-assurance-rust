#include <assert.h>     // assert
#include <stdio.h>      // puts
#include <string.h>     // strncpy
#include <stdlib.h>     // malloc

char* get_greeting() {
    char* greeting = (char*)malloc(6);
    if (greeting == NULL) {
        return NULL;
    } else {
        strncpy(greeting, "Hello", 6);
        assert(greeting[5] == '\0');
        return greeting;
    }
}

int main() {
    char* greeting = get_greeting();
    if (greeting != NULL) {
        // Buffer overwrite, spatial safety violation!
        greeting[12] = '!';

        puts(greeting);
        free(greeting);
    }
    return 0;
}