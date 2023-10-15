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


# Fundamentals: Type Systems

> **Note:** This section is a work-in-progress. It may be expanded or revised to cover more formal aspects, like type rules, in the future.

Rust's secret sauce is its type system.
So we should discuss types.
This is a dense topic whose dedicated field, type theory, predates computer programming.
We won't do it justice in a handful of pages.

Static type systems are perhaps the most widespread and powerful form of static analysis in existence.
Let's think of types as having two jobs:

## 1. Mapping abstract data to a physical machine

Types are specifications for how data is read and written.
At the mechanical level of hardware.
In memory, every construct is just a bit pattern - a sequence of `0`s and `1`s.
Types provide language-level overlays suited to human reasoning.

For example: integer types interpret sequences of bits, 64 at time on modern "64-bit" machines, as whole numbers.
They can be operated on mathematically (addition, subtraction, multiplication, etc) when stored in "registers" (think tiny, readily-accessible, CPU-specific chunks).

Let's revisit the `incr` function from Chapter 2.
We had a function taking pointers to integers as arguments.
In C, the below code couldn't guarantee that pointers `a` and `b` don't alias.
Or that either pointer refers to a valid memory location.

```c
void incr(int* a, int* b) {
    *a += *b;
}
```

The Rust port eliminated both of those problems:

```rust
fn incr(a: &mut isize, b: &isize) {
    *a += *b;
}
```

Both languages have type systems that do semantic superimposition.
They map source code operations to physical hardware operations:

1. Read bit-patterns from RAM address (values of integers types `a` and `b`) into registers (dereference reads).

2. Add the values of the two CPU registers, as if whole numbers, using a CPU instruction (mathematical operation).

3. Writes the result back to memory (dereference write).

## 2. Verifying program behavior by elimination

Types have another job in addition to, or perhaps in unison with, working out how the hardware sausage is made.
They verify what programs will do, by elimination.
A seminal textbook on the subject[^TPL] suggests:

> A type system is a tractable syntactic method for proving the absence of certain program behaviors by classifying phrases according to the kinds of values they compute.

That essentially means types can constrain possible behaviors, so you can be confident certain things won't happen.
In the case of the Rust `incr` function, that means eliminating two problem states (aliasing and invalid pointers) completely.

How do we prove absence of certain behaviors?
At a high level: by *grouping* values based on desired behavior.
For example:

* **Grouping:** Values `0`, `1`, `2`, ..., `255` can be grouped into the type `u8` (8-bit unsigned integer).

* **Proving absence of a behavior:** The `+` operator applied to `u8` operands performs *addition*, not *concatenation*. Thus we guarantee the program will never concatenate two unsigned bytes - that operation has no meaning in the language.

The difference between two broad classes of type systems, static and dynamic, comes down to *how* we do that proving of absence:

* **Static typing** does the proving at compile-time. Guaranteeing the program will never exhibit a behavior at runtime.

    * Variables have types. As a consequence, so do values. And types are known at compile-time, for every possible execution.

* **Dynamic typing** tags values with types at runtime. The legality of operations is checked during program execution. If the check fails, the program may terminate or throw an exception.

    * Values have types, variables do not. And the type of a value is only known at runtime.

## Case Study: Dynamic Typing

Sometimes the best way to internalize an idea is to look at a counter example.
Contrast can be illuminating.
Let's step away from Rust's static types for a minute.

Python is a scripting language with a beginner-friendly syntax and a large professional user base.
Unlike Rust, it's dynamically typed.
For many projects, this reduces development friction and improves prototyping speed.
The interpreter's abstractions allow developers to focus on the product they ship - not the machine they run it on.

But there's a world of use cases, from low-power embedded sensing to high-performance distributed workloads, for which Python is wholly unsuitable.
High assurance applications are one subset.
The relatively sluggish performance is a factor, but poor reliability is the bigger downside.
Let's see why.

Start by firing up the Python Read Execute Print Loop (REPL) with the command:

```ignore
python3
```

The REPL allows us to execute programs as we enter them, a convenient workflow enabled by dynamic typing.
Declare two variables, `word` and `x`, and inspect their types:

```python
>>> word = "Hello"
>>> print(type(word))
<class 'str'>
>>> x = 3
>>> print(type(x))
<class 'int'>
```

Multiplying `word` (a string) by `x` (an integer) is legal.
Here's the result:

```python
>>> print(word*x)
HelloHelloHello
```

But consider multiplying a string by a string (`word` by itself).
That doesn't generally make sense, so it will result in an error.
Rust will catch this error at compile time, long before we ship our code.
But Python throws it at *runtime*, and only if that particular line gets executed:

```python
>>> print(word*word)
Traceback (most recent call last):
  File "<stdin>", line 1, in <module>
TypeError: can't multiply sequence by non-int of type 'str'
```

For high assurance software, that's too late.
A single type error, hit on a code path not covered in our test suite, means service degradation or outage.
Without a static type system, executing uncommon or untested paths through a program is akin to "flying by the seat of your pants".
And if we refactor dynamically typed code against an incomplete test suite, we might actually be adding those rare paths.

> **Can't we just get 100% test coverage?**
>
> Achieving 100% coverage can be impractical for large projects.
> Even if we could, the state-space of a program (set of all possible states) isn't necessarily correlated with its coverage (set of statements executed).
> That means we could pass tests with full coverage and still hit a type error at runtime, in production.

## Goldilocks Assurance Contrast

In the 1970s, compilers for industrial applications were largely uncharted waters.
Most compile-time verification we take for granted today was still decades of research away.
Runtime checks were prohibitively expensive without a compiler sophisticated enough to know where they're not needed.
Thus, C's **weak, static typing** means subdued safety enforcement.
C types can be converted, or cast, implicitly - and thus often erroneously.

Weak typing, coupled with other design choices, has resulted in a language which allows for a devastating amount of **Undefined Behavior (UB)**[^UndefResearch]: C programs can exhibit unpredictable behavior at runtime.
Due to "gaps" in the language specification, not problems with the implementation of any specific compiler.
And these gaps can't be fixed retro-actively, that might break existing C programs we all rely on.

The "Goldilocks Principle"[^GoldiPrin], named after a child's fable[^Goldi], reflects a cross-domain understanding: we want to optimize for "just the right amount" of a property.
For us, that property is assurance under performance constraints.
Our three bears are C, Python, and Rust.
A rough high-level comparison:

| | C | Python | **Rust** |
| --- | --- | --- | --- |
| *Type-safe?* |  No (Weak, **Static**) | No (**Strong**, Dynamic) | **Yes** (**Strong**, **Static**) |
| *Memory-safe?* | No | **Yes** | **Yes** |
| *Fast?* | **Yes** | No  | **Yes** |

> **Doesn't Python support optional static typing?**
>
> A peer-reviewed, large-scale analysis[^Py3Types] of real-world Python projects found that a small minority use type annotations (3.8%), those that do rarely use them correctly enough to pass a type check (15% of the 3.8%), and that popular type checkers (`MyPy` and `PyType`) produce false positives (44-49% of the time).
>
> Worse yet, Python type checkers often disagree with each other.
> Optional typing isn't a viable substitute for a compiler-enforced static type system - especially in a high assurance context.

There's a great deal of nuance which the table doesn't capture.
But we'll use it to wrap up this aside on types and reliability.

> **How about Go?**
>
> Go is a popular, modern, statically-typed, natively-compiled programming language.
> It has fantastic concurrency support.
> But garbage collection makes it unsuitable for a wide range of systems programming tasks.
> Go has to "pause" your entire program at unpredictable internals and execute an algorithm to clean up memory.
> This is often unacceptable for real-time and low-latency systems.
>
> Rust helps you wrangle memory at compile time, inserting allocation/deallocation logic based on variable scope.
> The result is predictable performance.
> Discord, makers of a popular chat application, found that Go's garbage collection wasn't compatible with a service's performance targets.
> They re-wrote the service in Rust to eliminate latency spikes[^DiscGo].

## Takeaway

At a mechanical level, type systems power the machinery by which data is read and written.
At a more abstract level, type systems ensure certain undesirable outcomes don't occur - so long as programmers don't introduce typecasting bugs.

Dynamically typed languages, which do type checking at runtime, introduce a reliability risk.
Previously explored paths and states can introduce crashes or exceptions.

Weak static typing, where type casting with little restriction is tolerated, is similarly risky.
It can introduce UB.
Whose consequences include crashes, incorrect results, and security vulnerabilities.

---

[^TPL]: [*Types and Programming Languages*](https://www.cis.upenn.edu/~bcpierce/tapl/). Benjamin C. Pierce (2002).

[^UndefResearch]: [*Undefined Behavior: What Happened to My Code?*](https://people.csail.mit.edu/nickolai/papers/wang-undef-2012-08-21.pdf). Xi Wang, Haogang Chen, Alvin Cheung, Zhihao Jia, Nickolai Zeldovich, M. Frans Kaashoek (2012).

[^GoldiPrin]: [*Goldilocks principle*](https://en.wikipedia.org/wiki/Goldilocks_principle). Wikipedia (2021)

[^Goldi]: [*Goldilocks and the Three Bears*](https://en.wikipedia.org/wiki/Goldilocks_and_the_Three_Bears). Robert Southey (1837).

[^Py3Types]: [*Python 3 Types in the Wild: A Tale of Two Type Systems*](https://www.cs.rpi.edu/~milanova/docs/dls2020.pdf). Ingkarat Rak-amnouykit, Daniel McCrevan, Ana Milanova, Martin Hirzel, Julian Dolby (2020).

[^DiscGo]: [*Why Discord is switching from Go to Rust*](https://blog.discord.com/why-discord-is-switching-from-go-to-rust-a190bbca2b1f). Jesse Howarth (2020).
