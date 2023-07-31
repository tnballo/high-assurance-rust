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

# Understanding Memory Safety and Exploitation
---

> **Note:** This section is a work-in-progress.

The "Dunning-Kruger effect" [^DKEffect] is an ironic phenomenon: people tend to overestimate their own understanding and abilities, particularly in areas where they have little knowledge or experience.

Yet even veteran programmers can fall victim to some variation of this effect.
Our egos often convince us that we know precisely what will happen when our code executes.
We wrote it, after all.

In reality a modern program is an incredibly complex apparatus built atop an even more complex hierarchy of hardware and software abstractions.
All operating in miraculous unison.
Few among us actually understand program execution from the physics of logic gates to the corner cases of network protocols.
Most of the time we can't even get the layer we're working at right.
Hence version numbers.

The good news is we don't have to know it all.
In Chapter 1's Dreyfus Model, the "Competent" stage was marked by the rude realization that the learner's knowledge is remarkably limited.
In response, the learner needs to de-prioritize the less relevant details and focus on those pertinent to their end goals.
To separate the wheat from the chaff.

Systems programming requires a mental model of "what the computer is doing", but it doesn't have to be exhaustive.
In truth, programming languages which give developers "full control"[^Ctrl] over the hardware - like C and Rust - deal primary with the concepts and mechanisms of one thing: memory.

* If you understand the bulk of how and why memory works, you're well on your way to mastery of low-level programming.

* If you understand how attackers craft memory corruption exploits, you're more likely to catch real bugs and exploitable vulnerabilities in cross-language code and/or `unsafe` Rust before it reaches production.

## Isn't systems programming more than just managing memory?

Certainly.
Remember the three hypothetical engineers introduced in Chapter 1, when discussing what defines a "system program"?

Each engineer held a different view, because each came from a specialization requiring unique expertise.
For example:

* Distributed systems developers understand **consensus protocols** and **fault tolerance**.

* Device driver developers work with **kernel APIs** and **interrupt handling**.

* Microcontroller firmware developers interface with  **analog components** and read **device datasheets**.

But these facets of systems programming are largely domain-specific.
Effective use of memory is a sort of universal bottleneck, it's necessary but not sufficient for writing performant applications.
Regardless of domain.

This chapter will cover universal computer architecture principles relevant to controlling memory.
The meat of what every systems programmer ought to know.
We'll put these principles into practice in Chapter 6, implementing a stack storage abstraction that maximizes both safety and portability.

## ~~Memory~~ Knowledge Dump

Memory is perhaps the most single important topic in this book.
This is our final conceptual chapter, the rest of our adventure will focus on writing a Rust library.
Grab a coffee now (Yerba Mate if you must) because we're gonna really get into the mechanical details here.

We'll start by looking at memory from a software perspective, the model of most systems programmers work with day-to-day.
Then we'll dig into the attacker's perspective, learning about how memory corruption bugs are turned into devastating exploits.
We'll learn about dynamic debugging and perform introductory, hands-on heap exploitation.
Once you can subvert rules and assumptions, you truly understand how something works.
At least when it comes to security.

Armed with a deeper understanding of memory, we'll examine how Rust provides memory safety guarantees.
In detail.

<!--
TODO: add appendix section on integer representation instead

Then we'll tackle two more narrow but nonetheless important topics: integer representation issues and Rust's `!#[no_std]` attribute.
--->

We'll conclude our memory world tour by exploring language-agnostic mitigations and looking at real-world Rust CVEs.

> **What about the hardware perspective?**
>
> The [*Fundamentals: Memory Hierarchy*](../chp16_appendix/mem_hierarch.md) section of the Appendix takes a hardware-centric view, looking at performance tradeoffs within the modern memory hierarchy.
> Highly recommend it as a supplement to this section.

## Learning Outcomes

* Develop a mental model of system memory and program execution
* Develop a mental model of memory safety, type safety, and binary exploitation
* Learn to debug Rust code using Mozilla `rr`[^RR] (an enhanced variant of `gdb`[^GDB])
* Understand how attackers exploit heap memory corruption bugs, step-by-step
* Write your first an introductory exploit or two, bypassing modern protections!
* Understand how Rust actually provides memory safety, including current limitations
* Understand how modern, language-agnostic exploit mitigations work (and how they can fail)

---

[^DKEffect]: ["Unskilled and Unaware of It: How Difficulties in Recognizing One's Own Incompetence Lead to Inflated Self-Assessments"](https://pubmed.ncbi.nlm.nih.gov/10626367/). Justin Kruger, David Dunning (1999)

[^Ctrl]: In both programming and modern life, you never quite have full control. In programming that's because both compilers and interpreters make oft-inscrutable decisions for you (e.g. aggressive optimization[^Opt]) and, rarely, even contain bugs[^CompBug].

[^RR]: [*`rr`*](https://rr-project.org/). Mozilla (Accessed 2022).

[^GDB]: [*GDB: The GNU Project Debugger*](https://www.sourceware.org/gdb/). GNU project (Accessed 2022).

[^Opt]: [*C Is Not a Low-level Language: Your computer is not a fast PDP-11.*](https://queue.acm.org/detail.cfm?id=3212479). David Chisnall (2018).

[^CompBug]: One particular funny case is CVE-2020-24658[^StackVuln], a failed compiler-inserted stack protection. As an aside, vulnerabilities *patched* by new compiler versions are an interesting category. Which can include *hardware* vulnerabilities (e.g. CVE-2021-35465[^VLLDMVuln]).

[^StackVuln]: [*CVE-2020-24658 Detail*](https://nvd.nist.gov/vuln/detail/CVE-2020-24658). National Institute of Standards and Technology (2020).

[^VLLDMVuln]: [*VLLDM instruction Security Vulnerability*](https://developer.arm.com/support/arm-security-updates/vlldm-instruction-security-vulnerability). ARM (2021).