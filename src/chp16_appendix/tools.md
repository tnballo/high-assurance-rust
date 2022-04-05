# Tools of the Trade

This is a complete inventory of all the software assurance tools and Rust libraries you'll use in this book.
You'll get deep experience in a few, but only a taste of most.
Each name below is a link to the tool's homepage or documentation.

### Core Tooling

**Static Assurance**

Most tools in this category reason about source-level semantics to prove the absence of certain bugs.
They trust the compiler and, by extension, its backend.

* [`rustc`](https://rustc-dev-guide.rust-lang.org/) (Rust-only, it's literally the compiler!)
* [`cbmc` via `kani`](https://github.com/model-checking/kani) (Rust front-end, underlying tool also supports C/C++)
* [`viper` via `prusti`*](https://www.pm.inf.ethz.ch/research/prusti.html) (Underlying tool supports Rust, Python, Java, ...).
* [`creusot`*](https://github.com/xldenis/creusot) (Rust-only)

**Dynamic Assurance**

Most tools in this category test compiled executables to discover specific bugs or observe program behavior.
They remove the compiler from the chain of trust.

* [`valgrind`*](https://valgrind.org/) (x86, x86_64, ARM32, ARM64, MIPS, PPC)
* [`rr`](https://rr-project.org/) (x86, x86_64, ARM64)
* [`libfuzzer` via `cargo-fuzz`](https://llvm.org/docs/LibFuzzer.html) (x86, x86_64)
* [`qemu`](https://www.qemu.org/) (x86, x86_64, ARM32, ARM64, MIPS, PPC, AVR, ...)
* [`miri`](https://github.com/rust-lang/miri) (Rust-only)

**Operational Assurance**

Tools that support a system's lifecycle.

* [`docker`](https://docs.docker.com/engine/reference/commandline/cli/) (Linux guests)
* [`cbindgen`](https://crates.io/crates/cbindgen) (CFFI)

### Rust Ecosystem

Open-source binaries and libraries hosted on [crates.io](https://crates.io/).

**Development**

* [`clap`](https://crates.io/crates/clap) - Command line argument parsing.
* [`serde`*](https://crates.io/crates/serde) - Rust structure serialization and deserialization.
* [`tinyvec`](https://crates.io/crates/smallvec) - `!#[no_std]`, `#![forbid(unsafe_code)]` `Vec` alternative.
* [`micromath`](https://crates.io/crates/micromath) - `!#[no_std]`, `#![forbid(unsafe_code)]` floating point approximations.
* [`lazy_static`*](https://crates.io/crates/lazy_static) - runtime-initialized static variables.
* [`owo-colors`](https://crates.io/crates/owo-colors) - embedded-friendly text coloring.

**Testing**

* [`criterion`](https://crates.io/crates/criterion) - micro-benchmarking toolset.
* [`cargo-modules`](https://crates.io/crates/cargo-modules) - text render of project's module architecture.
* [`cargo-audit`](https://crates.io/crates/cargo-audit) - search project's decency graph for known-vulnerable versions.
* [`cargo-binutils`](https://crates.io/crates/cargo-binutils) - inspect the properties and contents of Linux binaries.
* [`cargo-bloat`](https://crates.io/crates/cargo-bloat) - determine what parts of an executable contribute to it's size.
* [`siderophile`*](https://crates.io/crates/siderophile) - search project's call graph for pockets of `unsafe` code.
* [`cargo-tarpaulin`*](https://crates.io/crates/cargo-tarpaulin) - code coverage reporting (MC/DC support planned).

**Other**

* [`xgadget`*](https://crates.io/crates/xgadget) - ROP/JOP exploit development.

---

> \* == may be subject to change. This book is a [work in progress](./faq.md#8-is-this-book-free).
>
> Additional tools are likely to be added as the book matures. Though unlikely, tools may also be removed.