int printf(char *format, ...);

int main() {
    int result = 1;
    
    // Calculate 5! = 5 * 4 * 3 * 2 * 1
    result = 5 * 4 * 3 * 2 * 1;
    printf("Factorial of 5 is %d\n", result);
    return result;
}
