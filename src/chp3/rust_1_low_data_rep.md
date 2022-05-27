# Rust: Low-Level Data (1 of 6)

Thus far we've discussed Rust's type system in the context of static assurance.
Specifically preventing mutable aliasing and eliminating UB.

But, for the majority of day-to-day development, one could argue that these are secondary benefits.
And that the true value of Rust's type system lies in its expressiveness, in the ability to map our problem domain to flexible constructs.

This line of argument quickly becomes subjective, you should form your own opinions of Rust over time.
But the first step in solving a programming problem is typically representing data to process.
So we'll sample the options Rust affords us.

## Primitive Types

Rust's primitive types are similar to almost any programming language you're familiar with - it has the usual booleans, integers, floats, characters, strings, etc.

One key difference, relative to higher-level interpreted languages, is that integers and floats have a fixed width.
This is a hallmark of high-performance systems languages, where individual numbers need to be stored in CPU registers (like C) and not as structures in heap memory (like Python).

This hardware-level concern has two important implications: bounded ranges and host-specific widths.

### 1) Bounded Numeric Ranges

Rust has 12 primitive numeric types:

* 5 unsigned integer types: `u8`, `u16`, `u32`, `u64`, `u128`, and `usize`.

* 5 signed integer types: `i8`, `i16`, `i32`, `i64`, `i128`, and `isize`.

* 2 IEEE-compliant floats: `f32` (at least 6 decimal digits precision) and `f64` (at least 15 decimal digits).

The postfix in the type name indicates bit-width, e.g. a `u128` is 128 bits (16 bytes) wide.
Here's the important implication: the range of values a given integer type can represent is finite.
Upper and lower bounds are determined by both signedness and width.
Consider the below table (non-exhaustive):

| Type | Width | Lower Bound | Upper Bound |
| --- | --- | --- | -- |
| u8 | 1 byte | 0 | 255 |
| i8 | 1 byte | -128 | 127 |
| u32 | 4 bytes | 0 | 4,294,967,295 |
| i64 | 8 bytes | -2<sup>63</sup> | 2<sup>63</sup>-1  |

Rust's standard library provides handy limit constants for upper and lower bounds, so you don't have to remember these ranges off-hand or reference an exhaustive table:

```rust,noplaypen
assert_eq!(0, u8::MIN);
assert_eq!(255, u8::MAX);
```

Exceeding the range of a type causes "wrap around".
In rare cases, that's desirable behavior.
We made judicious and intentional use of `wrapping_add` when implementing the RC4 cipher to simulate modular arithmetic.
To demonstrate how that works, consider what would happen if we exceeded the `255` upper bound of a `u8`:

```rust,noplaypen
let x: u8 = 200;
let y: u8 = 100;

assert_eq!(x.wrapping_add(y), 44);
```

`44` is `300 % 256`, e.g. total modulo range size.
Outside of cryptographic contexts, silent wraparound is considered an *integer overflow* bug.
If `200` represented the number of dollars in a bank account and the account owner deposited another `100` dollars, they'd be shocked to find a `44` account balance on their receipt!

This is where we get into some subtleties in Rust.
Had we written `assert_eq!(x + y, 44);` instead of `assert_eq!(x.wrapping_add(y), 44);`, the program would have spit out an error warning of an overflow:

```ignore
error: this arithmetic operation will overflow
 --> src/main.rs:8:12
  |
8 | assert_eq!(x + y, 44);
  |            ^^^^^ attempt to compute `200_u8 + 100_u8`, which would overflow
  |
  = note: `#[deny(arithmetic_overflow)]` on by default
```

We got lucky here in the sense that both `x` and `y` are constants, so the overflow could be detected at compile-time.
Rust uses optional run-time checks to catch overflow for variables whose values aren't known beforehand - a topic we'll return to in Chapter 4 when discussing safety in-depth.

There's one more detail to keep in mind about integer overflow in Rust: unlike C/C++, it's not a potential source of UB.
The rules of wrap around are specified and universal across target platforms[^IntWrap].

### 2) Host-specific Integers

You have noticed the `usize` and `isize` types, unsigned and signed integers respectively, don't specify a bit-width like their counterparts.
That's because their size depends on the specific machine the program is compiled for.

Both are 4 bytes long if compiling for a 32-bit system, and 8 bytes long on a modern 64-bit system.
In theory, they could be 16 bytes long for a 128-bit system - but no commercial processors use 128-bit architectures.

Given what we said about ranges and overflow, machine-dependant (aka host-specific) types might strike you as ambiguous.
Maybe even dangerous.
Per MISRA, you'd be right:

> **[RR, Directive 4.6]** Use numeric types of explicit size and signedness[^MISRA_2012]

While Rust lets us use explicit numeric types were possible, indexing is an exception: `usize` types are required for collection indexing (e.g. in `my_vec[i] = j`, `i` must be a `usize`).
This is because, under-the-hood, indexing a container often involves computing a memory address[^IndexAside].
And the width of an address depends on the target machine.

Now Rust lets us cast from an explicit numeric type, like an `u64`, into a `usize`.
Perhaps we need to perform this operation prior to indexing, to comply with the spirit of the above MISRA rule.

Casting between numeric types is one of the very few cases where Rust permits type casting.
Which helps with another rule:

> **[AR, Rule 11.3]** Never cast from a reference to one type into a reference to another type[^MISRA_2012]

Rust does allow safe and explicit conversion between types (not between references), via the concept of *traits* - specifically traits called `From`[^TraitFrom] and `Into`[^TraitInto].
We'll explain traits in the next section and use `From` in a later chapter.

## Type Inference

Rust is strongly and statically typed.
Every value has a type known at compile time.
Even generic parameters, whose ultimate type is decided during compilation (more on this later).

Unlike older statically typed languages, Rust uses *type inference*[^TypeInference] to automatically detect the type of an expression in certain cases. As a rule of thumb, explicitly writing out type annotations is:

* *Always required* for function signatures (e.g. parameter and return types), global variables, or exported types (e.g. part of a library's public API).

* *Occasionally required* within the body of a function.

Consider this example:

```rust,ignore
#![feature(type_name_of_val)]
use std::any::type_name_of_val;

fn sum(x: u128, y: u128) -> u128 {
    x + y
}

fn main() {
    let a = 1;
    let b = 3;
    let c = sum(a, b);

    println!("a is a {} with value {:?}", type_name_of_val(&a), a);
    println!("b is a {} with value {:?}", type_name_of_val(&b), b);
    println!("c is a {} with value {:?}", type_name_of_val(&c), c);

    let mut list = Vec::new();
    list.push(a);
    list.push(b);
    list.push(c);

    println!("list is a {} with value {:?}", type_name_of_val(&list), list);
}
```

This snippet will print:

```ignore
a is a u128 with value 1
b is a u128 with value 3
c is a u128 with value 4
list is a alloc::vec::Vec<u128> with value [1, 3, 4]
```

Two instances of automated inference occurred here.

First, primitive types were inferred from a function signature.
If the function `sum` wasn't part of the program, `let a = 1;` would be equivalent to `let a: i32 = 1;`.
`i32`, a 4-byte signed integer, is Rust's default integer type.
But, because of the line `let c = sum(a, b)`, the compiler realized that `a` is actually a `u128`, a 16-byte unsigned integer.

Second, the type of a dynamic collection was inferred from the type of the item stored.
All three of the below statements are equivalent:

* `let mut list = Vec::new();` - inferred type (like the above).
* `let mut list: Vec<u128> = Vec::new();` - explicit type annotation.
* `let mut list = Vec::<u128>::new();` - explicit constructor.

We got to use the convenient inferred shorthand because the example program had at least one `list.push()` statement.
The compiler looked at the type of items being pushed to the vector, `u128` integers in this case, and decided the type of the vector.

> **What about heterogeneous collections?**
>
> If we wanted a vector to store items of varying but logically related types, we couldn't rely on type inference.
> We'd have to explicitly use the `dyn` keyword and something called a "trait object".
> That's not a language feature we'll need or cover in this book.

## Tuples vs. Arrays

Rust provides two ways to represent an ordered, fixed-size sequences of values: tuples and arrays.

* Tuples can group multiple values of *different* types, but can only be indexed by a *constant*.

* Arrays can group multiple values of *only the same* type, but can be indexed by a *variable*.

### Tuples

There's no hard-and-fast rule for when to use which, but tuples are particularly useful as a return type.
For cases where a function should return multiple values.

[Slightly contrived] example: say we need to compute lengths for the sides of a 30-60-90 triangle (a special "right triangle"[^Triangle]) based on its shortest side.
There's a known formula:

```rust,noplaypen
// Side proportions are 1 : 2 : square_root(3)
fn compute_30_60_90_tri_side_len(short_side: f64) -> (f64, f64, f64) {
  (
    short_side,
    short_side * 2.0,
    short_side * 3_f64.sqrt() // "_f64" is an optional type postfix syntax
  )
}

fn main() {
  let tri_sides = compute_30_60_90_tri_side_len(10.0);

  // Tuple constant indexing
  assert_eq!(tri_sides.0, 10.0);
  assert_eq!(tri_sides.1, 20.0);
  assert_eq!(tri_sides.2, 17.32050807568877);

  // Tuple destructuring
  let (a, b, c) = compute_30_60_90_tri_side_len(10.0);

  assert_eq!(a, 10.0);
  assert_eq!(b, 20.0);
  assert_eq!(c, 17.32050807568877);
}
```

Function `compute_30_60_90_tri_side_len` returns three values: the length of 3 sides of a triangle.
In our first call to this function, the inferred type for variable `tri_sides` is `(f64, f64, f64)`.
Each float is accessible by constant position, but not by a variable (e.g. `tri_sides.1` works, but `tri_sides.i` or `tri_sides[i]` would not).

We could have defined a structure with named fields, but tuples provide a concise shorthand.
And we can set names with a technique called *destructuring*, demonstrated by our second call to `compute_30_60_90_tri_side_len`.
Instead of assigning to a single tuple variable, we destructure and assign each tuple item to its own named variable (e.g. `a`, `b`, and `c`).

### Arrays

Arrays are a general-purpose data structure you've likely seen in other programming languages, so we won't dwell on them here.
The Rust syntax for arrays declaration is `[T; N]`.
Each value stored is of type `T` and `N` is the length of the array.
It works like so:

```rust,noplaypen
// Explicit array type declaration
let numbers: [u64; 3] = [42, 1337, 0];

// Inferred array type (`[&str; 4]`, array of read-only string references)
let operating_systems = ["Linux", "FreeBSD", "Tock", "VxWorks"];

// Initialization of all elements (1,000 of them) to a single value (0)
let mut buffer = [0; 1_000];

// Index-based write access
for i in 0..1_000 {
  assert_eq!(buffer[i], 0); // Should have been zero-initialized
  buffer[i] = i; // Overwrite with new value
}

assert_eq!(buffer[0], 0);
assert_eq!(buffer[1], 1);
assert_eq!(buffer[2], 2);

// Iterator-based write access
for num in buffer.iter_mut() {
  *num += 7; // "*" is a dereference for write
}

assert_eq!(buffer[0], 7);
assert_eq!(buffer[1], 8);
assert_eq!(buffer[2], 9);
```

The above uses two loops to modify the contents of a 1,000 item array.
The first uses traditional, index-based access (e.g. `buffer[i]`).
The second uses an *iterator* (e.g. `buffer.iter_mut()`) to perform a similar operation.

Iterators enable functional programming constructs, like `map` and `filter`.
While that entails a performance penalty in many languages, you'll see these constructs used often in idiomatic Rust.
Because they can actually result in *faster* code.

Why?
There's an implicit contract in the first loop above: `i` has to be smaller than the length of the array.
Otherwise we'd *write out-of-bounds*, past the end of the array.
To ensure safety, the compiler has to add a run-time bounds check to the first loop (but not the second).
That check has a cost.
We'll see what failing the check looks like when discussing error handling later in this chapter.

> **Arrays vs. Vectors**
>
> Unlike the `Vec` we added items to when discussion type inference, arrays cannot grow dynamically.
> Their capacity is fixed.
> While that constraint can be inconvenient, it makes arrays portable - don't need to rely on runtime libraries for dynamic memory allocation to use arrays.

One major difference between Rust and C arrays is that the former have length explicitly encoded as part of the type.
This has several advantages, one of which is compliance with:

> **[AR, Rule 17.5]** Arrays used as function parameters must have the correct number of elements[^MISRA_2012]

## References

We already introduced references in the previous chapter, in the context of a function that increments an integer.
They're a modern alternative to raw pointers:

```rust,noplaypen
fn incr(a: &mut isize, b: &isize) {
    *a += *b;
}

fn main() {
  let mut x = 3;
  let y = 5;

  incr(&mut x, &y);

  assert_eq!(x, 8);
  assert_eq!(y, 5);
}
```

References are crucial for systems programming.
Recall that they enable *pass-by-reference* semantics (hand off a "pointer"), instead of *pass-by-value* (copy the entire value).
That level of control is essential, it enables performant manipulation of large values.
The programmer can choose when to perform a *shallow copy* (duplicate only a reference) and when to perform a *deep copy* (duplicate all data).
The former means less time spent copying bytes and less total memory used.

We'll return to the topic of references when discussing ownership later in this chapter.
When dealing with ownership errors, you'll quickly realize that Rust strongly encourages this MISRA rule:

> **[AR, Rule 8.13]** References should be immutable whenever possible[^MISRA_2012]

## Slices

Slices are a concept closely related to references, they also come in immutable and mutable variants:

* `&[T]` is an immutable, shared slice of `T`s.

* `&mut [T]` is a mutable, exclusive slice of `T`s.

Both slice types are "partial views" into a sequence of values that are stored within some other, larger value.
Let's make sense of that statement with an example:

```rust,noplaypen
// Array of 5 items
let mut buffer_overflow_defenses = [
    "stack canary",
    "ASLR",
    "NX bit",
    "CFI",
    "Intel CET",
    "ARM MTE",
];

// Create an immutable slice of the first 3
// [..=2] is inclusive range notation, equivalent to [..3]
let basic_defenses = &buffer_overflow_defenses[..=2];

assert_eq!(basic_defenses, &["stack canary", "ASLR", "NX bit"]);

// Create an mutable slice of the last 2
let advanced_defenses = &mut buffer_overflow_defenses[4..];

assert_eq!(advanced_defenses, &mut ["Intel CET", "ARM MTE"]);

// Modify via slice
advanced_defenses[1] = "safe Rust!";

// Notice both slice and it's "backing storage" are updated
assert_eq!(advanced_defenses, &mut ["Intel CET", "safe Rust!"]);
assert_eq!(buffer_overflow_defenses[5], "safe Rust!");
```

Sub-division of a larger sequence is one convenient use of slices, as demonstrated above.
You might recall seeing slice range notation (e.g. `[..=2]` and `[3..]`) in the previous chapter as well.
We used it in IETF test vector validation, to grab 16-byte chunks out of the RC4 key stream.

Slices are also useful in creating idiomatic APIs.
We leveraged this approach when defining parameters to our RC4 functions (like `new` and `apply_keystream`), but didn't explain the rationale in detail.
Consider the below:

```rust,noplaypen
fn count_total_bytes(byte_slice: &[u8]) -> usize {
    let mut cnt = 0;

    // Underscore indicates unused variable
    for _ in byte_slice {
        cnt += 1;
    }

    // Oops - we didn't need to loop, there's a built-in length method!
    assert_eq!(cnt, byte_slice.len());

    cnt
}

fn main() {
    let byte_arr: [u8; 4] = [0xC, 0xA, 0xF, 0xE];

    // Vec init shorthand
    let mut byte_vec = vec![0xB, 0xA, 0xD];

    // Push more data dynamically
    byte_vec.push(0xF);
    byte_vec.push(0x0);
    byte_vec.push(0x0);
    byte_vec.push(0xD);

    // Note both types can be borrowed as &[u8]
    assert_eq!(count_total_bytes(&byte_arr), 4);
    assert_eq!(count_total_bytes(&byte_vec), 7);
}
```

The advantage of slices in parameter signatures is that different kinds of collections can be *borrowed as a slice*.
In the above, we wrote one function that works for both dynamic vectors of bytes and fixed-size arrays of bytes.

Finally, we'd be remiss if we didn't mention the relationship between strings (`String` type) and string slices (`&str` type).
A proper discussion of the topic involves a fair bit of complexity, and strings aren't particularly relevant to code we'll be writing in this book.
Though the data structures we build can certainly store strings, we'll forgo a detailed discussion and recommend section 8.2 of the official Rust book[^TRPL] - "Storing UTF-8 Encoded Text with Strings" - if you're interested.

> **The `vec!` Macro**
>
> The above code includes short-hand notation for initializing a vector of elements.
> `let mut byte_vec = vec![0xB, 0xA, 0xD];` is equivalent to:
>
> ```rust,noplaypen
> let mut byte_vec = Vec::new();
> byte_vec.push(0xB);
> byte_vec.push(0xA);
> byte_vec.push(0xD);
> ```
>
> In fact, our `main` function above could have avoided `push` calls entirely with:
>
> ```rust,noplaypen
> let mut byte_vec = vec![0xB, 0xA, 0xD, 0xF, 0x0, 0x0, 0xD];
> ```
>
> This syntax may look similar to `byte_arr`'s initialization, but don't confuse the two: arrays have a fixed capacity, we can't `push` new items to an array after initialization.

## Takeaway

We've briefly covered primitives (focusing on integers), tuples, arrays, references, and slices.
And gotten a feel for type inference along the way.
You've now seen low-level techniques for representing and manipulating data in Rust.

Instead of spending dozens more pages on the intricacies, we'll move on to more exciting and interesting features of the language: ways to express higher-level constructs.

You'll master all of these topics through hands-on experience as we progress through the book.
Our present goal is to rapidly survey Rust's fundamentals.

---

[^IntWrap]: [*Myths and Legends about Integer Overflow in Rust*](https://huonw.github.io/blog/2016/04/myths-and-legends-about-integer-overflow-in-rust/). Huon Wilson (2016).

[^MISRA_2012]: *MISRA C: 2012 Guidelines for the use of the C language in critical systems (3rd edition)*. MISRA (2019).

[^IndexAside]: Well, that's true in the case of a `Vec`. Under-the-hood `Vec` is a *fat pointer* (memory address, length, and capacity) to an array allocated on the heap. Indexing `my_vec[i]` involves computing an offset to a memory location. But for custom containers you define, overloading the index operator can perform any operation that makes logical sense in the context of your container. We'll implement our own indexing logic for ordered maps and sets later in the book.

[^TraitInto]: [*Trait `std::convert::Into`*](https://doc.rust-lang.org/std/convert/trait.Into.html). The Rust Team (Accessed 2022).

[^TraitFrom]: [*Trait `std::convert::From`*](https://doc.rust-lang.org/std/convert/trait.From.html). The Rust Team (Accessed 2022).

[^TypeInference]: [*Type inference*](https://rustc-dev-guide.rust-lang.org/type-inference.html#type-inference). Guide to Rustc Development (Accessed 2022). Rust uses an extension of the Hindley-Milner type inference algorithm[^HMT].

[^Triangle]: [*30° - 60°- 90° Triangle*](https://www.mathopenref.com/triangle306090.html). Math Open Reference (Accessed 2022).

[^HMT]: [*Hindley–Milner type system*](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system). Wikipedia (Accessed 2022).

[^IEEEFloat]: [*IEEE 754-2008 revision*](https://en.wikipedia.org/wiki/IEEE_754-2008_revision). Wikipedia (Accessed 2022).

[^TRPL]: [*Storing UTF-8 Encoded Text with Strings*](https://doc.rust-lang.org/book/ch08-02-strings.html). by Steve Klabnik, Carol Nichols (Accessed 2022).
