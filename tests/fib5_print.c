int printf(char *format, ...);

int fib(int n) {
    if (n <= 1) {
        return n;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}

int main() {
    int result = 1;
    result = fib(5);
    printf("Fibonacci of 5 is %d\n", result);
    return result;
}
