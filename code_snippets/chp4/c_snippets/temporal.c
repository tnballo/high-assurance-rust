#include <assert.h>     // assert
#include <stdio.h>      // puts
#include <string.h>     // strncpy, strlen
#include <stdlib.h>     // malloc

char* get_greeting() {
    char* greeting = (char*)malloc(6);
    if (greeting == NULL) {
        return NULL;
    } else {
        strncpy(greeting, "Hello", 6);
        assert(greeting[6] == '\0');
        return greeting;
    }
}

int main() {
    char* greeting = get_greeting();
    size_t greeting_len = strlen(greeting); // Excludes null byte
    if (greeting != NULL) {
        // Append "!" correctly
        greeting = (char*)realloc(greeting, greeting_len + 2);
        if (greeting != NULL) {
            // strcat could be used here instead of the two lines below
            greeting[greeting_len] = '!';
            greeting[greeting_len + 1] = '\0';
        }
        puts(greeting);
        free(greeting);
    }

    // Double-free, temporal safety violation!
    free(greeting);
    return 0;
}