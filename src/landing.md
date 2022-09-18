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

<p align="center"><img src="../img/har_logo.svg" width="65%" alt="High Assurance Rust"></p>

<style>
h1, h2, h3 {
    display: inline;
}

h1 {
  font-size: 3em;
}

h3 {
  font-size: 1.875em;
}
</style>

<div class='token'>
<center>
  <h1>High Assurance Rust</h1>
  <br>
  <h3><i>Developing Secure and Robust Software</i></h3>
</center>
</div>

---

This book is an introduction to building performant software we can **justifiably trust**.
That means having sufficient data to support confidence in our code's functionality and security.
Trustworthiness is a hallmark of **high assurance** software.

With assurance as our driving concept, we'll take a hands-on, project-based approach to two fundamental but often inaccessible topics in software development: **systems programming** and **low-level software security**.

You'll learn Rust - a modern, multi-paradigm language that emphasizes speed and correctness.
Most programming books teach a new language by presenting a dozen small, unrealistic programs.
Not this one.

We'll design, write, and validate a fully-featured alternative to the ordered map and set implementations in Rust's standard library.
You'll gain a deep understanding of the Rust language by re-implementing one of its major dynamic collections, one idiomatic API at a time.

Unlike the standard version, our implementation will be:

* **Maximally Safe.** Upholds Rust's strongest memory safety guarantees, for all possible executions.
  * To test properties the compiler can't prove, we'll learn advanced program analysis techniques, including *differential fuzzing* and *deductive verification**.

* **Extremely Portable.** Capable of running on every operating system, or even without one (e.g. "bare metal").
  * Our library is a *hardened component*. To integrate it within larger codebases, we'll add *CFFI bindings* to make the Rust functions callable from other languages - including C and Python.

* **Highly Available.** Offers *fallible* APIs for handling cases that could otherwise result in a crash.
  * E.g. *Out-of-Memory (OOM) error* - when all pre-allocated memory has been exhausted.

## The State-of-the-Art in Practical Software Assurance

We'll use cutting-edge, open-source software assurance tools to validate the code we write in this book.
Some of these tools are mature and used in commercial industry:

* `rustc` (modern compiler)
* `libFuzzer` (fuzz testing framework)
* `rr` ("time-travel" debugger)
* `qemu` (whole-system emulator)

Other tools are experimental and under active research.
A full inventory is available in [the appendix](../chp16_appendix/tools.md).

Visually, this book covers the below topics (contrasted roughly on tradeoff of **development speed** and **formal rigor**).
Don't worry, we'll provide clear explanations and context for each.

Notice the bias toward development speed.
We're interested in **lightweight processes** that, in the long run, enable us to **ship quality code faster** and spend **less time patching** security and reliability failures.
Techniques you can apply to real-world code.
Today.

</br>
<p align="center"><img src="../img/book_topics.svg" width="90%" alt="Assurance Techniques"></p>

Unlike other Rust books, you won't just learn the language.
You'll learn how to *reason* about software security at the leading edge.
To think like an attacker.
And to write code resistant to attack.
That mental model is valuable no matter what programming language you primarily use.

## Sponsors Supporting this Book

The development of this book (research, writing, and coding) is made possible through the generous support of:

<style>
p#sponsor {
  margin: 0 auto;
  text-align: center;
  width: 80%;
}
</style>

<p id="sponsor">
</br>
<a href="https://foundation.rust-lang.org/">
<img src="../img/rust_foundation_logo.png" width="70%" alt="The Rust Foundation">
</a>
</br>
</br>
Under the first tranche of the <a href="https://foundation.rust-lang.org/grants/">2022 Project Grants Program</a>.
A full list of awarded projects is <a href="https://foundation.rust-lang.org/news/2022-06-14-community-grants-program-awards-announcement/">available here</a>, please check out the range of exciting work happening within the global Rust community!
</br>
</br>
</p>

> *You need to build a data structure library to serve a mission-critical application.
> It must run on nearly any device, operate in the field for years on end without error, and tolerate attacker-controlled input.
> There will be no patches and there can be no failures.
> Your code must survive.
> **Ship strong.***

---

> \* == may be subject to change! This book is a [work in progress](./faq.md#8-is-this-book-free). If you're like to be notified when it's finished and a physical print is available, please [sign up here](https://forms.gle/ESYgXgswCjEoCSHT9).
