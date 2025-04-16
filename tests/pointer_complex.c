// Complex pointer test
int main() {
    int x = 42;
    int y = 100;
    int *p = &x;
    int *q = &y;
    
    // Pointer assignment
    p = q;
    
    // Pointer dereference
    *p = 200;
    
    return *p + *q;
}
