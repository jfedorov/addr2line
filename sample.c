#include <stdio.h>

int add(int a, int b) {
    return a + b;  // Line 4
}

int multiply(int x, int y) {
    return x * y;  // Line 8
}

int main() {
    int result1 = add(10, 20);       // Line 12
    int result2 = multiply(5, 6);    // Line 13

    printf("Results: %d, %d\n", result1, result2);  // Line 15
    return 0;
}
