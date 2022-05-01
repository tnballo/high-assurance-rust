# Limitations and Threat Modeling

The trouble with computer security is that there's always bad news.
As security practitioners, we have to be keenly aware of limitations.
That is:

* What are we *not* confident in?

    * If we can't propose potential weaknesses, then we're very likely over-estimating our strengths. That's a risky place to be.

* Against which *threats* does a type system provide little assurance?

    * The composition of all **threat vectors**  decides our **attack surface**. As it grows, we present more potential entry points for an attacker. The riskiest surfaces are often at **trust boundaries**.

        * *Threat vectors:* Potential routes of access for an attacker.

        * *Attack surface:* The set of threat vectors present in a given system.

        * *Trust boundaries:* Interfaces between less-trusted and more-trusted components of a system.

* Which *functional requirements* can't be validated by our dynamic test suite?

    * Here lurk potential **design flaws**. If these oversights are discovered after a system is deployed, fixing them can be costly.

        * *Design flaw:* A flaw in fundamental functionality (as opposed to a bug in the code) that causes a system to fail to meet requirements. At the level of a principle or a pattern.

This section provides broader context for the static and dynamic assurance claims we've made thus far.
A reality check, if you will.
To cover a wide spectrum of related topics quickly, we'll jump around a little bit.

## Manual Static Analysis Forever!

Consider three potential causes of a security vulnerability.
Each is incredibly difficult to design an effective automated analysis for, static or dynamic, because each bug is almost impossible to define in a universally-applicable way.

1. **Improper Input Validation:** Fail to validate that user-provided data is both syntactically and semantically correct.

    * *Example:* A web portal uses only client-side (e.g. bypass-able!) JavaScript to validate form entries, and one of the inputs is passed to a server-side shell when a command gets executed.

2. **Information Leakage:** Expose sensitive[^IFC] or superfluous information.

    * *Example:* An authentication service logs verbosely for internal troubleshooting purposes, but the logs include users' plaintext passwords - sensitive data otherwise stored only in hashed form.

3. **Misconfiguration:** Introduce vulnerabilities by choosing incorrect or suboptimal parameters during the configuration of a system.

    * *Example:* A network router allows password authenticated SSH login and the same default password is used for every device shipped.

In reality, even the fanciest of type systems coupled with the most comprehensive of test suites can't guarantee security.

* **Static Limit:** Most vulnerability classes can't be eliminated by properties encoded within the semantics of a practical type system[^TypeState].

* **Dynamic Limit:** Many complex states aren't covered by tests[^DirtyPipe] and are difficult to replicate in production.

Moreover, on the static side of things, rules for converting between types can be misunderstood by programmers.
Especially in languages with complex class hierarchies.
Sometimes casting errors introduce *type confusion* vulnerabilities, where memory encoding one data type is misinterpreted as another - breaking type safety guarantees.

> **How does type confusion happen in C++?**
>
> Although C++ is statically typed, it remains a type-unsafe language.
> On the extreme end, its weak typing means programmers can cast between arbitrary types that have no logical relationship.
>
> More commonly, programmers cast between types within a given hierarchy of objects.
> This makes logical sense in the context, but introduces potential for subtle errors.
> Despite a static check passing, we can experience serious type issues at runtime:
>
> * **1st-order type confusion:** Say someone mistakenly casts an instance of a parent class to one of its descendants, and the parent lacks some of the descendant's fields. When one of the missing fields is accessed, we might treat arbitrary data as a pointer!
>
> Moreover, memory corruption bugs undermine type safety in general:
>
> * **2nd-order type confusion:** Maybe a program doesn't contain any casting errors, but a memory corruption bug elsewhere allows an attacker to write an object's memory. Specifically, to overwrite a pointer within an object's internal method dispatch table. We could arrive at the same arbitrary pointer problem.
>
> Scenarios like these are how browsers, virtual machines, and databases written in C++ are commonly exploited[^HexType].
> By contrast, Rust is type-safe (the language forces all type conversion to follow strict, statically-enforced rules[^RustTypes]) and memory-safe (no corruption).

We still need manual static analysis done by skilled humans, in the form of audits.
That means knowledgeable people reading code and design documents to find vulnerabilities before an attacker does.

We won't exhaustively enumerate high priority bug classes in this book.
There are already high-quality, freely-available resources for this purpose.
Two of the most impactful are the *MITRE CWE Top 25 Most Dangerous Software Weaknesses*[^CWETop25] and then *OWASP Top 10 Web Application Security Risks*[^OWASP].
The former describes itself as follows[^CWETop25]:

> The 2021 Common Weakness Enumeration (CWE) Top 25 Most Dangerous Software Weaknesses (CWE Top 25) is a demonstrative list of the most common and impactful issues experienced over the previous two calendar years. These weaknesses are dangerous because they are often easy to find, exploit, and can allow adversaries to completely take over a system, steal data, or prevent an application from working.

According to the CWE Top 25[^CWETop25], "Out-of-bounds Write", a memory safety issue, was the #1 weakness in 2021.
In fact, the list includes a handful of issues safe Rust eliminates entirely (e.g. Out-of-bounds Read, Use After Free, `NULL` Pointer Dereference).

But let's not give Rust too much credit.
The majority of enumerated weaknesses are completely outside of the domain Rust's compiler can reason about.
Even though Rust software has a head start in knocking out the top 25, software assurance *must* include *manual* processes.
Remember: tools are a supplement for scale, not a substitute for smarts.

## Example: Dynamic Testing Failing to Detect an RC4 Backdoor

March 2020 marked the start of the global Covid-19 pandemic[^Covid19].
Far less significant, but more relevant to this book: that same March saw one of the biggest security breaches in history - over 30,000 public and private organizations fell victim to malware spread via a backdoored product[^SolarWinds].
SolarWinds, the company behind the product, was unaware that malicious code had been inserted into a system at the time they shipped updates to their customers.

Let's make our limitations discussion more concrete with an example.
We'll demonstrate that Rust's static checks and our dynamic test vectors can't guarantee that our RC4 implementation is completely secure.

Imagine an attacker has a foothold on the network and can sniff traffic (e.g. "man-in-the-middle").
They've compromised our organization's build system, SolarWinds style, and backdoored the implementation of our RC4 library like so:

```rust,ignore
impl Rc4 {
    // ..new() definition omitted..

    // ..apply_keystream() definition omitted..

{{#include ../../code_snippets/chp2/crypto_tool/rc4/src/lib.rs:apply_keystream_static_backdoored}}

    // ..prga_next() definition omitted..
}
```

Notice the additional bit of malicious logic in `apply_keystream_static`: if the data to be encrypted starts with the text `ADMIN_TOKEN`, the function ignores the provided key and uses a hardcoded one!

Given prior knowledge of the hardcoded key, a **passive** network attacker may decrypt sensitive data in real-time, compromising the confidentiality of any system using the backdoored library.
An **active** network attacker might intercept the token and leverage it for lateral movement to new internal systems.

Worse yet, if we run `cargo test` all tests will pass.
Even if we had invested time in porting every single table/vector in the IETF document[^TestVec], our dynamic test suite would still fail to detect this backdoor.

The problem is that the malicious branch, the first block of the inserted `if`, is only taken when `ADMIN_TOKEN` appears at the start of the data.
That's a single execution path we're incredibly unlikely to hit - we wouldn't know to test for an arbitrary and obscure condition like this one.
And if our build system was compromised, an implant probably wouldn't backdoor code for test builds anyway.

> **How "unlikely"?**
>
> Say we add a supplemental test to our RC4 library that uses randomness.
> It verifies that, if we encrypt a random message with a random key, later decrypting with the *same key* produces the *original message*.
>
> `ADMIN_TOKEN` is a 11 character string.
> With an alphabet restricted to only upper and lowercase characters (no digits/symbols, etc), there are `52^11` possible 11 character strings.
> The chance of hitting `ADMIN_TOKEN` at random is only 1 in 7 *quintillion*.
> Basically zero.
> And that search space is too large to test exhaustively.

This demonstrates the achilles heel of dynamic analysis: any single execution is only a tiny sample of a program's state space.
We can test for the presence of specific bugs, should we have the foresight to write a relevant test case, but can never prove the absence of bugs.
Or backdoors.

> **Practical Backdoors and Evasion**
>
> Our backdoor could be detected statically.
> We branch, on a comparison to a specific constant, to overwrite a provided parameter.
> It's possible to generate a "signature" for this pattern or something similar - even without access to source code.
>
> While we could compensate via *packing* and *obfuscation* (techniques that make our code harder to analyze, both manually and automatically), that could backfire and make the backdoored program appear even more suspicious.
>
> While this book isn't focused on offense, we should be generally aware of evasion tactics to be effective defenders.
> For cryptographic code, that means subverting the *algorithm* in a subtle way that adversely affects its mathematical guarantees[^DHBackdoor].
> Not inserting an `if` statement, like the above example (which doesn't even handle the decryption case).

## Situational Context for Real-world Systems

Encryption alone does not guarantee *integrity* (that data hasn't been modified or corrupted) or *authenticity* (that data came from a trusted source).
And, again, the RC4 cipher is no longer considered secure.

For real-world use, we'd likely want a cipher that supports Authenticated Encryption with Associated Data (AEAD)[^AEAD].
To validate both integrity and authenticity, while still protecting confidentiality.
The RustCrypto organization, a community effort, maintains a list of open-source, pure-Rust crates[^AEADList] that offer AEAD-compatible ciphers (both stream and block).

Remember how RC4 was once part of major protocols, like SSL/TLS and WEP?
In protocol design, even an AEAD cipher isn't enough to meet all security requirements.
Imagine a network attacker does the following:

1. Listen to valid broadcast packets.

2. Store a copy of each packet received.

3. Retransmit stored packets to their original destination, perhaps later.

That retransmission is called a *replay attack*.
The attacker never decrypts, modifies, or forges a packet - AEAD is never compromised.
Yet the results can be devastating:

* Say the attacker re-transmits aggressively.
The destination server could be overwhelmed with the packet volume.
If it's busy processing the attacker's copies, it may miss response deadlines for legitimate users. That means suffering service degradation, or even *Denial of Service (DoS)*.

    * Example defense: per-client rate-limiting.

* Say the attacker only re-transmits once, but the message was to confirm a deposit and increase a user's account balance.
Maybe the attacker just doubled their money.

    * Example defense: trusted timestamps in encrypted payload, only commit transactions with newer stamp than last commit (and handle possible wrap-around).

Cryptography is only one factor in the security of real world protocols and systems.
Encryption algorithms address only one [core] part of an overall threat model.

## The Big Picture: Threat Modeling

We've alluded to a "big picture" perspective, system design context beyond individual blocks of code, several times in this section.
High-level security design review involves a process called **threat modeling**[^OSWAPThreat].
Generally, the workflow is something like this:

1. Identify assets (data or resources of value) within a system.

2. Review each asset's attack surface and enumerate the corresponding threats. Looking for sources of untrusted input can be a good way to start.

3. Rank the threats. One general way to prioritize is `risk = likelihood * severity`.

4. Implement controls and mitigations proportional to ranked threats.

5. Test the efficacy of mitigations and controls.

Threat modeling is most valuable when done early in a product's lifecycle, like during architectural design (before code has been written).
The earlier we can catch and remediate issues, the cheaper and easier they are to fix (aka "shift left" security).

There exist several methodologies for threat modeling, and entire books covering them.
Though closer to a taxonomy than a full methodology, **STRIDE** is one popular and enduring approach.
It was adopted by Microsoft in 2002 and has evolved since.
The acronym is broken down as follows[^SEIThreat]:

| Letter | Threat | Threat Definition | Property Violated |
| --- | --- | --- | --- |
| **S** | **Spoofing**  | Pretending to be someone or something | Authentication |
| **T** | **Tampering** | Unauthorized modification or deletion of data | Integrity |
| **R** | **Repudiation** | Claiming that you didn't do something | Non-repudiation |
| **I** | **Information Disclosure** | Exposing sensitive information to an unauthorized party | Confidentiality |
| **D** | **Denial of Service** | Exhausting resources needed to provide a service | Availability |
| **E** | **Elevation of Privilege** | Allowing an action to be performed by an unauthorized party | Authorization |

STRIDE is intentionally high-level.
It aims to be applicable to a broad range of products, systems, and services.
Hence the focus on generally desirable properties.

More granular threat enumeration frameworks can aid realistic threat model development.
**MITRE ATT&CK** is a modern example, it self-describes as[^MIREAttack]:

> ...a globally-accessible knowledge base of adversary tactics and techniques based on real-world observations. The ATT&CK knowledge base is used as a foundation for the development of specific threat models and methodologies in the private sector, in government, and in the cybersecurity product and service community.

Whereas limited aspects of code analysis *may* be supplemented with tools (e.g. the Rust compiler proving memory safety), automating threat modeling would require general artificial intelligence.
In other words, it's a task for which you should budget person hours - early and up front.

## Takeaway

Powerful as they are, type systems can't model most bug classes.
Rust doesn't solve the majority of real-world security problems.
Manual analysis, meaning code review and threat modeling, is a necessity.

Sitting in between the compiler's type checks and manual review is a form of semi-automated validation: dynamic analysis.
This usually takes the form of unit tests, run after building the code but before asking peers to review it.

While these tests help bolster confidence and can catch serious issues, they *cannot* prove the *absence* of bugs or back doors.
For real-world programs (not the naive backdoor above), this is true even if the test suite achieves 100% code coverage - meaning every branch of the program is taken at least once.

Why? Because the state space of a complex application is not solely a function of branches taken, it also depends on the value of pieces of data (among other factors).
And as we saw in the 11 character permutation example, we can't exhaustively test every possible value for even a tiny piece of data.

More generally, no automated tooling will ever be able to reliably determine if a program is malicious.
Or if a benign program is secure.
Making that determination is provably impossible (Rice's Theorem[^RiceTheorem]).

Systems we *consider* secure are typically those that have stood the test of time.
Like well-studied protocols or heavily audited open-source projects.
But a new vulnerability is always a possibility.

Remember: there is *no absolute* security.
Only levels of *assurance*.

With all of this context out of the way, let's return to our RC4 library and leverage it for something useful: encrypting local files via a command line interface.

---

[^IFC]: [*Compositional Information Flow Monitoring for Reactive Programs*](https://kilthub.cmu.edu/articles/report/Compositional_Information_Flow_Monitoring_for_Reactive_Programs/19214415). McKenna McCall, Abhishek Bichhawat, Limin Jia (2022). Information Flow Control (IFC), this paper's field, explores ways to address sensitive information leakage. Though formal, the problems tackled by this work - namely its titular support for event-driven programs and composition of heterogenous components - are representative of real-world systems. Information leakage might eventually be a problem we can solve in a systematic and principled fashion.

[^TypeState]: Although the "type state" pattern (in general, not unique to Rust) can help a bit. And "Session types"[^SessionTypes] are particularly useful for message-passing protocols.

[^DirtyPipe]: [*The Dirty Pipe Vulnerability*](https://dirtypipe.cm4all.com/). Max Kellerman (2022).

[^HexType]: [*HexType: Efficient Detection of Type Confusion Errors for C++*](https://dl.acm.org/doi/pdf/10.1145/3133956.3134062). Yuseok Jeon, Priyam Biswas, Scott Carr, Byoungyoung Lee, and Mathias Payer (2017).

[^RustTypes]: In Rust, certain primitive types can be converted with the `as` keyword. Integers of different sizes are one example. Conversions between user-defined types use "traits" (interfaces for shared behavior we'll cover in chapter 3). Specifically `From`[^TraitFrom] and `Into`[^TraitInto]. These safely cover the majority of type casting use cases. If a Rust programmer *really* needs to arbitrarily re-interpret a sequence of bits, the `unsafe` function `std::mem::transmute`[^FuncTrasmut] is a last resort.

[^FuncTransmut]: [*Function `std::mem::transmute`*](https://doc.rust-lang.org/std/mem/fn.transmute.html). The Rust Team (Accessed 2022).

[^CWETop25]: [*CWE Top 25 Most Dangerous Software Weaknesses*](https://cwe.mitre.org/top25/archive/2021/2021_cwe_top25.html). The MITRE Corporation (2021).

[^OWASP]: [*Top 10 Web Application Security Risks*](https://owasp.org/www-project-top-ten/). OWASP (2021).

[^TestVec]: [*Test Vectors for the Stream Cipher RC4*](https://datatracker.ietf.org/doc/html/rfc6229). Internet Engineering Task Force (2011).

[^Covid19]: [*Our Pandemic Year—A COVID-19 Timeline*](https://www.yalemedicine.org/news/covid-timeline). Kathy Katella (2021).

[^SolarWinds]: [*SolarWinds hack explained: Everything you need to know*](https://whatis.techtarget.com/feature/SolarWinds-hack-explained-Everything-you-need-to-know). Saheed Oladimeji, Sean Michael Kerner (2021).

[^DHBackdoor]: [*How to Backdoor Diffie-Hellman*](https://eprint.iacr.org/2016/644.pdf). David Wong (2016).

[^AEAD]: [*Authenticated encryption*](https://en.wikipedia.org/wiki/Authenticated_encryption). Wikipedia (Accessed 2022).

[^AEADList]: [*RustCrypto: Authenticated Encryption with Associated Data (AEAD) Algorithms*](https://github.com/RustCrypto/AEADs). RustCrypto organization (Accessed 2022).

[^RiceTheorem]: [*Rice's theorem*](https://en.wikipedia.org/wiki/Rice%27s_theorem). Wikipedia (Accessed 2022).

[^SessionTypes]: [*An Introduction to Session Types*](https://wen.works/2020/12/17/an-introduction-to-session-types/). Wen Kokke (2020).

[^TraitFrom]: [*Trait `std::convert::From`*](https://doc.rust-lang.org/std/convert/trait.From.html). The Rust Team (Accessed 2022).

[^TraitInto]: [*Trait `std::convert::Into`*](https://doc.rust-lang.org/std/convert/trait.Into.html). The Rust Team (Accessed 2022).

[^OWASPThreat]: [Threat Modeling Cheat Sheet](https://insights.sei.cmu.edu/blog/threat-modeling-12-available-methods/). OWASP (2021).

[^SEIThreat]: [Threat Modeling: 12 Available Methods ](https://insights.sei.cmu.edu/blog/threat-modeling-12-available-methods/). CMU SEI (2018).

[^MITREAttack]: [MITRE ATT&CK](https://attack.mitre.org/). MITRE (Accessed 2022).
