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


# Dynamic Assurance (1 of 3)

Static analysis can be a tough topic to tackle, it's the tip of the iceberg for a world of theory and proofs that may seem divorced from the realities of day-to-day development.
But, as users of compilers, we benefit from type systems without having to grok all the implementation details.

By comparison, dynamic analysis is the fun and relatable counterpart.
Few developers implement their own static analyses.
Most professional developers write unit tests - little dynamic analyses that exercise a subset of the program in meaningful ways.

Dynamic analysis is conceptually easy to understand: learn about a program by executing it with concrete inputs and observing what happens.

* **Pro:** We can trust the results of each execution because it's a real-world run of the actual program. There are no false positives[^FalsePos].

* **Con:** Because we can only observe a single execution at a time, we're building confidence by repeatedly sampling from a pool of data points. But the complete pool is often massive and our sample is a miniscule one. So we can't draw general conclusions.

    * Dynamic analysis can prove the *presence* of one or more bugs. But it *cannot* prove the *absence* of any bug type.

> **What happens when we execute a program?**
>
> An orchestra of hardware and software components perform tasks and interact in complex ways.
> The 10,000 foot view is something like this:
>
> A *loader* copies an instance of the program into memory and sets up an isolated environment for it, "spawning a process".
> When the process is running and wants to write to disk or read network data, it elicits the cooperation of the *Operation System (OS).*
> That OS manages access to *hardware*, like the physical disk drive or network interface card.
> A relatively tiny state machine, the *Central Processing Unit (CPU)*, drives the entire sequence of events by rapidly switching between executing your program, hundreds of other programs, and the OS itself.
>
> Dynamic analyses are small programs that piggyback onto the *Program Under Test (PUT)*.
> They "hook into" the PUT as it runs, or perhaps before and after it runs, to record "live events".
> For example, a debugger can read the current values of variables at specific points in execution.
> A unit test can check return values of a specific function run with specific parameters.

## Let's Roll Some Cryptography in Rust!

Realistically, some non-trivial percentage of readers may never make it past this 2nd chapter.
Real-life priorities shift, learning a new language and a new skill set is a tough task to follow through on.

That's why you're going to write an interesting Rust program right away.
Let's build something real, something you can run.
Both as an end-user of a command line tool and as a tester validating a security-sensitive library.

We're going to write a tiny yet modular program.
It'll have two parts:

1. **Single-cipher cryptographic library:** A from-scratch, embedded-friendly implementation of RC4 - a famous but outdated stream cipher. Consider it a standalone crash course in writing tricky Rust code.

2. **A command-line interface:** A way to use your crypto lib to encrypt and decrypt files on your computer. Being able to perform argument parsing and file I/O opens the door to practical projects, in any new language.

Now cryptography is notoriously hard to get right.
And the Rust compiler, powerful as it is, can't statically reason about the correctness of a specific algorithm's implementation - stream cipher or otherwise.
This is where dynamic analysis comes in:

* We'll write a unit test showing input-output equivalence to a known-good RC4 implementation.

* To understand where dynamic analysis fails, we'll insert a naive backdoor into our library.

> **What's a stream cipher?**
>
> If this term is new to you or you'd just like a quick refresher, please read the [*Fundamentals: Stream Ciphers*](../chp16_appendix/crypto.md) section of the Appendix before proceeding.
> It briefly covers the background necessary to understand the cryptographic code in the next section.

## Setting Up Our Modular Project

You'll want to log into the development environment you set up at the end of chapter 1, and follow along from this point on.
Don't just skim the below, learn by doing!

Let's start by checking that the Rust toolchain is correctly installed.
What happens if you run the below command?

```ignore
rustup doc --std
```

You should see documentation for Rust's standard library open in a web browser.
This command is a handy one to remember.
You might need offline-accessible documentation if you've ever coding ~~in an air-gapped secure facility~~ on a plane.

Next we'll use `cargo`, Rust's package manager, to create a "workspace"[^Workspaces].
Workspaces are a convenient way to organize programs composed of independent modules (called *crates* in the Rust parlance):

* Each **crate** is its own independent "project" - like that an IDE might create.

* In a **workspace**, two or more crates can share a single build directory. This saves compilation time for shared dependencies.

* Crates can call the public APIs of their workspace peers (other crates in the same workspace, but in a different subdirectory).

Our code in this chapter will be pretty short (less than 200 lines).
But for larger projects, workspaces aid modularity.
Modular code organization keeps complexity in check (more on this in Chapter 3).

First, we'll create a top-level directory to house both our crypto library and its command line interface.
Let's call it `crypto_tool`:

```ignore
mkdir crypto_tool
```

Next, we'll use `cargo` to generate skeletons for two crates:

1. A library (shared object) crate named `rc4`.

2. A binary (executable) crate named `rcli` (a questionable shortening of "RC4 CLI").

The `rcli` binary will depend on the `rc4` library's APIs.
Just like a real-world tool using a separate, pre-existing cryptographic library.

To generate the boilerplate for both crates:

```ignore
cargo new crypto_tool/rc4 --lib
cargo new crypto_tool/rcli
```

Notice the `--lib` flag tells `cargo` to create a library crate specifically.
Executable binaries with a `main` method are the default, if no flag is provided (but you can also use `--bin` if you want to be explicit).

> **What's the difference between a binary and a library?**
>
> *Binaries* are stand-alone programs you can run directly.
> The `tree` command below tells your shell to locate and execute the corresponding binary program.
>
> *Libraries* contain reusable code, typically APIs that can be called by binaries or other libraries.
> When `tree` prints output to your console, it calls `printf` - an API in C's standard library.
>
> Here's a fun fact: for file formats like Linux's ELF and Window's PE, the difference between a library and a binary is only 1 byte in the file header (metadata the loader understands).
> Both are just programs, as far as your CPU is concerned!

At present, `cargo` doesn't know that our two crates (`rc4` and `rcli`) are related.
Right now they just happen to exist in adjacent directories.
Let's keep `cargo` in the loop by creating a new `Cargo.toml` file in the `crypto_tool` directory:

```ignore
touch Cargo.toml
```

Open this newly-created file, in your editor of choice, and enter the following to inform `cargo` that `rc4` and `rcli` are part of the same workspace:

```ignore
[workspace]
members = [
    "rc4",
    "rcli"
]
```

If you run the Linux command `tree`, you should see the following file and directory layout:

```ignore
.
└── crypto_tool
    ├── Cargo.toml
    ├── rc4
    │   ├── Cargo.toml
    │   └── src
    │       └── lib.rs
    └── rcli
        ├── Cargo.toml
        └── src
            └── main.rs

5 directories, 5 files
```

`.rs` is the extension for Rust source files.
The two `.rs` files (`main.rs` and `lib.rs`) are where we'll write our code.

`Cargo.toml` files are project manifests[^Manifest], configurations for Rust's build system.
Notice the other two were created automatically when you ran `cargo new`.
Take a second to review their contents.

`rcli` will depend on the `rc4` library, so `cargo` needs a way to locate the library code at compile time.
We'll want to add an entry under the `[dependencies]` tag of its `Cargo.toml` file.
Open `rcli/Cargo.toml` and append the last line as below:

```toml,ignore
[package]
name = "rcli"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
rc4 = { path = "../rc4" }
```

To verify that your workspace is ready to roll, run `cargo build` from the `crypto_tool` directory.
You should see output similar to the below, showing both `rc4` and `rcli` being successfully compiled:

```ignore
   Compiling rcli v0.1.0 (/home/tb/proj/high-assurance-rust/code_snippets/chp2/crypto_tool/rcli)
   Compiling rc4 v0.1.0 (/home/tb/proj/high-assurance-rust/code_snippets/chp2/crypto_tool/rc4)
    Finished dev [unoptimized + debuginfo] target(s) in 0.43s
```

Now that the boilerplate is out of the way, we're ready to start writing our embedded-friendly RC4 library!

> **Do I have to understand all the details in the next section?**
>
> Nope.
> The next section is going to expose you to both Rust syntax and cryptography concepts.
> You don't need to fully understand the minutiae to proceed.
>
> * Rust's unfamiliar syntax will sink in as we progress, especially after Chapter 3.
>
> * Cryptography is not the focus of this book, you only need to grasp the broad strokes as context for the example program we're developing in this chapter.
>   * Remember to review [the corresponding appendix section](../chp16_appendix/crypto.md) if needed.

[^FalsePos]: Generally speaking, there are no false positives in dynamic analysis. But there exist test-specific exceptions. For example, say you're fuzzing (stress testing) a single function to find crashing inputs. You may find a crash, but in reality the full program may sanitize (normalize or reject) your crashing input before passing it along to the function under test. In this case, the crash may not actually be reproducible in the context of the larger program.

[^Workspaces]: [*Workspaces*](https://doc.rust-lang.org/cargo/reference/workspaces.html). The Cargo Book (Accessed 2022).

[^Manifest]: [*The Manifest Format*](https://doc.rust-lang.org/cargo/reference/manifest.html). The Cargo Book (Accessed 2022).
