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


# Hands-on Challenge: Port a Program

We've covered the bulk of Rust's most important syntax and features.
While you're likely not comfortable with Rust yet, you now know the basics.

One great way to learn a new language, including its idioms and quirks, is to port an existing program written in a language you're already familiar with.
That's the goal of this challenge.

Now we're not advocates of the "Rewrite-It-In-Rust" (RIIR) trend, although there are some great Rust alternatives to existing tools out there[^RIIR].
Rewriting a large piece of software is a risky proposition that, in many contexts, has questionable payoff.

It's typically wiser to write new features, new services, or hardened components in Rust - such that they can interoperate with existing code.
Chapter 13 will cover integrating Rust into non-Rust codebases.

Our motivation for this challenge is gaining a deeper understanding of Rust via contrast.
Not all idioms and patterns of other languages readily translate, so experiencing those differences first-hand can be enlightening.

## "Failing" is OK

You might get stuck somewhere in your port attempt.
Depending on the program you choose.
That's fine!

If that happens, use this challenge as a "personal watermark" - note how far you've come and what the error was.
You can return to the challenge later, after you've had more Rust experience.
Whenever you're up for it.

## Port a Program of Your Choice To Rust

* Choose a small program (maybe less than 1,000 lines) written in a language you know, and rewrite it from scratch in Rust. We recommend you pick a program you've personally written, especially if you ran into performance limitations. But any program you're deeply interested in is a good choice.

    * Before starting your rewrite, review your program's dependencies. If it uses one or more libraries that have no counterpart on Rust's [crates.io](https://crates.io/), you will either need to choose a different program or also write the dependency yourself. Don't over-scope this challenge!

## Semi-automate a C to Rust Port

* If you're already an experienced C developer, you can try porting an existing C program using the `c2rust`[^C2Rust] tool. It's an unofficial, open-source, best-effort transpiler that ingests C source code and outputs Rust source code.

    * The output Rust is, however, both unidiomatic and as `unsafe` as the input C. Translating C to safe Rust is an open research problem[^C2SaferRust] that involves inferring program semantics. So you will still need to do extensive refactoring.

    * Some readers may prefer to return to this challenge after Chapter 13, which covers CFFI and `unsafe`.

    * If you've never written C before but you're incredibly brave and want or need to learn it, you can still do this challenge! We recommend picking up a copy of *Effective C*[^Ccord] to get started.

<br>

[^RIIR]: [*Awesome Alternatives in Rust*](https://github.com/TaKO8Ki/awesome-alternatives-in-rust). Takayuki Maeda (Accessed 2022).

[^C2Rust]: [*`c2rust`*](https://github.com/immunant/c2rust). Immunant (Accessed 2022).

[^C2SaferRust]: [*Translating C to Safer Rust*](https://sites.cs.ucsb.edu/~benh/research/papers/oopsla21-extended.pdf). Mehmet Emre, Ryan Schroeder, Kyle Dewey, Ben Hardekopf (2021).

[^Ccord]: [***[PERSONAL FAVORITE]** Effective C: An Introduction to Professional C Programming*](https://amzn.to/3wBuNu7). Robert Seacord (2020).
