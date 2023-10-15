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


# Fundamentals: Memory Hierarchy

Most programmers aren't aware of the gory details of interacting with the hardware beneath them.
That's by design.
It's the result of a multi-decade software-hardware co-evolution.

Consider the humble `print` statement.
Last time you printed formatted output to a console, you likely didn't have think about:

* Flushing buffers for a character device (OS software "glue" abstraction to vendor-specific hardware).

* The intricacies of how parity bits work in the Universal Asynchronous Receiver/Transmitter (UART[^UART]) protocol (encoding for physical transmission medium).

These are minutiae most day-to-day development shouldn't have to factor in.
Memory hardware technology is, likewise, something you'll be largely abstracted from.
Most of the time, you can think of a computer, any computer, as a composite of two distinct parts:

1. A **Central Processing Unit (CPU)** that fetches, decodes, and executes instructions from compiled binaries.

2. A singular **memory system** holding instructions the CPU reads and data it operates on.

In this simple two part conceptualization, memory is just a linear array of bytes that can be addressed individually and accessed in constant time.
But that's not reality.

What we think of as "memory", from a software perspective, is actually a heterogeneous mix of hardware technologies that offer varying speeds and capacities at different price points.
And that can have repercussions for systems software.

Decisions you make as a systems programmer can determine what particular kind of storage hardware your program makes use of during its execution.
And this hardware's characteristics can have a *significant* impact on the speed of your program.
So we need to ground our discussion of memory in its physical reality: the performance hierarchy of contemporary storage technologies.

> **Credit where credit is due**
>
> The contents of this chapter are heavily influenced by [*Computer Systems: A Programmer's Perspective*](https://amzn.to/3IBnFA7)[^CSAPP].
>
> The book's authors, professors at Carnegie Mellon University, use it to teach an undergraduate CS course. The course number is 15-213, affectionately called "the course that gives CMU its zip" because 15213 is the ZIP code in which the university is located and because the course is so foundational in the university's core CS curriculum.
>
> CS:APP is a well-regarded text and a detailed introduction to both computer architecture and operating systems.
> Highly recommend it.

## The Memory Performance Hierarchy

We can arrange modern memory technologies into distinct, hierarchical tiers.

* The higher tiers are **incredibly fast** but also **incredibly expensive per byte**. They are a scarce resource. As a consequence, the amount of data they can store is limited.

* The lower tiers are **relatively slow** but also **relatively cheap per byte**. They're a plentiful resource, we're free to store large amounts of data in them.

Consider this breakdown:

| Storage Technology |  Unit of Storage | Access Time (Nanoseconds) [^CornellCompArch] | Explicit APIs? |
| --- | --- | --- | --- |
| CPU Registers | 4-8 bytes per register | 1 ns | No |
| SRAM Cache | Kilobytes, Megabytes | 1 to 4 ns (L1 to L2[^L1]) | No |
| DRAM | Gigabytes | 100 ns | Yes, stack and heap |
| Local Disk | Gigabytes, Terabytes | 16,000 to 4,000,000 ns (SSD read to HDD seek[^SSD]) | Yes, unless DRAM exhausted (swap) |
| Remote Storage | *N/A, Unlimited* | 15,0000,000 ns (Packet RTT[^RTT] California to/from Netherlands) | Yes, networking |

* **CPU Registers:** Accessed directly when instructions execute (e.g. in 0 CPU cycles[^CSAPP]), registers sit at the top of the hierarchy.

    * Register allocation is handled by the compiler, in accordance with a target machine's Application Binary Interface (ABI)[^ABI]. Unless you're writing inline assembly[^InAsm] (intermixing architecture-specific instructions with Rust source code), you won't control register usage directly.

* **SRAM Cache:** A small memory bank built into the CPU, physically close to the register action. Accessible in 4 to 75 CPU cycles[^CSAPP].

    * How often data your code needs will be present in the cache (aka "cache hit ratio") is a side effect of how your code is written, but not something you can explicitly control with API calls. Data-oriented Programming[^DataRust] deals with structuring programs to make optimal use of CPU caches.

* **DRAM:** Main memory, the sweet spot for most systems programming. Accessible in hundreds of cycles[^CSAPP].

    * Often, we can explicitly control which regions of main memory we use. Specifically whether data is stored on the stack, the heap, static memory, or even "memory-mapped" files - among other locations. Each option has performance implications. You already wrote a stack-only program: the RC4 implementation in the Chapter 2 specified the `#![no_std]` attribute.

* **Local Disk:** Long-term storage that persists across runs of a program or reboots of a machine. Accessible in tens of millions of cycles[^CSAPP], a significant penalty relative to the levels above.

    * Interacting with local disk storage typically means calling explicit APIs to open, read, or write files. Unless all available DRAM is currently in use - in which case the OS is forced to *page*[^Paging] your program data to/from local secondary storage behind-the-scenes.

* **Remote Disk:** Long-term storage on a physically separate machine, connected to the host running your program via a network. Access latency can't even be measured in CPU cycles, there are too many unpredictable factors at play (protocols, physical distance, network congestion, etc). The above table uses nanosecond estimates[^CornellCompArch] for your convenience.

    * There's no way to implicitly download/upload data from/to a remote machine, you must call a networking API directly or indirectly.

> **Memory Management in Modern Operating Systems**
>
> Paging schemes[^Paging] are a part of how *virtual memory* (an abstraction for DRAM managed by the OS) is implemented.
> This is a complex topic with many implications, I'd recommend Chapter 9 of CS:APP[^CSAPP] for a thorough exploration.
> To summarize, we can think of virtual memory as providing three benefits:
>
> * **Simplified view of memory:** Each process is given an uniform linear virtual address space to run in, regardless of where it's actually mapped in physical memory and whether or not some of those mappings are shared with another process.
>
> * **Address space isolation:** The OS can enforce isolation between the address spaces of each process, preventing one process from corrupting another. Likewise userspace applications (e.g. your program) can't access kernel space (e.g. OS internals).
>
> * **Efficiency through caching:** Allows main memory (system DRAM) to act as a cache for files on disk, making active items more quickly accessible and managing the back-and-forth transfer between DRAM and disk (paging). The smallest unit of data an OS can move is 1 page (typically 4 kB).

## Ok, but what are the practical implications of all this?

Distinct types of memory hardware have a significant impact on performance.
Potentially orders of magnitude.
So, assuming we've picked the *fastest known algorithm*, we should keep two facts in mind:

* **Disk and/or network I/O is expensive but explicit.** The bottom two rungs of the memory performance hierarchy are far slower, but at least we can consciously control their usage.

    * In addition to being slow, file and network I/O are *fallible*. A file might have moved or changed permissions. A remote server may become temporarily or permanently inaccessible. Logic for handling these cases, whether it be propagating an error or retrying an alternate path/host, further exacerbates performance cost.

* **Cache optimization can be a differentiator.** Rust's `BTreeSet`[^BTreeSet] and `BTreeMap`[^BTreeMap], the standard library collections we'll build alternatives to, are specifically designed to maximize SRAM cache efficiency. Both are very performant.

    * As an aside, the standard libraries of C++ and Java both use Red-Black Trees. B-trees are more common in filesystem and database use cases.

    * Our library's optimizations target another level of the hierarchy: DRAM. Using an index-based allocator pattern (introduced in Chapter 6), we'll ensure our code only uses *stack* DRAM. The result is embedded portability.

> **Why the "fastest known algorithm" caveat?**
>
> Algorithms typically have a much greater performance impact than the kind of physical memory backing execution.
>
> If you implement a quadratic time solution when a problem can be solved in linear time, no amount of SRAM locality can compensate.
> The latter solution scales far, far better regardless.
>
> We discuss the basics of algorithmic complexity in Chapter 7.

## Takeaway

Much as we'd like to believe memory is just a linear array of bytes, the reality is it's a hierarchy of hardware making cost/performance tradeoffs.
This physical view of memory is foundational.

But day-to-day systems programming is more concerned with a logical view of memory, namely managing stack frames and heap buffers.
Stored in DRAM.
That's the abstraction through which we view all code in this book.

---

[^UART]: [*Basics of UART Communication*](https://www.circuitbasics.com/basics-uart-communication/). Scott Campbell (Accessed 2022).

[^CSAPP]: [***[PERSONAL FAVORITE]** Computer Systems: A Programmer's Perspective*](https://amzn.to/3IBnFA7). Randal Bryant, David O'Hallaron (2015).

[^CornellCompArch]: [*ORIE 6125: Week 8 - Computer architecture*](https://people.orie.cornell.edu/bdg79/ORIE6125/lecture8.html). Cornell University (Accessed 2022).

[^ABI]: [Application binary interface](https://en.wikipedia.org/wiki/Application_binary_interface). Wikipedia (Accessed 2022).

[^L1]: [Multi-level caches](https://en.wikipedia.org/wiki/CPU_cache#MULTILEVEL). Wikipedia (Accessed 2022).

[^SSD]: Solid State Drives (SSDs) and Hard Disk Drives (HDDs) are two secondary storage technologies. The former offers faster read and write speeds.

[^RTT]: Round Trip Time (RTT), in this context, is the amount of time it takes for a packet to reach a destination and for an acknowledgment to come back.

[^InAsm]: [*New inline assembly syntax available in nightly*](https://blog.rust-lang.org/inside-rust/2020/06/08/new-inline-asm.html). Josh Triplett (2020).

[^DataRust]: [*An introduction to Data Oriented Design with Rust*](http://jamesmcm.github.io/blog/2020/07/25/intro-dod/). James McMurray (2020).

[^Paging]: [Paging](https://wiki.osdev.org/Paging). OSDev Wiki (Accessed 2022).

[^BTreeSet]: [*Struct `std::collections::BTreeSet`*](https://doc.rust-lang.org/stable/std/collections/struct.BTreeSet.html). The Rust Team (Accessed 2022).

[^BTreeMap]: [*Struct `std::collections::BTreeMap`*](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html). The Rust Team (Accessed 2022).
