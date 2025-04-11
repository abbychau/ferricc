// Factorial program with command-line argument
int printf(char *format, ...);
int atoi(char *str);

// Calculate factorial using a loop
int factorial(int n) {
    int result = 1;
    int i = 1;
    
    while (i <= n) {
        result = result * i;
        i = i + 1;
    }
    
    return result;
}

int main(int argc, char **argv) {
    int n;
    
    // Check if an argument was provided
    if (argc < 2) {
        printf("Usage: %s <number>\n", argv[0]);
        return 1;
    }
    
    // Convert the argument to an integer
    n = atoi(argv[1]);
    
    // Check if the number is valid
    if (n < 0) {
        printf("Error: Cannot calculate factorial of a negative number.\n");
        return 2;
    }
    
    // Calculate the factorial
    int result = factorial(n);
    
    // Print the result
    printf("Factorial of %d is %d\n", n, result);
    
    // Return success
    return 0;
}
