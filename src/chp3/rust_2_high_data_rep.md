<meta name="title" content="High Assurance Rust">
<meta name="description" content="Developing Secure and Robust Software">
<meta property="og:title" content="High Assurance Rust">
<meta property="og:description" content="Developing Secure and Robust Software">
<meta property="og:type" content="article">
<meta property="og:url" content="https://highassurance.rs/">
<meta property="og:image" content="https://highassurance.rs/img/har_logo_social.png">
<meta name="twitter:title" content="High Assurance Rust">
<meta name="twitter:description" content="Developing Secure and Robust Software">
<meta name="twitter:url" content="https://highassurance.rs/">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:image" content="https://highassurance.rs/img/har_logo_social.png">


# Rust: High-Level Data (2 of 6)

We saw low-level fundamentals in the last section.
They're important and common.
But where Rust really starts to shine is the higher-level constructs: "custom" data types that map more closely to our problem domain.

Rust draws influence from functional languages like ML, OCaml, and Haskell - among others[^RustInfluence].
It brings to the table some interesting, perhaps even exotic, constructs.
Features we don't often see in performant systems languages.

We'll ease our way into some of these constructs in this section, assuming no prior familiarity with functional languages.

## Enums

Enumerations, "enums" for short, let you define a type whose possible values are a set of named constants.
In their most basic usage, Rust enums are similar to enums present in most other languages.

We're going to be using a running example for the next handful of sections - that of an Operating System (OS) capable of running several processes (in-memory, isolated instances of programs).
To show how constructs of Rust code can map to a specific domain[^TockOS].
And to learn or review a few OS concepts along the way.

Let's assume a process can, at any given time, be in one of three states:

1. **Running** - Currently executing on a CPU core.

2. **Stopped** - Suspended indefinitely (perhaps the user pressed `Ctrl+Z`).

3. **Sleeping** - Suspended temporarily (perhaps it's waiting for data, like user input, to become available).

Enums are a natural way to express mutually exclusive but related possibilities.
We can declare a `State` enum with three *variants* (named constants `Running`, `Stopped`, and `Sleeping`):

```rust,noplaypen
{{#include ../../code_snippets/chp3/proc/src/main.rs:state}}
```

An OS needs to take different actions depending on what state a process is currently in.
For example, when an internal timer goes off (e.g. an "interrupt fires"), it may be time to stop a currently running process, save its state, and run/restore a different process.
CPU time is a shared resource, processes need to take turns.

Rust supports *pattern matching* as a means to conditionally decide which logic should be executed.
One common use is matching on the variant of an enum.
For example, the OS could execute a different function depending on the state of a process:

```rust,ignore
fn manage_process(curr_state: State) {
    match curr_state {
        State::Running => stop_and_schedule_another_process(),
        State::Stopped => assign_to_available_cpu_core(),
        State::Sleeping => check_if_data_ready_and_wake_if_so(),
    }
}
```

Each line inside the `match` brackets is called an *arm*.
The *pattern* is to the left of the arrow operator (`=>`) and the code executed if the pattern *matches* is to the right.
We'll discuss pattern matching in more detail in the next section, which covers control flow.

What differentiates Rust enums from those of C, C++, and many other languages is their ability to encapsulate additional data of varying types.
This ability makes Rust enums akin to "sum types" in functional languages (which are a specific kind of "algebraic data type").
In practice, what that means is we have the flexibility to store arbitrary data in each variant.
That data could even be another enum!

Let's say we had design requirements for a more granular process state representation.
Specifically, say an OS needs to:

* Track two kinds of stop requests: those that can be ignored by the process and those that can't.

* Record a start timestamp for sleeping processes, to later calculate how long a sleeping process has been inactive.

We could replace our `State` enum with a `DetailedState` that reflects the new requirements:

```rust,noplaypen
{{#include ../../code_snippets/chp3/proc/src/main.rs:detailed_state}}
```

Notice how the `Stopped` variant now contains another enum (`StopKind` - ignore the `#[derive(...` above it for now) and the `Sleeping` variant now contains a `u64` timestamp (akin to UNIX's epoch representation[^Epoch]).
Yet the `Running` variant remains empty.

We can freely choose data types encapsulated within variants and can still "pull out" the inner type when matching.
The below snippet is a test where the first arm checks the `Stopped` variant's inner data.
The second arm uses a wildcard (`_`) to assert that this test won't match against any other variants (since `state` is hardcoded).

```rust,ignore
#[test]
fn test_detailed_stop_match() {
    let state = DetailedState::Stopped {
        reason: StopKind::Mandatory,
    };
    match state {
        DetailedState::Stopped { reason } => {
            assert_eq!(reason, StopKind::Mandatory);
        }
        _ => unreachable!(), // Will panic at runtime if reached
    }
}
```

One devilish detail: the in-memory size of an enum is determined by its largest variant.
An instance of the `Running` variant is the same size as an instance of `Sleeping` variant, despite the latter holding more information.
Memory layout isn't something you'll need to think about often, but it's worth noting.
We may be using fancy sum types, but we're still writing low-level code.

## Structures

Structures, specifically name-field structs like the below, are the primary way you'll represent data in most Rust programs.
Rust structs serve the same purpose as Python classes or Java objects - they're a way to group data and functions that operate on that data[^EnumAside].

One of main responsibilities of an OS kernel is *task scheduling* - deciding which process (or its threads) should be running on which CPU core and for how long.
Many programs are composed of multiple processes, a *parent process* can create one or more *child processes*.

If we were implementing an OS, we'd likely want to group process-relevant data into a struct.
A simplified example[^LinuxTaskStruct] could look like this:

```rust,ignore
pub struct Proc {
    pid: u32,           // Process ID (unsigned integer)
    state: State,       // Current state (enum)
    children: Vec<u32>, // Child IDs (dynamic list)
}
```

> **How do multiprocess programs work?**
>
> One program (parent process) can start (e.g. "spawn") a second helper program (child process).
> If the helper is doing independent work, they can both run *simultaneously* on a modern, multi-core system.
> The parent runs on one core while the child runs on another.
>
> That's what helps your web browser feel faster and more responsive.
> By default, the Chromium runs one process per website connected to[^ChromeProc].

The `Proc` struct represents a concept from our problem domain (the idea of an OS-managed process) as typed data.
To make working with the data easier, we'd likely add methods (have `self` parameter) and associated functions (no `self` parameter) - just like we did with the `Rc4` struct in the last chapter.
Both types of functions must be defined within a struct's `impl` block.
For example:

```rust,ignore
impl Proc {
    /// Associated function (constructor)
    pub fn new(pid: u32) -> Self {
        Proc {
            pid,
            state: State::Stopped,
            children: Vec::new()
        }
    }

    /// Method (takes self, mutable setter in this case)
    pub fn set_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    // ...more methods/functions here
}
```

Note that named fields (`pid`, `state`, and `children`) are private by default.
They can only be accessed by code in the *module* in which the struct is defined.
Modules are a way to group related code, think of them as Rust's version of namespaces.

If this code where in another module that imported `Proc`, it would not compile because the private field `state` cannot be assigned to:

```rust,ignore
use my_os_module::Proc;

let mut my_proc = Proc::new(0);
my_proc.state = State::Running;
```

That's why we defined a setter method, the below would work:

```rust,ignore
use my_os_module::Proc;

let mut my_proc = Proc::new(0);
my_proc.set_state(State::Running);
```

This manner of *data encapsulation*[^DataEncap] is considered a best practice for public APIs.
But it's not required, nor is it always appropriate.
If we wanted `state` to be writable by external code (e.g. `my_proc.state = State::Running;` to work), we could use the `pub` visibility specifier at declaration:

```rust,ignore
pub struct Proc {
    pid: u32,           // Process ID (unsigned integer)
    pub state: State,   // Current state (enum)
    children: Vec<u32>, // Child IDs (dynamic list)
}
```

We'll discuss modules and visibility later in this chapter.

Notice how Rust takes a conservative approach: external visibility, mutability, and unsafety all require explicit opt-in.
This is a conscious design choice that helps reduce potential sources of error in large programs.

## Generics

We've already used a generic library: standard's `Vec`.
It's defined as `Vec<T>`, where `T` is a generic type.
That's why we can have both a vector of unsigned integers (`Vec<usize>`) and a vector of strings (`Vec<String>`), without needing to use a different library API for each type of item we want to store.

Imagine that instead of writing a single hobby OS for yourself, you're actually writing a reusable scheduling library - code that can potentially be leveraged by anyone writing an OS.
This is where *generics* would come in.
Instead of creating a specific instance of a structure or function, you can define a *template* that users of your code can plug *types* into.
Including custom types defined in by external code written in the future!

Maybe some of your users are writing an OS for tiny embedded devices that will never have more than 100 processes running simultaneously.
They need to save precious memory by using a `u8` to represent `pid`, instead of a `u32`.
But we can't just change `pid`'s type to a `u8` - other users need to represent thousands of processes.
Updating `Proc`'s definition and implementation to be generic lets us accommodate both groups:

```rust,ignore
pub struct Proc<T> {
    pid: T,             // Process ID (generic)
    pub state: State,   // Current state (enum)
    children: Vec<T>,   // Child IDs (dynamic list, generic)
}

impl<T> Proc<T> {
    // Associated function (constructor)
    pub fn new(pid: T) -> Self {
        Proc {
            pid,
            state: State::Stopped,
            children: Vec::new()
        }
    }

    // ...more methods/functions here
}
```

The resource-constrained users can specify `let mut my_proc: Proc<u8> = Proc::new(0);`, others can use `let mut my_proc: Proc<u32> = Proc::new(0);`.
Our code becomes flexible enough to work for either.

> **How do generics work in the final binary?**
>
> The Rust compiler implements generics via *monomorphization*.
> For each concrete type (like `u8`) used at any callsite, the compiler generates specialized code in the output binary.
> So generics have no runtime cost - each unique `T` "template" creates one "stamp" (unique code) in the final executable.

Generics are a core feature of Rust, you'll see them often.
Coupled with traits, they enable the creation of reusable, maintainable software components.

## Traits

The constructs we've discussed thus far haven't been that drastic a departure from the mainstream.
Rust's enums and pattern matching likely feel like an extension of language features you're already familiar with.
Traits are where, for many readers, Rust will start to feel significantly different.

We previously said that Rust structs fill the same role as Python classes and Java objects.
But unlike both of those languages, Rust doesn't support *inheritance*.
There are no class hierarchies, a struct can't inherit fields or methods from a parent.

Instead, shared behavior is defined by *composition*, via *traits*.
Some consider this approach a best practice, even in object-oriented languages[^Composition].

In terms of code-level mechanics, a trait is akin to an "abstract base class" in an object-oriented language.
Meaning it defines an interface (set of APIs) that any type implementing the trait must support.

Types can implement one or more traits, and doing so allows the type to be used in any context in which that trait is appropriate.

> **What is inheritance, again?**
>
> Inheritance, a kind of "subtype polymorphism", allows us to perform limited substitution of two types.
>
> Say a `Vehicle` class has the method `accelerate(int speed_mph)` and both `Car` and `Plane` subclasses inherit it.
> We want to write code that processes an array of `Vehicle` derivatives, calling `accelerate` on both `Car`s and `Plane`s.
> There's two ways for inheritance to achieve that goal, most languages offer both:
>
> * **Interface Inheritance:** `Car` and `Plane` share the public method interface of `Vehicle` but override the actual `accelerate` implementation with their respective customizations. Here, `Vehicle` acts as an "abstract base class". Rust's traits embody this best practice.
>
> * **Implementation Inheritance:** `Car` and `Plane` share the data and implementation of `Vehicle`'s generic `accelerate` method. This pattern is widely used in real-world programs, but the tight coupling of base and derived classes can make code more difficult to maintain and extend.

So what kind of behavior can traits specify?
And how do we make use of them?
We'll add two traits to our `Proc` struct to find out.

### Deriving Trait `Debug`

Being able to print out a text representation of a struct is useful for debugging.
In fact, it's a need so common that Rust provides a default format specifier specifically for this purpose: `{:?}`.
Let's try using it to print the original, non-generic `Proc` struct:

```rust,ignore
pub enum State {
    Running,
    Stopped,
    Sleeping,
}

pub struct Proc {
    pid: u32,           // Process ID (unsigned integer)
    state: State,       // Current state (enum)
    children: Vec<u32>, // Child IDs (dynamic list)
}

fn main() {
    let my_proc = Proc {
        pid: 1,
        state: State::Stopped,
        children: Vec::new(),
    };

    println!("{:?}", my_proc);
}
```

We get this error (some lines omitted):

```ignore
error[E0277]: `Proc` doesn't implement `Debug`
  --> src/main.rs:20:22
   |
20 |     println!("{:?}", my_proc);
   |                      ^^^^^^^ `Proc` cannot be formatted using `{:?}`
   |
   = help: the trait `Debug` is not implemented for `Proc`
   = note: add `#[derive(Debug)]` to `Proc` or manually `impl Debug for Proc`
```

If we want to use `{:?}`, compiler needs `Proc` to implement the `Debug` trait[^TraitDebug].
This trait defines how its implementor should be printed to the console, a common and desirable *behavior*.
At this point we have two options:

1. Review the documentation[^TraitDebug] for `std::fmt::Debug` to understand the interface it requires (in this case it's only one function) and implement the interface within a `impl Debug for Proc { ... }` block.

2. Attempt to *derive* the trait automatically, with the derive macro `#[derive(Debug)]`.

The latter option is easier, and is the route recommended by the documentation[^TraitDebug].

> **Getting familiar with Rust documentation**
>
> If you haven't done so already, take a second to review the documentation for the `Debug` trait[^TraitDebug] now.
> Although you won't understand the entirety of the function signature yet, you can still get a sense of the broad strokes.
>
> Understanding library documentation is a key skill for any developer, but it's especially useful for Rust programming.
> Popular libraries tend to be well-documented because Rust has a built-in, 1st party document generator (which we'll cover soon!).

Let's make the suggested update:

```rust,ignore
#[derive(Debug)]
pub struct Proc {
    pid: u32,           // Process ID (unsigned integer)
    state: State,       // Current state (enum)
    children: Vec<u32>, // Child IDs (dynamic list)
}
```

We now get a new error (aka a programmer's definition of "progress"):

```ignore
error[E0277]: `State` doesn't implement `Debug`
  --> src/main.rs:10:5
   |
7  | #[derive(Debug)]
   |          ----- in this derive macro expansion
...
10 |     state: State,       // Current state (enum)
   |     ^^^^^^^^^^^^ `State` cannot be formatted using `{:?}`
   |
   = help: the trait `Debug` is not implemented for `State`
   = note: add `#[derive(Debug)]` to `State` or manually `impl Debug for State`
```

Well, not entirely new. It's the same error as before, but this time for the `state` *field* of `Proc`.
Remember the idea of defining behavior by *composition*?

If every individual field of a struct implements the `Debug` trait, then deriving it for the entire struct is trivial - the behavior is simply a composite of the individual behaviors of each field.
We can build up powerful abstractions and reuse existing code, without the need to fit everything into a strict hierarchy.

Per this second error, our only remaining blocker is that the `State` type doesn't implement `Debug`.
Let's correct that:

```rust,noplaypen
#[derive(Debug)]
pub enum State {
    Running,
    Stopped,
    Sleeping,
}
```

The program will now compile and run.
We get the desired print output:

```ignore
Proc { pid: 1, state: Stopped, children: [] }
```

> **Quick Tip for `Debug` Printing**
>
> For debugging systems code, it's often useful to print structs with hexadecimal numerical values and one field per line.
> If we update the last line of `main` to `println!("{:#x?}", my_proc);`, the program prints:
>
> ```ignore
> Proc {
>   pid: 0x1,
>   state: Stopped,
>   children: [],
> }
> ```

### Implementing Trait `Ord`

Sometimes a trait can't be derived automatically.
Take, for example, the `Aead` trait[^TraitAead].
It's defined in a 3rd party library and specifies an [unofficial] interface for Authenticated Encryption with Associated Data (AEAD) ciphers.
Recall from the previous chapter that this is a family of cryptographic algorithms providing both message *confidentiality* and *integrity*[^AEAD].

Rust's trait system is powerful, but a derive macro isn't going to synthesize cryptographic code for us.
Traits are just interfaces, and we often need to implement the backing logic ourselves.

Moreover, even if a trait is derivable, the default behavior may not be what we want.
Say an OS needs to maintain a sorted list of process structures.
Sorting requires a notion of "order".
What mathematicians call "total order"[^TotalOrder].
The underlying idea is that we want to use logical comparison operators (`==`, `>`, `<=`, etc) to sort, and we must be able to make these comparisons unambiguously.

Rust's standard library includes a trait specifically for ordering: `Ord`[^Ord].
Any type that implements it becomes comparable to items of the same type, and collections of it can be sorted.
In many contexts, that's an incredibly useful behavior to support.

Can we derive `Ord` for `Proc`?
Yes.
But, per the documentation[^Ord], `Ord` depends on other traits: `PartialEq`, `Eq`, `PartialOrd`.
Because traits themselves can be defined by composition!

Let's not split hairs with the distinctions between these four order-related traits.
Instead consider what happens if we derive them:

```rust
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Running,
    Stopped,
    Sleeping,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Proc {
    pid: u32,           // Process ID (unsigned integer)
    state: State,       // Current state (enum)
    children: Vec<u32>, // Child IDs (dynamic list)
}

fn main() {
    let my_proc_stopped = Proc {
        pid: 1,
        state: State::Stopped,
        children: Vec::new(),
    };

    let my_proc_sleeping = Proc {
        pid: 3,
        state: State::Sleeping,
        children: Vec::new(),
    };

    let my_proc_running = Proc {
        pid: 2,
        state: State::Running,
        children: Vec::new(),
    };

    let mut proc_queue = vec![
        my_proc_stopped,
        my_proc_sleeping,
        my_proc_running,
    ];

    proc_queue.sort();

    println!("{:#?}", proc_queue);
}
```

The above creates a `Vec` of three processes (`proc_queue`) and sorts it.
Why is calling `proc_queue.sort()` possible?
Consider the function signature for `sort` from `Vec<T>`'s documentation[^VecSort]:

```rust,ignore
pub fn sort(&mut self)
where
    T: Ord,
{
    // ...code here
}
```

`where T: Ord` is a *trait bound*.
It stipulates what behavior `T` needs to support for the function to work.
That means `sort` is available on any `Vec<T>`, but *only if* `T` is a type implementing `Ord`.
The above code works because:

1. Type inference filled in `let mut proc_queue: Vec<Proc> = ...`.

2. The `Proc` struct derived the `Ord` trait.

Trait bounds major ramifications for code reuse and library composability.
`Vec` is a generic container (will work even for types that haven't been invented yet) and offers additional functionality for items that support specific behaviors (like sorting types that can be ordered).

But `Vec` isn't some one-off that only the official standard library can implement.
Any Rust developer can similarly use generics and traits to implement equally useful data structures.
We'll write an API-compatible alternative to another standard library collection in this book.

Trait bounds enable you to rapidly and confidently compose disparate components into large, harmonious systems.
They're a powerful high-level construct.

> **Reading Rust Syntax**
>
> It'll take time to get comfortable reading Rust, the syntax is complex.
> The `where` keyword is actually a readability convenience, the above `sort` signature is equivalent to:
>
>```rust,ignore
>pub fn sort<T: Ord>(&mut self) {
>    // ...code here
>}
>```
>
> But where did `T` come from?
> Neither `sort` variation is a stand-alone function, both exist within the `impl` block for `Vec<T>`.
> We omitted that detail for brevity, but it's consequential:
>
>```rust,ignore
> impl<T> Vec<T> {
>   pub fn sort<T: Ord>(&mut self) {
>       // ...code here
>   }
>
>   // ...other functions here
> }
>```

So, due to trait bounds, the call to `sort()` works.
But does it work *well*?
That's debatable, the output shows we've sorted by `pid`:

```ignore
[
    Proc {
        pid: 1,
        state: Stopped,
        children: [],
    },
    Proc {
        pid: 2,
        state: Running,
        children: [],
    },
    Proc {
        pid: 3,
        state: Sleeping,
        children: [],
    },
]
```

The derived composite behavior will attempt to sort by the 1st field of the struct (`pid`).
If the values happen to be equal, it will sort by the 2nd field (`state`, which also derives `Ord`).
If those values happen to be equal, then it will sort by the 3rd field (`children`), etc.

This compiled and ran, but it isn't quite the behavior we want.
Imagine our OS uses this list of processes as a scheduling queue, to decide which process to run next.
We'd need to sort them via some notion of priority, not `pid`-first.

Real-world scheduling algorithms can be complex[^SchedAlg].
For simplicity, let's assume we have three priorities based solely on the current `State`.
Any `Sleeping` process should be the highest priority for execution, followed by `Stopped` processes.
`Running` processes are, by definition, already running - they're the lowest priority.
We want them at the back of the list.
It's time to implement `Ord` the hard way!

First, we need to understand a little more about how the `State` enum works under-the-hood.
In memory, each variant starts with a *discriminant* - an integer number.
It's like a tag unique to the variant.

Had two `pid`s been equal, we'd need to break the sorting tie by looking at `state`.
Thus this discriminant integer would have come into play for sorting.
Let's keep the derived `Ord` on `State` but overwrite the default values to reflect our chosen priorities:

```rust,noplaypen
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Running = 3,    // 0 by default
    Stopped = 2,    // 1 by default
    Sleeping = 1,   // 2 by default
}
```

For the `Proc` struct, we'll now implement the actual functions required by the `Ord`[^TraitOrd], `PartialOrd`[^TraitPartialOrd], and `PartialEq`[^TraitPartialEq] traits - per the respective documentation.
We can still derive `Eq`[^TraitEq], because it's implied by `PartialEq` and has no methods of its own (a technicality that doesn't generalize to other traits):

```rust,ignore
use std::cmp::Ordering;

#[derive(Debug, Eq)]
pub struct Proc {
    pid: u32,           // Process ID (unsigned integer)
    state: State,       // Current state (enum)
    children: Vec<u32>, // Child IDs (dynamic list)
}

impl Ord for Proc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.state.cmp(&other.state)
    }
}

impl PartialOrd for Proc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Proc {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}
```

The details of the above code aren't as important as the implication: now the language, at a very fundamental level, will *only* consider the `state` field when ordering `Proc` structs.
By implementing a few specific traits, we've prescribed how the struct will *behave* in a range of contexts - like sorting and comparison.

With this new implementation of `Ord` and the traits it relies on, `println!("{:#?}", proc_queue);` will now output the `state`-prioritized order we desire:

```ignore
[
    Proc {
        pid: 3,
        state: Sleeping,
        children: [],
    },
    Proc {
        pid: 1,
        state: Stopped,
        children: [],
    },
    Proc {
        pid: 2,
        state: Running,
        children: [],
    },
]
```

> **Be careful, traits are powerful!**
>
> In implementing a trait manually, we've changed not only how `Proc` structs should be ordered for sorting but also what it means for two `Proc` structs to be equal!
>
> Now, any two structs with the same `state` are considered logically equivalent as far as the `==` operator is concerned, even if they have different `pid`s and `children`.
>
> Whenever you manually implement a trait, it's important to ensure all of the ramifications of that implementation are indeed appropriate for your program.
>
> In this case, trait implementation is actually overkill (we did it just to illustrate important concepts).
> Instead, we could've used `Vec`'s `sort_by_key` function[^VecSortByKey] after updating the enum discriminants:
>
> ```rust,ignore
> proc_queue.sort_by_key(|p| p.state);
> ```

## Takeaway

Rust's facilities for expressing high-level constructs include enums, structs, generics, and traits.
To recap:

* Enums are useful for representing a finite set of possible values, but may also carry additional data.

* Structs are a way to group related data and functions that operate on it, akin to classes or objects in other languages.

* Generics enable code reuse: functions and structures can be written only once yet support different types. While merely handy for avoiding code duplication, it's a truly killer feature for library design.

* Traits enable shared behavior via composition. They define specific interfaces, can be derived or implemented, and become especially useful when *bound* to generic parameters.

Let's take a breather to talk about a simpler topic, control flow, before we delve into ownership.

> **Can we encode domain-specific invariants directly into the type system?**
>
> In a limited yet potent fashion, yes.
> Sometimes important, domain-specific behavior can be modeled as a *state machine*.
> A structure that transitions through a sequence of states, in which certain operations can only be performed in certain states.
> And only certain transitions are legal.
>
> The *typestate pattern* is a way of encoding a structure's possible *runtime* states at *compile-time*.
> It can eliminate both state-related errors (static correctness) and the need for some runtime checks (performance). The former benefit is amenable to:
>
> **[RR, Directive 4.13]** Functions operating on a resource must be called in the correct sequence[^MISRA_2012]
>
> We'll cover Rust implementation of the typestate pattern in a future appendix section.

---

[^RustInfluence]: [*The Rust Reference: Influences*](https://doc.rust-lang.org/reference/influences.html). The Rust Team (2021).

[^TockOS]: [*Tock*](https://www.tockos.org/). Tock OS (Accessed 2022). Operating systems, perhaps the quintessential example of systems software, are a domain for which Rust is well-suited. There are several OSs written in Rust, Tock is one of them.

[^Epoch]: [*The Current Epoch Unix Timestamp*](https://www.unixtimestamp.com/). Dan's Tools (Accessed 2022).

[^EnumAside]: Rust also lets us define methods and associated functions on enums - we're not restricted to structs. But structs are more commonly used, many programming problems don't require representing groups of data that have multiple distinct variants.

[^LinuxTaskStruct]: Real OSs have much more complex task structures, our examples in this section are greatly simplified. If interested, you can check out the source code for Linux's `task_struct` [here](https://github.com/torvalds/linux/blob/4f12b742eb2b3a850ac8be7dc4ed52976fc6cb0b/include/linux/sched.h#L728).

[^ChromeProc]: [*Process Models*](https://dev.chromium.org/developers/design-documents/process-models). The Chromium Project (Accessed 2022).

[^DataEncap]: [*Data encapsulation*](https://en.wikipedia.org/wiki/Data_encapsulation). Wikipedia (Accessed 2022).

[^Composition]: [*Composition over inheritance*](https://en.wikipedia.org/wiki/Composition_over_inheritance). Wikipedia (Accessed 2022).

[^TraitDebug]: [*Trait `std::fmt::Debug`*](https://doc.rust-lang.org/std/fmt/trait.Debug.html). The Rust Team (Accessed 2022).

[^TraitAead]: [*Trait `aead::Aead`*](https://docs.rs/aead/latest/aead/trait.Aead.html). RustCrypto organization (Accessed 2022).

[^AEAD]: [*Authenticated encryption*](https://en.wikipedia.org/wiki/Authenticated_encryption). Wikipedia (Accessed 2022).

[^TotalOrder]: [Total order](https://en.wikipedia.org/wiki/Total_order) Wikipedia (Accessed 2022).

[^Ord]: [*Trait `std::cmp::Ord`*](https://doc.rust-lang.org/std/cmp/trait.Eq.html). The Rust Team (Accessed 2022).

[^VecSort]: [*`sort`*](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort). The Rust Team (Accessed 2022).

[^SchedAlg]: [*Scheduling Algorithms*](https://wiki.osdev.org/Scheduling_Algorithms). OSDev Wiki (2021).

[^TraitOrd]: [*Trait `std::cmp::Ord`*](https://doc.rust-lang.org/std/cmp/trait.Eq.html). The Rust Team (Accessed 2022).

[^TraitPartialOrd]: [*Trait `std::cmp::PartialOrd`*](https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html). The Rust Team (Accessed 2022).

[^TraitPartialEq]: [*Trait `std::cmp::PartialEq`*](https://doc.rust-lang.org/std/cmp/trait.PartialEq.html). The Rust Team (Accessed 2022).

[^TraitEq]: [*Trait `std::cmp::Eq`*](https://doc.rust-lang.org/std/cmp/trait.Eq.html). The Rust Team (Accessed 2022).

[^VecSortByKey]: [*`sort_by_key`*](https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by_key). The Rust Team (Accessed 2022).

[^MISRA_2012]: *MISRA C: 2012 Guidelines for the use of the C language in critical systems (3rd edition)*. MISRA (2019).
