#include <stdio.h>
#include <stdint.h>

// Call a Rust function neither accepting nor returning anything.
extern void meow();

// Pass an int to Rust.
extern void pass_cint_to_rust(int arg);

// Pass an int of a specific size to Rust.
extern void pass_int32_to_rust(int32_t arg);

// Return an int from Rust.
extern int get_cint_from_rust();

// Simple struct defined in Rust.
typedef struct
{
    int x;
    int y;
} Point;

// Get a pointer to a struct allocated in Rust.
extern Point *get_point(int x, int y);

// Define the variants of a heterogenous enum in Rust.
typedef enum
{
    _Integer,
    _Float,
} Number_Variant;

// Define the `Integer` variant inner body.
typedef struct
{
    int _inner;
} Integer_Body;

// Define the `Float` variant inner body.
typedef struct
{
    float _inner;
} Float_Body;

// Define the enum as struct with variant and a union of possible bodies.
typedef struct
{
    Number_Variant variant;
    union
    {
        Integer_Body _int;
        Float_Body _float;
    };
} Number;

// Get an integer enum variant from Rust.
extern Number *get_integer_number(int x);
// Get a float enum variant from Rust.
extern Number *get_float_number(float x);

// Print an enum in C which was created in Rust.
void print_number(Number *number)
{
    if (number->variant == _Integer)
    {
        printf("Number::Integer(%d)\n", number->_int._inner);
    }
    else if (number->variant == _Float)
    {
        printf("Number::Float(%f)\n", number->_float._inner);
    }
}

// Define a Rust struct to pass strings and vectors.
typedef struct
{
    const char *name;
    int32_t *values_ptr;
    size_t values_len;
} NamedCollection;

// Get a NamedCollection from Rust.
extern NamedCollection *get_named_collection();
// Free a NamedCollection in Rust.
extern void free_named_collection(NamedCollection *ptr);

int main(void)
{
    meow();
    pass_cint_to_rust(42);
    pass_int32_to_rust(123);

    printf("Received %d from Rust\n", get_cint_from_rust());

    Point *p1 = get_point(1, 2);
    printf("Created Point in Rust with x: %d and y: %d\n", p1->x, p1->y);

    Number *i1 = get_integer_number(34);
    print_number(i1);
    Number *f1 = get_float_number(3.14);
    print_number(f1);

    NamedCollection *n1 = get_named_collection();
    printf("C got NamedCollection %s from Rust\n", n1->name);
    for (int i = 0; i < n1->values_len; i++)
    {
        printf("Value: %d\n", *(n1->values_ptr + i));
    }
    free_named_collection(n1);

    return 0;
}
