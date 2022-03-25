#include <iostream>

int main() {
    int *p; // Pointer to an integer

    { // Start of scope A
        int x = 1337;   // Value
        p = &x;         // Reference to value
    } // End of scope A

    // Undefined behavior! :(
    std::cout << "x = " << *p << std::endl;
    return 0;
}