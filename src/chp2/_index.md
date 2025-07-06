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


# Software Assurance
---

The U.S. Department of Defense (DoD) defines **software assurance** as[^DoD]:

> ... the level of confidence that software functions as intended and is free of vulnerabilities, either intentionally or unintentionally designed or inserted as part of the software.

That's a succinct definition, yet there's depth to unpack.
Regardless of your political ideology or national affiliation, this definition is a shrewd lens through which to reason about the security of a specific system.

## Vulnerabilities: The Root Cause of Insecurity

As someone who's written a program before, you're likely all too familiar with the idea of a **bug** - a mistake that causes your program to misbehave.
A subset of the bugs present in a program may be **vulnerabilities**, meaning they can be *exploited* by an attacker.
Let's contrast two scenarios to clarify the distinction:

1. Can't login with valid credentials?
    * Authentication is broken.
    * Everyone, including legitimate users, *can't* access data.
    * That's a *bug*, the software doesn't work correctly.

2. Can login with invalid credentials?
    * Authentication is broken.
    * Anyone, including an attacker, *can* access data.
    * That's a *vulnerability* which might be exploited to view or modify sensitive data.

While the DoD definition encapsulates a lack of bugs with *"software functions as intended"*, the security-relevant goal is moving toward software that is *"free of vulnerabilities"*.
This book is about both.
We want to build robust and secure systems.

To be frank: no software is *completely* free of vulnerabilities or *absolutely* secure.
A secure system, colloquially, is one in which the cost of an attack far exceeds the value of any assets.
Assets could be hardware, software, or confidential information - any part of the system we should protect.
We can't make attacks impossible, just impractical.

As security practitioners, we strive to ~~weaponize~~ minimize vulnerabilities - just as software engineers strive to minimize bugs.
Fewer vulnerabilities, fewer practical attacks.
But even formal verification, a topic we'll explore in later chapters, has limitations.
There are just too many complex interactions, both with hardware and between software components, for anyone to be totally certain that any given system can withstand every possible threat model.

## Hence Assurance

This is where *"level of confidence"* comes in.
By applying a series of tools and processes, some of which we'll sample in this chapter, we can build confidence in the security of our software.
Broadly speaking, these fall into one of three categories:

* **Static Assurance** - checks done on code without running it, during development and/or testing.

* **Dynamic Assurance** - checks done by running the program, during testing.

* **Operational Assurance** - measures taken when the software is running in **production**.

> **What does "production" mean in this context?**
>
> The environment in which information systems serve their customers.
> Every security decision you make should be driven by the realities of this environment.
>
> For a web application, that might mean "the cloud" (virtual machines provisioned at various geographic locations) for backend components. And "the client" (hardware owned by the end user, like a smartphone running an app or browser) for front-end components.
>
> For an embedded system, production could be a wide variety of adventurous places.
> In the automotive case, it's on a little computer inside the dashboard of your car, connected to both sensors and steering.

We'll build a conceptual foundation for understanding the why and how of each category, and spend the bulk of this book applying that knowledge hands-on.
Since  we're focused on writing code and using tools, we will *not* cover high-level software engineering[^SWEBook] methodologies.
This includes:

* **Systems Development Life Cycle (SDLC[^SDLC]):** a general process for planning, creating, testing, and deploying any information system.

* **Microsoft Security Development Lifecycle (SDL[^SDL]):** a framework for reducing the likelihood and maintenance cost of software security vulnerabilities.

While methodologies like these are valuable blueprints, and you can map concepts from this chapter to them, we won't discuss them. Just know that project-level best practices exist and can provide a shared language for communicating with organizational leadership.

The DoD definition also mentions the idea of *"intentionally...designed or inserted"* vulnerabilities, commonly called *back doors*.
Because code we write will only have a few well-trusted dependencies outside of Rust's standard library[^StdVuln], we won't be concerned with detecting back doors.
But you will see a simple backdoor in this chapter, to get a more visceral feel for the topic.

Rust is a promising security technology because it provides an unprecedented level of **assurance** under systems programming constraints.
The Rust compiler can prove the absence of a vicious class of vulnerabilities, colloquially referred to as "memory corruption bugs", while simultaneously matching the bare metal performance of languages that can't provide the same guarantee.
We'll deep-dive memory mechanics and safety in the next chapter, but it's worth reiterating as we head into this one.

## Your First High Assurance Rust Program

Theory is the foundation, but growth requires hands-on experience.
Let's hit the ground running.
The latter half of this chapter will be your first taste of **high assurance systems programming** in Rust.
In less than 200 lines of code (including tests), you will:

* Implement a tiny cryptographic library capable of running in nearly any embedded environment.

* Verify your implementation using officially-released test vectors.

* Insert a naive back door to understand where dynamic testing fails.

* Add a command line front-end so you can use your library to encrypt local files.

Despite the miniscule line count, our tool will be a modular system.
Composed of trustworthy components:

<p align="center">
  <figure>
  <img width="100%" src="rcli_model.svg">
  </figure>
</p>

Our 200 lines are those green boxes, the safe Rust components.
Both components carry guarantees with respect to memory safety.
Due to how we'll test, the encryption library carries evidence of logical correctness as well.

> **Memory Safety Validation for 3rd Party Libraries**
>
> Rust projects can enable an optional attribute: `#![forbid(unsafe_code)]`.
> It makes any use of `unsafe` a compile-time error, within the boundary of a single binary or library.
>
> Building a 3rd-party `#![forbid(unsafe_code)]` dependency from source allows the compiler to automatically verify that code procured by an external entity is memory-safe.
> Barring a bug in the compiler itself.

But real-world teams can't expect to validate every single byte of executable code in any non-trivial system.
Whether the validation is for memory safety or some other property.
To ship quickly, we:

* Rely on Rust's standard library, and widely-used 3rd party libraries, to build the front-end.

* Transitively rely on `libc` - the C standard library (dynamic memory allocator, POSIX APIs, etc) - to build the front-end.

* Transitively rely on a mature operating system to deliver interactive functionality to the end-user.

Our ambition is to eliminate high-level design flaws, logic bugs, and memory errors in the code we write.
If an attacker's only viable option is finding vulnerabilities in the standard library, the latest version of a well-known 3rd party dependency, or the OS itself - then the *cost* to compromise our system is likely *high*.

## But is it *truly* high assurance?

No, we intentionally make a concession: using RC4, a broken encryption algorithm.
Two reasons:

* To keep source line count low. RC4 is simple, so it works as an example.

* To motivate you to take on this chapter's challenge, which asks you to switch the modular CLI tool to a modern encryption backend.

RC4 was once part of protocols our society relies on, like SSL/TLS and WEP.
But since its debut in 1987[^Rivest], multiple weaknesses have been found and several practical attacks have been demonstrated.

This is a microcosm of an important axiom: assurance is a moving target.
As the security landscape changes, so must requirements and measures taken.

## Motivation

Before pressing forward, let's take a quick step back: why prioritize software assurance in the first place?

We'd wager another DoD statement sums it up nicely.
Feel free to replace *"mission critical"* with "business critical" in the below description of the price of insecure software[^DoD]:

> Consequences: The enemy may steal or alter mission critical data; corrupt or deny the function of mission critical platforms

Welcome to cyberspace.
Let's secure the frontier!

## Learning Outcomes

* Understand the tradeoffs between static and dynamic analyses
* Understand the role of operational deployment measures
* Write your first interesting Rust program: a tiny encryption tool
* Learn how to build a statically-linked executable (works on nearly any Linux client)

[^DoD]: [*DoD Software Assurance Initiative*](https://www.acqnotes.com/Attachments/DoD%20Software%20Assurance%20Initiative.pdf). Mitchell Komaroff, Kristin Baldwin (2005, Public Domain)

[^SWEBook]: [*Guide to the Software Engineering Body of Knowledge*](https://cs.fit.edu/~kgallagher/Schtick/Serious/SWEBOKv3.pdf). Pierre Bourque, Richard E. Fairley (2014)

[^SDLC]: [*Systems development life cycle*](https://en.wikipedia.org/wiki/Systems_development_life_cycle). Wikipedia (Accessed 2022).

[^SDL]: [*Microsoft Security Development Lifecycle (SDL)*](https://www.microsoft.com/en-us/securityengineering/sdl). Microsoft (2021)

[^StdVuln]: Rust's standard library, like any large piece of software, it not guaranteed to be free of vulnerabilities. Two previously discovered vulnerabilities include a memory safety error in `unsafe` code[^CVE1] and a Time-of-check-to-time-of-use (TOCTTOU) race condition[^CVE2]. But `std` is a widely used component maintained by an official Rust team, so we can generally trust it more than 3rd party packages. Especially when it comes to back doors.

[^Rivest]: Coincidentally, Ron Rivest, inventor of RC4, also co-invented scapegoat trees - the data structure we'll implement in Chapter 7. Scapegoat trees never enjoyed the popularity of RC4, but they've certainly stood the test of time.

[^CVE1]: [*Analysis of CVE-2018-1000657: OOB write in Rust's VecDeque::reserve()*](https://gts3.org/2019/cve-2018-1000657.html). GeorgiaTech SSLab (Accessed 2022).

[^CVE2]: [*Security advisory for the standard library (CVE-2022-21658)*](https://blog.rust-lang.org/2022/01/20/cve-2022-21658.html). The Rust Team (Accessed 2022).

[^AEAD]: [*Authenticated encryption*](https://en.wikipedia.org/wiki/Authenticated_encryption). Wikipedia (Accessed 2022).
