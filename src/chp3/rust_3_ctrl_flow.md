# Rust: Control Flow (3 of 6)

Almost[^BranchProg] any useful program is going to make some decision based on a condition, or execute some logic multiple times.
Thus every imperative programming language offers some mechanism for determining *control flow*: deciding the order in which individual statements get executed.

Languages tend to settle on the same handful of constructs for expressing control flow.
Rust is no exception.
Its *pattern matching* may be new to you depending on what language you're coming from, but its conditional statements and loops should feel familiar.

## Conditional Statements

The `if` and `else` keywords work much like you'd expect.

```rust
fn conditional_print(num: usize) {
    if num > 10 {
        println!("{} is greater than 10.", num);
    } else if num % 2 == 0 {
        println!("{} is even.", num);
    } else {
        println!("{} is odd.", num);
    }
}

fn main() {
    conditional_print(11);
    conditional_print(4);
    conditional_print(5);
}
```

The above outputs:

```ignore
11 is greater than 10.
4 is even.
5 is odd.
```

What differentiates Rust is that the condition after the `if` keyword *must* evaluate to a `bool` type.
There's no implicit casting allowed.
This strictness helps obey another MISRA rule:

> **[AR, Rule 14.4]** *If* expressions must evaluate to boolean types[^MISRA_2012]

Many other languages don't enforce strict typing for conditional statements.

* In Python, a `None` value is implicitly cast to `false` if a condition evaluates to it.

* Likewise, in C, a zero integer is implicitly cast to `false` (and a non-zero is cast to `true`).

This doesn't hamper our ability to express a condition in Rust.
`x == None` and `y != 0` can still be written out explicitly.
But it does eliminate one potential source of error.

## While Loops

The `while` keyword lets us continue executing a loop as long as a boolean condition holds.
The below prints a countdown from 10 to 1:

```rust
let mut countdown = 10;

while countdown > 0 {
    println!("{}...", countdown);
    countdown -= 1;
}
```

Rust doesn't support "do while" loops directly, but the same logic can be implemented with the `loop` and `break` keywords.
An equivalent countdown could be implemented as:

```rust
let mut countdown = 10;

loop {
    println!("{}...", countdown);
    countdown -= 1;
    if countdown == 0 {
        break;
    }
}
```

## For Loops

The `for` keyword enables looping over any *iterable*.
Take a range for example.
The below prints the numbers 0 through 9:

```rust
for i in 0..10 {
    println!("{}", i);
}
```

What if we want to access the elements of a collection in a loop?
On the surface, our `for` syntax seems to "just work":

```rust
use std::collections::{HashSet, BTreeSet};

// List
let list = vec![3, 2, 1];

println!("Iterating over vector:");

for item in list {
    println!("list item: {}", item);
}

// Ordered set
let mut o_set = BTreeSet::new();
o_set.insert(3);
o_set.insert(2);
o_set.insert(1);

println!("\nIterating over ordered set:");

for elem in o_set {
    println!("set element: {}", elem);
}

// Hash set
let mut h_set = HashSet::new();
h_set.insert(3);
h_set.insert(2);
h_set.insert(1);

println!("\nIterating over hash set:");

for elem in h_set {
    println!("set element: {}", elem);
}
```

But consider the output of the above:

```ignore
Iterating over vector:
list item: 3
list item: 2
list item: 1

Iterating over ordered set:
set element: 1
set element: 2
set element: 3

Iterating over hash set:
set element: 2
set element: 3
set element: 1
```

Each collection has its own strategy for accessing elements:

* `Vec` (a list) returns its values in the order they were inserted.
* `BTreeSet` (an ordered set) returns values in sorted order, relative to each other.
* `HashSet` (a hash set) doesn't have any notion of order - either sort or insertion.

Under-the-hood, each collection implements its own *iterator*.
Each has its own logic, but shares a common interface: the `Iterator` trait[^TraitIterator].
The `for` loop leverages this interface to perform traversal of the underlying data structure.

Iterators are a key part of idiomatic Rust, we'll dedicate an entire chapter to implementing our own.
For now, know that they enable a world of conveniences.
like enumeration:

```rust
let list = vec![3, 2, 1];

for (i, item) in list.iter().enumerate() {
    println!("list item {}: {}", i, item);
}

// Prints:
//
// list item 0: 3
// list item 1: 2
// list item 2: 1
```

And functional transformations:

```rust
let list = vec![3, 2, 1];

let triple_list: Vec<_> = list.iter().map(|x| x * 3).collect();

for item in triple_list {
    println!("triple_list item: {}", item);
}

// Prints:
// triple_list item: 9
// triple_list item: 6
// triple_list item: 3
```

Iterators also prevent common errors, like Out-Of-Bounds (OOB) indexing.
The help us comply with:

> **[AR, Rule 14.2]** *for* loops must be well-formed[^MISRA_2012]

## Pattern Matching

In its simplest usage, pattern matching is akin to C's `switch` statement - we chose one action from a finite set.

We saw `match`-ing on `enum` variants in the previous section.
This can be a convenient way to take different actions based on domain-specific context.
To review:

```rust,ignore
#[derive(Debug)]
pub enum State {
    Running,
    Stopped,
    Sleeping,
}

fn do_something_based_on_state(curr_state: State, pid: u32) {
    match curr_state {
        State::Running => stop_running_process(pid),
        State::Stopped => restart_stopped_process(pid),
        State::Sleeping => wake_sleeping_process(pid),
    }
}
```

Unlike a C `switch`, pattern matching allows us to specify a list of *expressions* and a corresponding action for each.
Expressions can encode relatively complex conditions succinctly.
For example:

```rust
let x = 10;

match x {
    1 | 2 | 3 => println!("number is 1 or 2 or 3"),
    4..=10 => println!("number is between 4 and 10 inclusive"),
    x if x * x < 250 => println!("number squared is less than 250"),
    _ => println!("number didn't meet any previous condition!"),
}
```

* The 1st *match arm* (`1 | 2 | 3 => ...`) specifies three literal values. It triggers if the matched variable, `x`, equals any of the three.

* The 2nd arm specifies a range, 4 to 10 inclusive. It triggers if `x` is any value within the range.

* The 3rd arm uses a *guard expression*. It triggers if `x` multiplied by itself is less than 250.

* The 4th and final arm is a *default case*. It matches *anything* using the wildcard `_`. It's only triggered if none of the previous cases trigger.

Note that an input can't match multiple arms, only the first pattern it conforms to.
Thus order matters.

Rust also requires matches to be *exhaustive*, meaning the programmer has to handle every possible case.
Exhaustive matching of `State` variants in the first example was easy, there are only three: `Running`, `Stopped`, and `Sleeping`.

In the second example,`let x = 10;` didn't specify a type for `x`.
So the compiler inferred `i32` by default.
Exhaustively matching every possible value of a 32-bit unsigned integer would be tedious - instead, each of our patterns covers a subset of possible values.

The fourth pattern, a wildcard default, is required to ensure we don't miss anything.
If that line was omitted, we couldn't handle the case where `x` is `16`, for example.

The exhaustiveness requirement ensures any `match` we write gracefully handles any possible input, which meets the spirit of another MISRA rule:

> **[AR, Rule 16.4]** Switch statements must have a default case[^MISRA_2012]

While the rule is specific to C's `switch` statement, the idea robust of matching carries over - we should never accidentally "fall through" a `switch`/`match` without taking an appropriate action.

## Condensed Pattern Matching

Rust offers constructs for condensing pattern matching to a single, conditional action - triggered when a specific pattern fits (ignoring the rest).
If you see `if let` and `while let` in Rust code, it's a shorthand for "drilling down" to a single `match` arm.

This syntax can be obtuse when starting out, so we'll gradually introduce it later in the book - in the context of a larger program.
As a preview, consider this code (assume we're using our `State` enum from before):

```rust,ignore
let curr_state = State::Running;

match curr_state {
    State::Running => println!("Process is running!"),
    State::Stopped => {},   // Do nothing
    State::Sleeping => {},  // Do nothing
};
```

It's equivalent to this shorthand:

```rust,ignore
let curr_state = State::Running;

if let State::Running = curr_state {
    println!("Process is running!");
}
```

Notice how we print a message only for a `Running` state, but we don't have to exhaustively `match` different cases.
Instead, `if let` allows conditional action only for a specific `enum` variant.

Aren't we losing robustness by ignoring the other cases, in light of the previous MISRA rule?
Perhaps surprisingly, not quite.

* `if let` is like any other `if` statement in that the body is only executed if a specific condition is true. By design, it's not intended to be exhaustive. `if` only "cares" about one case. And that's obvious to a reader.

* A `match` supports multiple patterns and doesn't know which its input will trigger. By design, it's responsible for handling all of them. So the compiler enforces exhaustiveness. Something a reader might otherwise miss.

Deciding whether `match` or `if let` is appropriate depends on the context of the broader program.

## Takeaway

Rust's control flow constructs aren't vastly different from other programming languages.
`while` loops work like you'd expect, `for` loops are backed by iterators, "do while" can be emulated with alternative syntax.
There's a bit more strictness - conditions must evaluate to booleans and pattern matching must be exhaustive if not using `if let`.
Rust encourages a notion of correctness.

Pattern matching may be new to you, depending on your background.
Its uses vary from simple switching on variants to complex matching of intricate patterns.
But you probably won't need complex patterns often.
And when you do, you'll be glad the feature exists!

We've covered data representation and control flow.
It's time to dig into what makes Rust unique.
The language's most distinctive and novel feature: ownership.

---

[^BranchProg]: [*Branchless programming. Does it really matter?*](https://dev.to/jobinrjohnson/branchless-programming-does-it-really-matter-20j4). Jobin Johnson (2021).

[^MISRA_2012]: *MISRA C: 2012 Guidelines for the use of the C language in critical systems (3rd edition)*. MISRA (2019).

[^TraitIterator]: [*Trait `std::iter::Iterator`*](https://doc.rust-lang.org/std/iter/trait.Iterator.html). The Rust Team (Accessed 2022).