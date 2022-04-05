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

# Rust Zero-Crash Course
---

> **Note:** this chapter's content may be subject to revision.

The prior chapter walked through a Rust library and CLI tool in the service of introducing software security concepts.
This chapter will focus on the Rust language proper - we'll tour its syntax, features, and conventions.

Now we won't be covering all of Rust.
It's a *big* language.
Much closer to C++ than C.
Our favorite comprehensive Rust book, *Programming Rust*[^ProgRust], is a 700+ page tome, a relentless enumeration of language features.
Now it's a phenomenal book, and a major inspiration for this one.
But we're talking multiple-Costco-mini-barrels-of-whole-bean-coffee kinds of endurance.

Part of the challenge is the sheer breadth of features Rust offers.
Being a relatively new language, Rust has the benefit of hindsight: it's free to cherry-pick successful aspects of its predecessors.

This includes OCaml's algebraic data types, C++'s monomorphization, Scheme's hygienic macros, etc[^RustInfluences].
While the Rust Team strives for a cohesive design[^BuildJosh], the language juggles several influences.

Fortunately for us, you don't need an exhaustive understanding of Rust to be productive in it.
This section previews key concepts - just enough to get you started reading and writing Rust snippets.
We'll cement those concepts in the rest of the book by building an embedded-friendly, high assurance library.

With that as a foundation, you'll be prepared to write your own real-world Rust programs.
And to tackle learning additional language features (smart pointers, channels, async, macros, etc) as project needs arise.

Our tour of Rust will be broken into six short-ish parts:

1. **Low-Level Data Representation** - primitives, tuples, arrays, references, and slices.

2. **High-level Data Representation** - structs, enums, generics, and traits.

3. **Control Flow** - conditional statements, loops, and pattern matching.

4. **Ownership Principles** - understanding the core principles of Rust's most novel feature.

5. **Ownership in Practice** - concepts for working with ownership day-to-day.

6. **Error Handling** - propagating failures and/or maintaining availability.

## Emphasizing Field-tested Assurance Guidelines

This book is an introduction to building robust, reliable, and secure systems.
Hence the *zero-crash* pun in this chapter's title.

To emphasize actionable assurance techniques, we'll frame our Rust tour in the context of a well-established industry standard.
On wisdom tested in the most unforgiving of production environments.

The Motor Industry Software Reliability Association (MISRA) C[^MISRA] guidelines are a set of C software development rules originally created for, as the acronym implies, the automotive industry.

Unlike a *style* guide, MISRA C outlines *best practices* for developers of safety-critical systems.
It's intended to maximize reliability, security, and maintainability.
For systems in which a bug can potentially endanger lives.

Today, these guidelines are widely used in the aerospace, defense, telecommunication, and medical device industries (in addition to industry-specific frameworks like DO-178C[^DO-178C] and ISO-26262[^ISO-26262]).
The most recent version[^MISRA_2012] introduces itself as:

> The MISRA C Guidelines define a subset of the C language in which the opportunity to make mistakes is either removed or reduced.
> Many standards for the development of safety-related software require, or recommend, the use of a language subset, and this can also be used to develop any application with security, high integrity or high reliability requirements.

MISRA C has been tested and refined over decades.
Even outside of regulatory certification, these are practical guidelines for building high assurance systems.

Rust's core design is directly applicable to building safe, reliable software.
Because we won't use the `unsafe` keyword, you could say that this book introduces a safe subset of the Rust language.

> **A "Safe Subset" for Our Purposes**
>
> What truly constitutes a "safe subset" of the Rust programming language is the subject of current standardization and research efforts.
> We will not attempt to formally define a safe subset in this book.
>
> Instead, for our core project, we'll use two crate-wide macros to restrict ourselves to what a practicing engineer could consider a "safe subset":
>
> * `#![forbid(unsafe_code)]`: Usage of the `unsafe` keyword is a compile-time error. This helps us maximize Rust's static guarantees.
>
> * `#![no_std]`: We don't use standard library facilities (which contain `unsafe` code). More strictly, we opt out of all dynamic memory usage. Not relying on an external allocator has certain robustness benefits.
>
> Because the core project is based on an open-source library[^Scapegoat], we know that working within these constraints is viable for non-trivial codebases.

Now, being a new language, Rust is not yet certified for use in a safety-critical setting - although this is an area of industry effort[^Ferrocene] and research[^RustCrit].
There is no Rust counterpart to the MISRA C guidelines.
Yet.

Many MISRA C rules are specific to the C language.
We'll split a portion of the remainder into two categories, using these labels to differentiate:

* **Automated by Rust (AR):** Rule that is easy to follow consistently or natural to express in idiomatic Rust. For any Rust program, if it compiles it likely adheres to this category.

* **Reliably for Rust (RR):** Rule generally applicable to the design and implementation programs prioritizing correctness and robustness. Can be readily applied in Rust, just not automatically. Conscious effort is required on the programmer's part.

As we introduce a safe subset of the Rust language, we'll occasionally highlight an applicable MISRA C[^MISRA_2012] rule.
Both in this chapter and throughout the book, preceded by one the the labels above.

As a preview - here's three MISRA C rules we'll conform to for the core library we write (but not for development tools we build or use, since those aren't safety-critical):

> **[RR, Directive 4.1]** Minimize runtime failures[^MISRA_2012]

> **[RR, Directive 4.12]** Do not use dynamic memory allocation[^MISRA_2012]

> **[RR, Rule 17.2]** Functions can't call themselves recursively (directly or indirectly)[^MISRA_2012]

Note we're omitting rationale.
Which can be convincing, if the above three rules appear restrictive.
Fortunately, Rust makes it feasible and ergonomic to meet this sort of high assurance criteria.

> **A Distinct Take on MISRA C**
>
> To be cautious of respecting copyright, we'll only provide a rough paraphrase of each MISRA rule's "heading" - not its exact phrasing, full explanation, rationale, exceptions, category, etc.
> This is the same approach taken by academic publications[^MisraPub] that enumerate MISRA rules.
>
> In several cases, our paraphrase will introduce Rust-specific terminology not present in the MISRA C Guidelines.
> Unlike prior work mapping MISRA rules to Rust[^MisraRust], we're not aiming to be exhaustive.
> We're sampling rules for the purpose of learning assurance concepts.

We can taxonomize the MISRA rules and directives mentioned in this chapter as follows:

<br>
<p align="center">
  <img width="100%" src="misra.svg">
  <figure>
  <figcaption><center>A visual breakdown of our MISRA sample (rules mentioned in this chapter).</center></figcaption><br>
  </figure>
</p>

At a high-level, **directives** are MISRA rules that are difficult to describe in a definitive, universal way.
Directives tend to be harder to check and validate in a complex system.
**Rules**, on the other hand, are possible to completely capture.
They can often be validated with accuracy by static analysis tools (like the Rust compiler).

Again, note that our sample of MISRA rules and directives isn't exhaustive.
If you're a professional safety or security engineer, we recommend purchasing the full MISRA C 2012 Guidelines from MISRA itself.
Understanding widely-adopted best practices is valuable, regardless of the specific toolchain a project uses.

## Software Engineering in Rust

High assurance or not, modern development is about more than language syntax and language features.
It involves tools, processes, and, most importantly, people: external customers and internal teams.

Professional experience is the best way to learn how to implement effective processes and serve the needs of stakeholders.
We'll focus on tools in this book.

Using `clap` in the last chapter already gave us a taste for integrating 3rd party libraries into our builds.
We also leveraged Rust's built-in, 1st-party unit testing framework to verify our RC4 implementation against official test vectors.
Yet we've only scratched the surface on what `cargo` can do to aid day-to-day development tasks.
The Cargo Book[^CargoBook] offers a more complete overview.

In this chapter, we'll highlight a few more components of Rust's tooling ecosystem, both 1st and 3rd party.
We'll also discuss Rust's release cycle to understand how stability is enabled for production systems.
More generally, we'll cover a key pillar of successful software projects: code organization.

## Brief Prerequisites: Stack, Heap, Values

This chapter will occasionally use two technical terms: "stack" and "heap".
In this context, these terms refer to two kinds of *distinct memory locations*.
Not the data structures of the same name (an unfortunate jargon overload).

The next chapter will discuss memory in detail.
For now, think of it like this:

* **Stack memory** is short-term (live for function call duration) storage that's readily available. However, it can **only store fixed-size** variables.

    * The mechanics of the stack are closely related to CPU hardware. In fact, many processors have a specific register called a "stack pointer".

    * Stack memory works like the stack data structure - memory "frames" are *Last In First Out (LIFO).*

    * Integers and arrays are stored on the stack by default.

* **Heap memory** is long-term (live until freed) storage that has to be requested explicitly and cleaned up later. But it can store variables whose **size is decided at runtime**.

    * The mechanics of the heap are handled by software, but map to DRAM hardware. A memory allocation library, typically working in tandem with the OS[^Malloc], implements complex logic to manage chunks of RAM.

    * Vectors and non-literal strings are typically stored on the heap.

The stack/heap distinction is a computer architecture concern that needs to show up in the syntax of systems programming languages.
Programming "close to the metal" requires a mental model that reflects the hardware/software interface.

"Value" is another term this chapter makes use of.
It's a concept that spans every kind of programming language:

* A **value** is a memory-location-independent *concrete instance* of typed data.

For example, in `let string_literal = "Hello, World!";`, `string_literal` is a *variable* (label) assigned the *value* `"Hello, World!"`. This value has two parts:

1. A type (here, `T: &'static str` - we'll break down how to read that signature)

2. A concrete bit-pattern (whatever encodes the specific UTF-8 string `"Hello, World!"`).

With that out of the way, let's start the zero-crash course.

## Learning Outcomes

* Learn key guidelines for writing high assurance software.
* Understand "Undefined Behavior" and its implications.
* Learn core Rust language features, get comfortable reading/writing Rust snippets.
* Learn must-have Rust tooling to ease day-to-day software engineering tasks.

---

[^ProgRust]: [***[PERSONAL FAVORITE]** Programming Rust: Fast, Safe Systems Development*](https://amzn.to/35cu1Za). Jim Blandy, Jason Orendorff, Leonora Tindall (2021).

[^RustInfluences]: [*The Rust Reference: Influences*](https://doc.rust-lang.org/reference/influences.html). The Rust Team (2021)

[^BuildJosh]: [*Josh Triplett on Building the Build System of his Dreams*](https://anchor.fm/building-with-rust/episodes/Josh-Triplett-on-Building-the-Build-System-of-his-Dreams-e1dt81c). Sean Chen (2022).

[^MISRA]: [*MISRA C*](https://www.misra.org.uk/misra-c/). MISRA (Accessed 2022).

[^DO-178C]: [*DO-178C*](https://en.wikipedia.org/wiki/DO-178C). Wikipedia (Accessed 2022).

[^ISO-26262]: [*ISO 26262*](https://en.wikipedia.org/wiki/ISO_26262). Wikipedia (Accessed 2022).

[^MISRA_2012]: *MISRA C: 2012 Guidelines for the use of the C language in critical systems (3rd edition)*. MISRA (2019).

[^Scapegoat]: [*`scapegoat`*](https://github.com/tnballo/scapegoat). Tiemoko Ballo (Accessed 2022).

[^Ferrocene]: [Ferrocene](https://ferrous-systems.com/ferrocene/). Ferrous Systems (2021).

[^RustCrit]: [*Towards Rust for Critical Systems*](https://ieeexplore.ieee.org/document/8990314). Andre Pinho, Luis Couto, Jose Oliveira (2019).

[^MisraPub]: [*The MISRA C Coding Standard and its Role in the Development and Analysis of Safety- and Security-Critical Embedded Software*](https://arxiv.org/pdf/1809.00821.pdf). Roberto Bagnara, Abramo Bagnara, and Patricia Hill (2018).

[^MisraRust]: [*MISRA-Rust*](https://github.com/PolySync/misra-rust). Shea Newton (Accessed 2022).

[^CargoBook]: [*The Cargo Book*](https://doc.rust-lang.org/cargo/). The Cargo Team (Accessed 2022).

[^Malloc]: A user space "memory allocator" can issue "system calls" to an OS to grow heap capacity as needed. If your program uses heap memory, it must link against this runtime support library. This is extremely common, it's how most programs work.