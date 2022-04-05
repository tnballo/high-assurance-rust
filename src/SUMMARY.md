# Summary

[High Assurance Rust](landing.md)
[Frequently Asked Questions (FAQ)](faq.md)
[Engage with this Book!](engage.md)
[Sponsor Call for Proposals (CFP)](cfp.md)
[Changelog](changelog.md)
[License](license.md)

# Novice: Systems Security
---

* [Introduction](./chp1/_index.md)
    * [Why this book?](./chp1/why_this_book.md)
    * [How is this book structured?](./chp1/how_is_this_book_structured.md)
    * [Hands-on Learning](./chp1/challenges.md)
    * [About the Team](./chp1/about_the_team.md)
    * [Warmup: Environment Setup](./chp1/_hands_on.md)

* [Software Assurance](./chp2/_index.md)
    * [Static vs. Dynamic Tools](./chp2/static_vs_dynamic.md)
    * [Static Assurance (1/2)](./chp2/static_assurance_1.md)
    * [Static Assurance (2/2)](./chp2/static_assurance_2.md)
    * [Dynamic Assurance (1/3)](./chp2/dynamic_assurance_1.md)
    * [Dynamic Assurance (2/3)](./chp2/dynamic_assurance_2.md)
    * [Dynamic Assurance (3/3)](./chp2/dynamic_assurance_3.md)
    * [Limitations and Threat Modeling](./chp2/limits.md)
    * [DIY CLI Encryption Tool](./chp2/cli.md)
    * [Operational Assurance (1/2)](./chp2/operational_assurance_1.md)
    * [Operational Assurance (2/2)](./chp2/operational_assurance_2.md)
    * [Challenge: Extend the CLI Tool](./chp2/_hands_on.md)

* [Rust Zero-Crash Course](./chp3/_index.md)
    * [On Undefined Behavior](./chp3/undef.md)
    * [Rust: Low-Level Data (1/6)](./chp3/rust_1_low_data_rep.md)
    * [Rust: High-Level Data (2/6)](./chp3/rust_2_high_data_rep.md)
    * [Rust: Control Flow (3/6)](./chp3/rust_3_ctrl_flow.md)
    * [Rust: Ownership Principles (4/6)](./chp3/rust_4_own_1.md)
    * [Rust: Ownership in Practice (5/6)](./chp3/rust_5_own_2.md)
    * [Rust: Error Handling (6/6)](./chp3/rust_6_error.md)
    * [The Module System](./chp3/modules.md)
    * [Recommended Tooling](./chp3/tooling.md)
    * [Rust's Release Cycle]()
    * [Challenge: Port a Program](./chp3/_hands_on.md)

* [Understanding Memory](./chp4/_index.md)
    * [A Software Perspective]()
    * [An Attacker's Perspective]()
    * [Rust's Memory Safety Guarantees](./chp4/safe_rust_PLACEHOLDER.md)
    * [Integer Representation Issues]()
    * [The `#![no_std]` Attribute]()
    * [Case Study: Real-world Rust CVEs]()
    * [Debugging with Mozilla `rr`]()
    * [Writing an Exploit]()
    * [Challenge: TODO]()

# Advanced Beginner: Core Project
---

* [Binary Search Tree (BST) Basics]()
    * [Core BST Operations in Python]()
    * [Problems Translating to Rust]()
    * [The Importance of Balance]()
    * [TODO]()
    * [Challenge: TODO]()

* [Building an Arena Allocator]()
    * [Let's Talk Allocators]()
    * [A Stack-Only Arena]()
    * [Index-based Data Structures]()
    * [TODO]()
    * [Challenge: TODO]()

* [A Self-balancing BST]()
    * [Interface-relevant Traits](./chp7/traits.md)
    * [Scapegoat Trees]()
    * [Insert]()
    * [Remove]()
    * [Find]()
    * [Challenge: TODO]()

* [Digital Twin Testing]()
    * [Basic QEMU Internals]()
    * [How Semi-hosting Works]()
    * [CLI REPL Harness]()
    * [Limitations]()
    * [TODO]()
    * [Challenge: TODO]()

* [Building Maps and Sets]()
    * [TODO]()
    * [Challenge: TODO]()

* [Implementing Iterators]()
    * [TODO]()
    * [Challenge: TODO]()

# Competent: Validation and Deployment
---

* [Static Verification]()
    * [An Introduction to 1st Order Logic]()
    * [Proving Absence of Panics]()
    * [Deductively Verifying our Arena Allocator]()
    * [Model Checking for `unsafe` Code]()
    * [TODO]()
    * [Challenge: Prove a Sorting Algorithm]()

* [Dynamic Testing]()
    * [Introduction to Coverage-Guided Fuzzing]()
    * [Building a Differential Fuzzing Harness](./chp12/diff_fuzz_PLACEHOLDER.md)
    * [Using Miri to Detect Undefined Behavior]()
    * [Benchmarking and Optimization]()
    * [TODO]()
    * [Challenge: Bug-hunting with Fuzzers]()

* [Operational Deployment]()
    * [Understanding `unsafe` (1/3)]()
    * [Understanding `unsafe` (2/3)]()
    * [Understanding `unsafe` (3/3)]()
    * [CFFI 101]()
    * [C99 Interoperability]()
    * [Python3 Interoperability]()
    * [Runtime Balance Reconfiguration]()
    * [TODO]()
    * [Challenge: TODO]()

* [Maximizing Assurance]()
    * [Rust Security Research]()
    * [Rust's Limitations]()
    * [Best Practices Beyond Rust]()
    * [TODO]()
    * [Challenge: TODO]()

# Conclusion
---

* [Review]()
    * [Key Concepts]()
    * [Key Blue-Team Skills]()
    * [Key Red-Team Skills]()

* [Appendix](./chp16_appendix/_index.md)
    * [Setup: Using our Docker Container]()
    * [Inventory: Tools of the Trade](./chp16_appendix/tools.md)
    * [Inventory: Recommended Reading](./chp16_appendix/books.md)
    * [Inventory: Additional Resources](./chp16_appendix/resources.md)
    * [Fundamentals: Stream Ciphers](./chp16_appendix/crypto.md)
    * [Fundamentals: Type Systems](./chp16_appendix/types.md)
    * [Fundamentals: Component-Based Design](./chp16_appendix/components.md)
    * [Fundamentals: Memory Hierarchy](./chp16_appendix/mem_hierarch.md)
    * [Fundamentals: Dynamic Linking]()
    * [Misc: Size Optimization]()
    * [Misc: The Typestate Pattern]()
    * [Misc: C++ Interoperability]()
    * [Misc: Compile-time Metaprogramming]()