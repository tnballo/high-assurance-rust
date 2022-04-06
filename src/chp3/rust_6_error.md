# Rust: Error Handling (6 of 6)

Detecting and handling errors is fundamental to software development in general, but it's an especially pressing topic for software prioritizing robustness and availability.
Error handling is also one area where Rust differentiates itself - both in mechanism and meticulousness.

Broadly speaking, we can bin errors into one of three classes:

1. **Compile-time errors** - Syntax or ownership errors that prevent a single module from compiling. Rust's compiler tends to output actionable error messages in these cases. Of which you'll likely see many, especially when first learning the language. Just remember: you're aiding a safety verification process.

2. **Link-time errors** - Symbol resolution errors that prevent multiple modules from composing. Thanks to `cargo`, linking errors should be a rarity when working on pure-Rust codebases. But they may appear in large, multilingual projects or when using C/C++ libraries as dependencies.

3. **Run-time errors** - Errors caused by a broken invariant or an operation failure, at runtime. This class impacts assurance. It's the subject of this section, we'll look at strategies for handling run-time errors in Rust.

Notice that *logical errors* (e.g. implementing an incorrect algorithm) are not listed above.
We consider these to be general *bugs* and outside the scope of an error handling discussion.

For errors proper, some developer communities make the below distinction:

* "Error" referring specifically to catastrophic failures that a program cannot reasonably handle (e.g. exhausting system memory).

* "Exceptions" being errors that can be "caught" and handled by programmer-defined logic (e.g. a file doesn't exist).

We don't make that distinction here.
We'll use the term "error" to capture both the catastrophic and handleable cases.

## `Option` vs `Result`

Rust's standard library provides two `enum` types for expressing fallible operations: `Option`[^Option] and `Result`[^Result].
Strictly speaking, *error handling* refers only to `Result`.
But the two are conceptually similar and widely used as function return types, so we'll cover both now.

### `Option`

`Option` conveys that a function could, potentially, have nothing to return.
Even though the operation was completed successfully.
That's normal behavior.

Since we've now covered both enumerations and generics, try interpreting the definition[^Option] of this standard library type:

```rust,noplaypen
pub enum Option<T> {
    None,
    Some(T),
}
```

Notice how the `None` variant of `Option`'s definition doesn't contain data.
This definition encodes the concept of "some type `T` XOR nothing".
Ideal for a fallible operation that may return a result.

An example, one we'll become intimately familiar with later, is an ordered set's `get` method.
Retrieval of an element returns `None` if that element isn't present the set:

```rust,noplaypen
use std::collections::BTreeSet;

let set = BTreeSet::from([1, 2, 3]);

assert_eq!(set.get(&2), Some(&2));
assert_eq!(set.get(&4), None);
```

> **Conceptual Checkpoint**
>
> There are intricacies in the above `BTreeSet` usage snippet, related to concepts we introduced in this chapter.
> Let's solidify understanding:
>
> * `let set: BTreeSet<i32> = ...` is inferred. `i32` is Rust's default integer type and we're creating a set from an array of 3 integer literals.
>
> * Thus, `get` returns `Option<&i32>` here. The reference operator, `&`, in this return signature ensures retrieval doesn't *move* the element out of the set. The set still *owns* it, we're just checking if it's present.
>
>   * To actually remove the element we'd use a different set method, `take`, which returns `Option<T>` (`Option<i32>` in our example) and transfers ownership.
>
> * Similarly, the argument to `get` is of type `&i32` (hence `set.get(&2`) - we don't want the `get` function to take ownership of the element we're searching for.
>
>   * Why, given that primitive integers can be *copied* cheaply? Because `BTreeSet<T>` is a *generic* container. Items stored in the set could be large and complex objects, not just `i32`s.

### `Result`

Now `Result` has an entirely different use case.
It conveys that a function could fail to complete an operation.
A failure is abnormal, it means a problem needs to be reported or an operation needs to be retried.

In `Result`'s definition[^Result], both variants contain data.
The `Ok` variant encapsulates the output of a successful operation, whereas the `Err` variant signals failure and encapsulates a custom error type:

```rust,noplaypen
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

An example, one we've already seen in the context of Chapter 2's CLI tool, is file I/O.
Attempting to open a file can fail for several reasons - the file might not exist or we do not have permission to read it.
We previously used the `?` operator to short-circuit error propagation, but we could also explicitly match the file open `Result` like so:

```rust
use std::fs::File;

match File::open("/path/to/non-existent/file.txt") {
    Ok(f) => println!("Successfully opened: {:?}", f),
    Err(e) => eprintln!("Error occurred: {:?}", e),
}
```

Unlike `Option`, `Result` is marked with the `#[must_use]` attribute internally.
Whenever you write a function that returns a `Result`, the caller *must* explicitly handle both the `Ok` and `Err` cases.
This built-in enforcement lends itself to another MISRA rule:

> **[AR, Directive 4.7]** Always test error information returned from functions[^MISRA_2012]

While `Result` provides a convenient mechanism for representing potential failures, and automatically enforces handling, we're still left with the application-specific task of doing the error handling.
Generally, we can take one of three approaches:

1. **Assert invariants** - Terminate the program immediately if an error occurs. Useful when errors cannot be reasonably recovered from.

2. **Merge and propagate** - Merge multiple kinds of errors into a single, opaque error and pass it along to the caller. Useful when we want to abstract away irrelevant details, but still give the caller a chance to respond.

2. **Enumerate and propagate** - Pass along detailed error information to the caller. Useful when the caller's response action depends on the exact kind of error that occurred.

To make each approach more concrete, and explore some of the finer details, we'll make modifications to Chapter 2's RC4 library and the corresponding CLI tool.

> **Rust Errors vs C++ Exceptions**
>
> C++ allows two error handling strategies[^CppExcep]:
>
> 1. **Return codes:** A function can return a special value, like `-1` or `NULL`, to implicitly indicate an error has occurred. But the developer must remember to check for this special case at every callsite and interpret its meaning.
>
>       * Accidentally omitting the check is a common violation of Directive 4.7 above, in both C and C++.
>
> 2. **Thrown exceptions:** exceptions *must* be caught either by a programmer-defined handler or, if none is provided, the OS itself. So handling is enforced. And they may provide descriptive context.
>       * However, C++ exceptions occur outside of regular code flow - one might be propagated from a function so deeply nested that it appears unrelated. This introduces "invisible" exit points for functions, which both violates a different MISRA rule (one we haven't mentioned) and causes some C++ programmers to consider using exceptions a "bad practice".
>
>       * Additionally, unwinding is a performance bottleneck on multi-core systems (due to a global lock)[^CppSlowExcep].
>
> With `Result`, Rust offers the best of both worlds.
> Like return codes, `Result` is passed up via the regular call chain.
> Like C++ exceptions, `Result` can't be accidentally ignored and, via the `Err` variant, provides meaningful context.

## Assert Invariants

In the previous chapter, we wrote a constructor for an RC4 cipher instance.
By convention, constructors are associated functions named `new`.
Our `new` function took a single parameter, a key byte array, and asserted an invariant:

```rust,ignore
pub fn new(key: &[u8]) -> Self {
    // Verify valid key length (40 to 2048 bits)
    assert!(5 <= key.len() && key.len() <= 256);

    // ...more code here...
}
```

On one hand, this adheres to an important rule (input validation):

> **[RR, Directive 4.14]** External inputs must be validated[^MISRA_2012]

On the other hand, we made a debatable decision on behalf of our library's users: if the provided key was too short or too long, we'd terminate the program.
Users won't have a chance to respond if this error condition is hit.

For certain catastrophic failure cases, the Rust language itself makes a similar decision.
For example, say we indexed an array out-of-bounds:

```rust,ignore
let mut five_item_arr = [0; 5];

for i in 0..6 {
    five_item_arr[i] = i;
}
```

The loop will run for 6 iterations, `i == 0` through `i == 5`, but the array only has 5 valid indexes (`0` through `4`).
This program will *compile successfully* but *terminate at runtime* with:

```ignore
thread 'main' panicked at 'index out of bounds: the len is 5 but the index is 5', src/main.rs:7:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

It's a classic "off-by-one" error.
Testing would have helped catch this indexing failure.
But not all fatal invariants are easy to test for, so most real-world programs will contain some assertion-based error handling.
Including implicit cases like this one.

One goal of testing is to show that a program is robust enough to not hit such assertions in practice, due to checks and/or mitigations.
Some number of fatal assertions will always be present, but thorough testing can give us confidence that a program avoids them.

Now in certain cases, we may be able to remove problem potential entirely.
For example, we could have initialized the array using an iterator to eliminate the possibility of an out-of-bounds index:

```rust,noplaypen
let mut five_item_arr = [0; 5];

for (i, item) in five_item_arr.iter_mut().enumerate() {
    *item = i;
}
```

Now let's look at the non-fatal cases - errors we can detect and propagate.
We'll refactor our RC4 constructor to demonstrate error propagation strategies.

## Merge and Propagate

Recall that if a provided key wasn't the right size, our Chapter 2 RC4 CLI gave the user a descriptive error - essentially re-prompting for a valid-length key.
We accomplished that with `clap`'s `min_values = 5` and `max_values = 256` annotations.

Our library itself (not the CLI front-end) asserted the invariant.
The front-end's check just ensured this assertion would never trigger.

Say we wanted the library to enforce a similar check for any program that uses it, front-end or otherwise.
We could have it propagate a single, opaque error like so:

```rust,ignore
impl Rc4 {
    /// Init a new Rc4 stream cipher instance
    pub fn new(key: &[u8]) -> Result<Self, ()> {
        // Verify valid key length (40 to 2048 bits)
        if (key.len() < 5) || (key.len() > 256) {
            return Err(());
        }

        // Zero-init our struct
        let mut rc4 = Rc4 {
            s: [0; 256],
            i: 0,
            j: 0,
        };

        // ...more initialization code here...

        // Return our initialized Rc4
        Ok(rc4)
    }
}
```

Choosing the unit type (`()`, an empty value) instead of a custom error type is a "bare bones" approach.
One typically better-suited in private, internal APIs.
But it does the job, since the caller has to take appropriate action for both the `Ok` and `Err` variants of the returned `Result`.
The `Ok` variant contains a successfully-initialized cipher.

## Enumerate and Propagate

For public APIs, a custom error `enum` is likely preferable to `()`:

```rust,ignore
{{#include ../../code_snippets/chp3/rc4/src/lib.rs:new_error_handling}}
```

In the above, we've opted to enumerate both error conditions (too short and too long) instead using a single `KeyLengthInvalid` variant or similar.
Each variant also contains the threshold length, a minimum for the `KeyTooShort` variant and a maximum for `KeyTooLong`.

That level of granularity may or may not be appropriate in this context.
It's definitely not a common pattern in stream cipher libraries.
But our example demonstrates enumerating various internal errors and passing them along.

It allows a caller to `match` on error `enum` variants and handle each case accordingly.
Notionally, that'd be something akin to:

```rust,ignore
use rc4::{Rc4, Rc4Error};

let key = [0x1, 0x2, 0x3];

match Rc4::new(&key) {
    Ok(rc4) => println!("Do en/decryption here!"),
    Err(e) => match e {
        Rc4Error::KeyTooShort(min) => eprintln!("Key len >= {} bytes required!", min),
        Rc4Error::KeyTooLong(max) => eprintln!("Key len <= {} bytes required!", max),
    },
}
```

### The `Error` Trait

There's one more important piece to the Rust's error handling puzzle: the `Error` *trait* in defined in the standard library[^Error].
Implementing this special trait for our `Rc4Error` type would have two advantages:

* Clearly marking `Rc4Error` as an error type - not just an `enum` that happens to have `Error` in the name.

* Enabling richer error reporting, via the `source` and [currently unstable] `backtrace` methods of the trait.

However, there's a good reason we won't use this trait in our RC4 library.
Recall that our cipher implementation is `#![no_std]` compatible - it can run any environment, even "bare metal".

The `Error` trait assumes the presence of an operating system, whose runtime support is needed to capture and print a backtrace.
Thus we can't import `std::error::Error` in a `#![no_std]` library.

> **Can't we support that use case?**
>
> If omitting the `Error` trait strikes you as an unsatisfying compromise, you try *feature-gating* support for this trait as an exercise.
> That'll entail modifying the `Cargo.toml`[^Features] build file and implementing the trait behind a `cfg` macro[^CondComp].
> By convention, this feature would be called `std` and selected with:
>
> ```ignore
> cargo build --features="std"
> ```
>
> A dependency could chose to enable the optional feature within it's own `Cargo.toml` entry:
>
> ```ignore
> [dependencies]
> rc4 = { path = "../rc4", version = "0.1.0", features = ["std"] }
> ```
>
> This enables the best of both worlds - support embedded systems by default, but allow richer error reporting if a library user enables an optional feature when building for non-embedded targets.

## Takeaway

Rust's `Result` type, not to be confused with the conceptually similar `Option`, is our main mechanism for reporting run-time errors and enforcing their handling.
Like C++ exceptions, it can't be ignored.
Unlike C++ exceptions, it's part of the regular call chain.

Error handling is essential for assurance, but the specific actions to be taken are ultimately application specific.
We can choose the best approach for each situation: asserting invariants, propagating an opaque error, or propagating specific errors.

That concludes our six-part tour of Rust's core concepts!
The rest of this chapter looks at features and tools that help us to build large, ambitious systems in the language.

---

[^Option]: [*Enum `std::option::Option`*](https://doc.rust-lang.org/std/option/enum.Option.html). The Rust Team (Accessed 2022).

[^Result]: [*Enum `std::error::Error`*](https://doc.rust-lang.org/std/error/trait.Error.html). The Rust Team (Accessed 2022).

[^MISRA_2012]: *MISRA C: 2012 Guidelines for the use of the C language in critical systems (3rd edition)*. MISRA (2019).

[^CppExcep]: [*C++ Exceptions: Pros and Cons*](https://www.codeproject.com/Articles/38449/C-Exceptions-Pros-and-Cons). Nemanja Trifunovic (2009).

[^CppSlowExcep]: [*P2544R0 C++ exceptions are becoming more and more problematic*](http://www.open-std.org/jtc1/sc22/wg21/docs/papers/2022/p2544r0.html). Thomas Neumann (2022).

[^Error]: [*Enum `std::error::Error`*](https://doc.rust-lang.org/std/error/trait.Error.html). The Rust Team (Accessed 2022).

[^Features]: [*Features*](https://doc.rust-lang.org/cargo/reference/features.html). The Cargo Book (Accessed 2022).

[^CondComp]: [*Conditional compilation*](https://doc.rust-lang.org/reference/conditional-compilation.html). The Rust Reference (Accessed 2022).