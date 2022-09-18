# On Undefined Behavior

One of MISRA C's driving goals is reducing the amount of "Undefined Behavior" (UB) [^MISRA_TALK] present in a codebase.
We made passing mention of UB in Chapter 1, but it's an essential concept.
Eliminating UB is necessary, but not sufficient, for high assurance software.
So let's tackle the topic now, before we get into Rust syntax.

The ISO C standard[^ISOC] defines **behavior** as:

> External appearance or action.

Consequently, **Undefined Behavior (UB)** is defined as[^ISOC]:

> Behavior, upon use of a nonportable or erroneous program construct or of erroneous data, for which this International Standard imposes no requirements.

In other words, should a developer inadvertently trigger UB, *the program can do absolutely anything*.
It may crash, produce an incorrect result, or even execute a sequence of seemingly unrelated operations[^NasalDemons].

Notice we didn't say "the undefined operation can do absolutely anything", we said "the program".
It's important to understand one fact about UB:

* Once undefined behavior is triggered, the adverse impact often cannot be *localized*. It may compromise the security and/or reliability of the *entire* system.

UB is an undesirable source of bugs and vulnerabilities.
Yet UB is ingrained into both C and C++ standards for a variety of complex historical reasons.
Removing even a small portion of UB from either language would break a large percentage of available compilers or their ability to compile existing code.
So it's unlikely to happen.

The best we can do is make a diligent effort to avoid it.
That means heavy auditing and thorough testing of C-family codebases and, where possible, introducing Rust.

## How easy is it to introduce UB in C?

C and C++ programs are, relative to memory-safe languages, difficult to debug and easy to exploit because introducing UB is both *trivial* and *common*.

Researchers have found instances of UB in widely used projects, like the Linux kernel and PostgreSQL[^UndefResearch].
Memory corruption bugs, whose severity and prevalence we discussed in Chapter 1, are just one consequence of UB.

Let's get a tangible feel for what UB looks like.
Consider the below C program - what does it return?

```c,ignore
#include <stdio.h>

int undef_func() {
    int uninit_var; // Never assigned to!
    if (uninit_var > 0) {
        return 1;
    } else {
        return 0;
    }
}

int main() {
    printf("%d\n", undef_func());
}
```

That was a trick question.
The answer is `1` or `0`, depending whatever just happened to be in memory at the time.
Simply because the `if` statement read an uninitialized value and then branched on the result.
The C Standard (6.7.8, paragraph 10[^ISOC]) states:

> If an object that has automatic storage duration is not initialized explicitly, its value is indeterminate.

In the context of the C standard, "indeterminate" means a variable can either:

* Take on any legal value for the type (e.g. "unspecified value").

* Take on a value that doesn't represent any instance of the type (e.g."trap representation"). UB ensues.

Neither case bodes well for program reliability.
This simple function violates a MISRA C rule that explicitly targets this part of the standard:

> **[AR, Rule 9.1]** The value of an object shouldn't be read if it hasn't been written[^MISRA_2012]

By default `gcc` - a popular, open-source C compiler - will not warn about this serious error.
We have to remember to pass the `-Wall` flag to get the below warning, and even then the program will build and run:

```ignore
undef.c: In function ‘undef_func’:
undef.c:5:8: warning: ‘uninit_var’ is used uninitialized in this function [-Wuninitialized]
    5 |     if (uninit_var > 0) {
      |        ^
```

Unfortunately, remembering special compiler flags is not a general solution.
C has *hundreds* of possible undefined behaviors and the *vast majority* cannot be caught by compiler warnings.
Thus, these "misbehaviors" rapidly creep into codebases as they grow in complexity.

> **What's a real example of this problem? In a security context?**
>
> CVE-2022-0847, aka "Dirty Pipe", was a highly exploitable vulnerability affecting Linux kernel versions after 5.8 (patched in stable releases 5.16.11, 5.15.25 and 5.10.102.[^DirtyPipe]).
> A code refactor caused a structure's field to be uninitialized, and the instance of UB wasn't caught by compiler warnings or testing.
>
> The uninitialized field was the `flags` member of a kernel-space `pipe_buffer` data structure.
> This is used by the kernel to set up "pipes", an Inter-Process Communication (IPC) mechanism.
>
> By performing a sequence of normal, unprivileged operations, an attacker could reliably control the in-memory value that would later be read (instead of being reset/initialized correctly) as a flag for page cache write permissions[^DirtyPipe].
>
> By abusing this ill-gotten permission to pipe into files, an attacker can overwrite small chunks of content in system files that should be read-only.
> This can enable, among other things, changing the root password to escalate local privileges and then overwriting SSH key data used for remote access.
>
> Effectively, an attacker can gain "full control" of a vulnerable system just by getting a user to execute an unprivileged program. All because of one uninitialized field!
> Give the attacker an inch, they might take a mile.

## Let's try that in Rust

Undefined behavior is still possible in Rust, if using `unsafe` keyword[^UndefRust], but it's *almost eliminated* in the safe subset of Rust.
That's a major part of why the Rust language is so amenable to writing correct, reliable software.
Rust removes UB, nearly entirely, by default.

> **Why the "almost eliminated" and "nearly entirely" caveats?**
>
> At the time of this writing, Rust does not yet have an official language standard or specification.
> There's no Rust equivalent to C or C++'s ISO documents.
> So it's difficult to make a definitive claim.
>
> The Rust Reference contains a non-exhaustive list of behaviors considered undefined in Rust[^UndefRust], all of which would require the `unsafe` keyword to introduce.
> So there are likely only two potential sources of UB in Rust:
>
> * `unsafe` functions or blocks whose invariants aren't actually upheld (our fault).
>
> * Rare compiler bugs[^RustcBug] that threaten soundness (patched once discovered).
>
> We'll use `miri`[^Miri], an experimental dynamic tool for detecting UB in Rust programs, in Chapter 12.
>
> An unofficial Rust language specification effort is currently underway[^FLS] to support Ferrocene, a vendored Rust toolchain to be qualified for safety-critical use.
> While this specification does not aim to document the entire Rust language and standard library, it will enumerate UB where relevant and be publicly available[^FLS].

To make Rust's benefits more visceral, let's port our buggy C program to Rust:

```rust,ignore
fn undef_func() -> isize {
    let uninit_var: isize;
    if uninit_var > 0 {
        return 1;
    } else {
        return 0;
    }
}

fn main() {
    println!("{}\n", undef_func());
}
```

Running `cargo build` results in the following:

```ignore
error[E0381]: use of possibly-uninitialized variable: `uninit_var`
 --> src/main.rs:3:8
  |
3 |     if uninit_var > 0 {
  |        ^^^^^^^^^^ use of possibly-uninitialized `uninit_var`

For more information about this error, try `rustc --explain E0381`.
```

The `gcc` warning was similar, but it heeding it was entirely optional.
In Rust, this same mistake is a hard error - the program will not compile unless we address the issue.
In other words, all safe Rust programs obey the aforementioned MISRA C Rule 9.1.

More generally, successfully compiling a safe Rust project means UB is likely eliminated.
So obeying the below buys us a great deal of assurance:

> **[RR, Directive 2.1]** The entire project should compile without error[^MISRA_2012]

## Why does UB even exist?

Let's assume it's desirable to have multiple compilers for a language, whether they be commercial or open-source.
Each compiler implementation may serve a different niche, offer unique features, or just experiment with promising ideas.
Just like we have multiple web browsers that all support the same standards and protocols (HTML, HTTP2, etc).

Thus, a singular language standard (like the ISO C standard[^ISOC] we've mentioned) needs to be applicable to any compiler implementation targeting any platform architecture.
This is akin to an interface design problem.
It entails designing failure modes, which is where UB comes in[^CppUndef] - it's one way to "handle" edge cases the standard won't, shouldn't, or can't impose a universal rule for.

So the standard draws a boundary: it defines an "abstract machine" general enough to represent a variety of underlying hardware.
This has the upside of giving compiler developers room to introduce platform-specific optimizations.
Which is one of the main jobs of a compiler: it repeatedly applies *rewrite rules* to generate efficient machine code.

An optimizing compiler assumes that input source code never introduces UB, per the language specification.
If this assumption is:

* **True** (source is indeed UB-free) - rewrite rules replace existing code with new code that is both faster and *logically equivalent*.

    * These rules often take advantage of the "wiggle room" an abstract specification provides to play with architecture-specific instruction and/or memory model semantics. Or remove checks that prove necessary[^RalfUB].

* **False** (source contains UB) - application of rewrite rules may lead to *logical contradiction*.

    * If the UB present is "triggered", results include incorrect code replacements and/or arbitrary runtime operations.

This dichotomy begs the question: couldn't a sufficiently "smart" compiler simply *verify* its assumption of UB-free source?
Just like it checks syntax and typing at compile time?

The answer is yes!
As alluded to, that's exactly what `rustc` does when compiling fully-safe Rust code[^IntOverflow].
Using a combination of its advanced type system and runtime check insertion.

But guaranteeing absence of all UB automatically is technically infeasible for C, C++, *and* `unsafe` Rust.
Futhermore, even the safest of Rust programs might link against *some* `unsafe` code internally, like C's `libc` or parts of Rust's `core`[^Core].
From an assurance perspective, we're betting that such widely-used and well-vetted dependencies are less likely to contain UB than `unsafe` code we'd write ourselves.

> **What's an example of an optimization?**
>
> John Regehr presented a compelling snippet in a 2017 talk[^CppUndef], we'll adapt it here.
> Say we have this function:
>
> ```c
> int set(int* a, int* b) {
>    *b = 3;
>    *a = 7;
>    return *b;
> }
> ```
>
> `a` and `b` are pointers to the same type, `int`, and they may alias (recall our discussion of pointers and aliasing from Chapter 2).
> That means the function should return `3` if the pointers don't alias or `7` if they do.
>
> Thus, the compiler is forced to generate machine code that *loads from memory* before returning an integer.
> It needs to read the freshest data to handle both the alias and no-alias cases.
> Something akin to the following snippet of x86-64 assembly may be emitted (ATT syntax):
>
> ```assembly
> set:
>   movl $3, (%rsi)
>   movl $7, (%rdi)
>   movl (%rsi), %eax
> ```
>
> If you're not familiar with x86 assembly, the key idea here is that the last line is a load from a memory address:
>
> * `%rsi` is a register holding a pointer.
>
> * `(%rsi)` is a dereference of the pointer, we read the data it points to.
>
> * `movl (%rsi), %eax` copies the data read into `%eax`, the register used for return values[^EAX].
>
> Now say the two parameters are pointers to different integer types:
>
> ```c
> int set(long* a, int* b) {
>    *b = 3;
>    *a = 7;
>    return *b;
> }
> ```
>
> In this case the compiler can, per the C standard's definition of "strict aliasing", assume that the pointers don't alias.
> We no longer need to read a current value from memory, we can return a constant `3`.
> That's faster.
> This optimization may result in assembly like:
>
> ```assembly
> set:
>   movl $3, (%rsi)
>   movq $7, (%rdi)
>   movl $3, %eax
> ```
>
> Great, we got more efficient code.
> There's no read in the last instruction, just a move of constant.
>
> So what's the problem?
> A C programmer can break that assumption by casting an `int*` to a `long*` before calling the function.
> Behavior of the below program is undefined:
>
> ```c
> include <stdio.h>
>
> int set(long* a, int* b) {
>     *b = 3;
>     *a = 7;
>     return *b;
> }
>
> int main() {
>     int x = 0;
>     printf("%d\n", set((long*)&x, &x));
> }
> ```
>
> For various reasons, casting is a common operation in C and C++ programs.
> Projects like the Linux kernel explicitly disable this specific optimization for safety[^UndefResearch].
>
> In safe Rust, we can't cast references. And we're able to guarantee absence of mutable aliasing at all times.
> So, in this particular case, Rust is capable of performing the optimization without the UB danger.

## What are the consequences of UB in practice?

There are four possible outcomes[^CppUndef].
We can enumerate them, roughly in order of best to worst case:

1. **Program breaks immediately:** Crash (e.g. segmentation fault) or exception (e.g. attempt to divide by zero) will be hit at runtime, the program will halt .

    * The easiest case to detect prior to shipping a product. We just need to execute the faulty code path once, in a dynamic test.

2. **Program continues with corrupted state:** Internal state becomes logically invalid, but the program continues to execute. It may crash at a later point in time, if some arbitrary condition is met, or simply finish but produce the wrong result.

    * This case is more challenging to detect, it can require more thorough test cases to uncover.

3. **Program works as expected, despite relying on UB:** The program appears correct from a testing perspective, but the UB is a "time bomb" waiting to trigger. The program may no longer work compiled for a different architecture, with a newer compiler, or simply using different settings.

    * Detection requires a change or update of the build toolchain. And, if the UB manifests as case 2 above, detection may not be immediate.

4. **Program is vulnerable to attack:** The program doesn't trigger UB given expected inputs, but will if an attacker provides a specially-crafted input. Exploiting memory corruption bugs entails triggering UB (we'll see next chapter).

    * This is the worst case scenario - an attacker detects UB our tests failed to catch, and then leverages it to compromise production assets.

The first three potential consequences of UB are a threat to functionality and reliability.
The fourth is a threat to security.
That's why the MISRA C standard includes this broad rule:

> **[AR, Rule 1.3]** Eliminate all occurrences of undefined behavior[^MISRA_2012]

Rust's design generally makes it easier to comply with the rule.
The developer isn't responsible for remembering hundreds of obscure UB edge cases simultaneously, and then enforcing them without fail across a million-line codebase.
Instead, the Rust compiler checks for potential issues.
Automatically and accurately.

## Takeaway

Our best tools can't pinpoint every Undefined Behavior in a moderately-sized C or C++ codebase.

* Commercial static analysis tools suffer from false positives: actionable results are often buried in noise. Moreover, a lot of UB is difficult to design a detection algorithm for. As you may recall from Chapter 2, important problems in static analysis, like aliasing, are mathematically undecidable.

* Dynamic tools (like LLVM's open-source `UBSan`[^UBSan], `ASan`[^ASan], and `TSan`[^TSan]) have improved greatly in recent years, but still miss bugs due to fundamental limitations of dynamic testing (tiny sample of program state-space). Even when combined with coverage-guided fuzzing (introduced in Chapter 12).

That's a part of why standards like MISRA C exist, and why countless engineering hours are devoted to ensuring these standards are followed.

Reducing defect rate is an uphill battle.
One could make the argument that, due to the vast amount of Undefined Behavior C and C++ allow for in their respective standards, it's a war of attrition.
Winners pay an incredible engineering cost - in tool licensing, processes that slow shipping, and debugging person-hours.
Or in service disruption, should the UB lead to an exploitable vulnerability.

So let's start learning that safe subset of Rust!
Rust isn't perfect, but eliminating UB is certainly its strong suit.

> **`gcc` hasn't surrendered the fight!**
>
> Given a weak type system and a high-UB written specification, we believe that C compilers have low assurance ceilings with respect to memory safety[^CompilerAssurance].
> But important advances are still being made.
> And, given C's widespread usage, every inch of progress is high-impact.
>
> `gcc` 12 offers improved, experimental, static taint analysis[^GCC] (flow tracking for untrusted data).
> In conjunction with source annotations, that's a way to systematically review potential attack entry points.
> And an advanced feature not currently offered by `rustc`.
>
> This same version adds a new `-Wanalyzer-use-of-uninitialized-value` flag[^GCC].
> Unlike the `-Wuninitialized` warning our above use of `-Wall` encapsulated, this new flag uses branch-sensitive static analysis of flows between functions.
> That may mean less false positives *and* more actionable warnings.
>
> We did not test `gcc` 12's ability to detect the aforementioned "Dirty Pipe"[^DirtyPipe] kernel vulnerability.
> But that could be a worthwhile exercise for interested readers.

---

[^MISRA_TALK]: [*The Misra C Coding Standard and its Role in the Development (SAS Talk)*](https://www.youtube.com/watch?v=LCZotsYizRI). Roberto Bagnara (2018).

[^ISOC]: [*ISO/IEC 9899:TC3*](http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1256.pdf). International Organization for Standardization (2007). Note newer standards for the C language must be paid for, they are not freely available online. The points we make in this book are still applicable to newer C standards.

[^NasalDemons]: [*nasal demons*](http://www.catb.org/jargon/html/N/nasal-demons.html). According to an infamous Usenet post, the arbitrary consequences of UB could include making "demons fly out of your nose". Hence UB is sometimes joking referred to as "nasal demons".

[^UndefResearch]: [*Undefined Behavior: What Happened to My Code?*](https://people.csail.mit.edu/nickolai/papers/wang-undef-2012-08-21.pdf). Xi Wang, Haogang Chen, Alvin Cheung, Zhihao Jia, Nickolai Zeldovich, M. Frans Kaashoek (2012).

[^MISRA_2012]: *MISRA C: 2012 Guidelines for the use of the C language in critical systems (3rd edition)*. MISRA (2019).

[^DirtyPipe]: [*The Dirty Pipe Vulnerability*](https://dirtypipe.cm4all.com/). Max Kellerman (2022).

[^UndefRust]: [*Behavior considered undefined*](https://doc.rust-lang.org/reference/behavior-considered-undefined.html). The Rust Reference (Accessed 2022).

[^RustcBug]: [Unsoundness in `Pin`](https://internals.rust-lang.org/t/unsoundness-in-pin/11311). comex (2019).

[^Miri]: [*`miri`*](https://github.com/rust-lang/miri). Ralf Jung (Accessed 2022).

[^FLS]: [*Introducing the Ferrocene Language Specification*](https://ferrous-systems.com/blog/ferrocene-language-specification/). Ferrous Systems (2022).

[^CppUndef]: [*CppCon 2017: "Undefined Behavior in 2017"*](https://www.youtube.com/watch?v=v1COuU2vU_w). John Regehr (2017).

[^RalfUB]: [*Undefined Behavior deserves a better reputation*](https://www.ralfj.de/blog/2021/11/18/ub-good-idea.html). Ralf Jung (2021).

[^IntOverflow]: This claim may have debatable edge cases. For example, if `overflow-checks = false` is specified in `Cargo.toml` (the default setting for the optimized `release` profile) then integer overflow can happen at runtime. That's not technically UB in Rust, like it is in C/C++, because you can reliably expect two's complement wrap. But it might still cause unanticipated bugs in the context of your larger application.

[^Core]: [*The Rust Core Library*](https://doc.rust-lang.org/core/). The Rust Team (Accessed 2022).

[^EAX]: Technically, `%eax` is the lower 4 bytes of the 8-byte `%rax` register on an x86-64 system. `%rax` is used for return values. In this example, we're dereferencing 8-byte pointers but returning a 4-byte integer.

[^UBSan]: [*UndefinedBehaviorSanitizer*](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html). LLVM Project (Accessed 2022).

[^ASan]: [*AddressSanitizer*](https://clang.llvm.org/docs/AddressSanitizer.html). LLVM Project (Accessed 2022).

[^TSan]: [*ThreadSanitizer*](https://clang.llvm.org/docs/ThreadSanitizer.html). LLVM Project (Accessed 2022).

[^CompilerAssurance]: On the other hand, C compilers are mature and well-understood from a safety qualification perspective. And further ahead in formal verification. As an example, the CompCert[^CompCert] C compiler proves that source code semantics match machine code semantics. No current Rust compiler can claim that level or kind of assurance.

[^GCC]: [*The state of static analysis in the GCC 12 compiler*](https://developers.redhat.com/articles/2022/04/12/state-static-analysis-gcc-12-compiler). David Malcom (2022).

[^CompCert]: [*CompCert*](https://compcert.org/). Xavier Leroy (Accessed 2022).