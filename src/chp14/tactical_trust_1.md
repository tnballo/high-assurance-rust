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

# Tactical Trust: Platform Cryptography for Developers (1 of 2)

<!---
* Side-by-side diagrams
--->

<style>
  .diagram-row {
    display: flex;
    justify-content: space-evenly;
    align-items: center;
  }

  .diagram-col {
    flex: 50%;
    max-width: 35%;
    padding-top: 1%;
    padding-bottom: 4%;
  }

  .diagram-col2 {
    flex: 50%;
    max-width: 40%;
    padding-top: 1%;
    padding-bottom: 4%;
  }

   .diagram-solo {
    flex: 100%;
    max-width: 75%;
    padding-top: 1%;
    padding-bottom: 4%;
  }
</style>

Digital systems our society relies on all have some notion of **trust**.
A communicating party can identify, with confidence, *who* they are "talking" to (authentication).
And they can rest assured that their "conversation" is *private* (confidentiality).
Even non-networked systems will validate that code they flash or execute hasn't been *modified* or *corrupted* (integrity).

Cryptographic libraries are the technical mechanism underpinning properties like authentication, confidentiality, and integrity.
These imperfect software components are the foundation on which societal trust is built and maintained.
Thus exploitable flaws in crypto libs tend to have severe and widespread impact[^MatterHeartbleed].

Now this two-part section isn't about applied cryptography in the proper academic sense, we won't explain cryptographic primitives or protocol design from the ground up.
Let's assume those more formal concepts live in an ivory tower[^Ivory].
We're medieval peasants fighting in mud that is long-surviving production software - shipping, patching, refactoring.

These sections are concerned with brutal realities of deploying [theoretically] sound designs - we aim to reduce certain risks inherent to real-world software.
It's one interpretation of what **platform security engineering** entails when shipping trust at scale.
Concepts we cover are language-agnostic: they likely apply to your problem domain and tech stack of choice.

> ***How are we defining "platform security engineering"?***
>
> Building libraries, frameworks, and tools that allow feature teams to ship both securely and quickly.
> Essentially providing a "solid security foundation" for high-velocity software.
> In terms of code-level consistency.


## So what's the agenda for this two-part section?

Part 1 focuses on *code* (full, runnable source is available within the book repo under `code_snippets/chp14/tactical_trust`).
Our proof-of-concept programs aim to raise the bar for "shift left" automation, even if modestly (give the attacker an inch, they'll take a mile).
We'll sample solutions to two cryptographic platform security problems at different levels of the stack:

* **API:** Can we systematically prevent nonce reuse vulnerabilities in an arbitrarily-large codebase?

* **Supply-chain:** How should CI enforce policies specific to cryptographic dependencies?

Part 2 will focus on *concepts* but still include plenty of code. The emphasis is higher-level exploration of a {problem,solution} space.
We'll narrow scope to the problem of *information disclosure*, deep-diving vulnerabilities and state-of-the-art mitigations through the lens of two general threat models:

* **Man-in-the-Middle (MITM):** Attacker intercepts network communications between two or more endpoints.

* **Man-at-the-End (MATE):** Attacker directly compromises one or more communication endpoints.

> ***What if I'm interested less in software engineering, more in cryptographic design?***
>
> Good news, everyone![^GoodNews]
> Although we're focused on code-level tactics, there's several quality, strategy-focused resources to meet you wherever you're currently at and help you construct correct designs. Here's a sample:
>
> * Crypto novice but an experienced developer? &#8594; ["Real-world Cryptography" by Dave Wong](https://amzn.to/43ov045)[^RWCBook]
>
> * Work in applied cryptography professionally? &#8594; [Soatok's Cryptography Blog](https://soatok.blog/category/cryptography/)[^SoatokBlog]
>
> * At the cutting-edge of near-future cryptography? &#8594; [Real World Crypto Symposium](https://rwc.iacr.org/)[^RWCConf]

## API: Prevent Nonce Reuse with Stronger Types

"Nonce" is a portmanteau of "<span style="text-decoration: underline;">n</span>umber used only <span style="text-decoration: underline;">once</span>".
As the name implies: accidentally using the same nonce multiple times, aka *nonce reuse*, is a devastating footgun for many widely-used cryptographic algorithms.
Common operations rely on a random nonce as input in order to uphold critical security properties:

* **Encryption** - Unique nonces are often called "Initialization Vectors" (IVs). They prevent *plaintext and/or key recovery* as well as *replay attacks* (malicious repetition of previous communications).

  * WPA2 was the de facto standard for encryption on Wi-Fi networks from 2006 to 2020. Toward the end of that lifespan, researchers demonstrated a practical attack against all implementations [^KRACK]. By abusing re-transmission logic in the 4-way handshake between a Wi-Fi endpoint and a client joining the network, an attacker could force *reset/reuse* of the nonce/IV for all protocol-supported stream ciphers (e.g. "keystream reuse"). That means an attacker can decrypt, replay, and [in some cases] forge network packets. Full compromise of the transport layer (e.g. TCP but not HTTPS).

* **Signing** - Unique nonces prevent *signature forging* (generating a passing signature for attacker-created data) and *signature duplication* (replay of previously-signed data).

  * The Sony PlayStation 3 was poised to become the most secure game console ever made, with no true jailbreak 4 years into production. The PS3 used ECDSA to create a chain-of-trust from early boot to userspace app launch - cryptographically enforcing software license checks. ECDSA signing takes as input a nonce and a hash of data to sign. Hackers discovered  that Sony's implementation used a hardcoded nonce[^PS3Signing]. This flaw enabled trivial re-computation of the ECDSA *private* signing key and therefore attacker ability to execute arbitrary unlicensed software.

<br>
<body>
  <div class="diagram-row">
    <div class="diagram-solo">
      <img src="tt_nonce_reuse.svg" alt="Nonce reuse in context of encryption">
      <br>
      <figcaption><i><center><b>Fig. 1:</b> Nonce reuse: a single nonce used for multiple encryption operations (red input, step 3+).</center></i></figcaption>
    </div>
  </div>
</body>

So then: how do we prove that, in some arbitrarily-large codebase, all nonces are both random and single-use?
By encoding safety invariants into the language's *type system*.
We can create APIs that are nearly impossible to misuse, and we get automatic static verification of that correctness just by compiling a program which uses exclusively the safe APIs!

Bold claim, yet relatively straight-forward implementation:

```rust,ignore
{{#include ../../code_snippets/chp14/tactical_trust/nonce_typing/src/lib.rs:nonce_typing}}
```

`Aead`[^TraitAead] is a widely-used trait in the Rust cryptography ecosystem.
It defines a common interface to the `encrypt` and `decrypt` operations of Authenticated Encryption with Associated Data (AEAD) algorithms like AES-256-GCM and XChaCha20Poly1305.
This class of algorithms provides both confidentiality and integrity, plus optionally allows binding unencrypted, "associated" metadata (think network headers, UUIDs, or contextual info).
Basically, an AEAD should be your preferred all-in-one solution for most day-to-day encryption problems.

Now the `Aead` enc/decrypt APIs[^APIAead] both take a single nonce type by reference: `&Nonce<A: AeadCore>`. So a programmer is free to encrypt new data with the same nonce they used for decryption earlier (see Figure 1 above).

* Notice how a nonce is generic over the `AeadCore` trait, allowing *compile-time* verification of algorithm-specific array sizes - e.g. `[u8; 12]` (96-bit) for AES-256-GCM, `[u8; 24]` (192-bit) for XChaCha20Poly1305 - at *all* call-sites.

The crux of our above reuse solution is this: we use two distinct nonce types, `EncryptionNonce<A: AeadCore>` for `encrypt` and `DecryptionNonce<A: AeadCore>` for `decrypt`.
This bifurcation prevents nonce-reuse vulnerabilities, again at *compile-time* (before shipping and systematically across the entire codebase), because:

* `EncryptionNonce` is guaranteed to be *randomly-generated* (opaque type with rand-only constructor) and *single-use* (pass-by-value parameter semantics). The single-use property is especially amenable to Rust's [linear] type system. Its decryption counterpart, alias `type DecryptionNonce<A> = Nonce<A>;`, continues to work normally.

* Marker trait `CryptoRng`[^TraitCryptoRng] in `fn generate_nonce(rng: impl CryptoRng + RngCore)` is critical. A *biased* (meaning not uniformly random) nonce can be as disastrous as a reused nonce. In another ECDSA debacle, biased nonces allowed extraction of Bitcoin private keys[^BitFail].

> ***What about "nonce misuse-resistant" algorithms? And size limitations?***
>
> Strong typing isn't the only possible solution for nonce-reuse.
> Defenses can also be implemented in the design of the algorithm itself, see AES-GCM-SIV[^AesSIVRFC].
> A "Synthetic Initialization Vector" (SIV) uses inputs, including plaintext, to derive the final IV/nonce - effectively forcing two different plaintexts to use two different nonces.
>
> However: if the *same message* is encrypted with the *same nonce* <span style="text-decoration: underline;">twice</span> under the *same key*, an attacker will learn that the two messages are equivalent (but not their contents).
> That equivalence leak could have serious implications in context of a larger threat model, so preventing reuse with strong typing is still the higher assurance option.
>
> But we're not out of the woods yet.
> AES-256-GCM can only safely encrypt 2<sup>32</sup> (~4.3 billion) messages[^AdamAEAD] under the same key using random nonces - beyond that we risk *nonce collision* (chance reuse). XChaCha20Poly1305 bumps that safe limit to 2<sup>80</sup> (practically infinite!)[^XChaCha] and is faster on devices without hardware support for AES.

We can verify that the `NonceSafeAead` trait enc/decrypts as expected with the below unit test:

```rust,ignore
{{#include ../../code_snippets/chp14/tactical_trust/nonce_typing/tests/encrypt_decrypt.rs:demo_test}}
```

But does it actually prevent reuse?
You're welcome to try passing the same `enc_nonce` to two different `nonce_safe_encrypt` calls - the compiler error should look familiar!

> ***Where do I start with "formally verified" cryptography?***
>
> Proving that a program satisfies a specific property, for any input, is the goal of *formal verification*.
> Rust's type system, which guarantees that data is "shared XOR mutable", is particularly amenable to certain formal techniques - less reasoning about the state of memory is needed.
> Cryptography is also lower-cost to verify: detailed specifications exist, data structures are statically-allocated, and input size is bounded.
>
> Verification techniques vary widely (theorem proving, model checking, abstract interpretation, symbolic execution, etc) and the corresponding tools typically require significant expertise to leverage.
> But as ~~lazy~~ busy developers, we can readily integrate and benefit from already-formally-verified libraries.
> Two contenders for native cryptography are:
>
> 1. `aws-lc-rs` (Amazon)[^AWSLC] - Symbolic execution of source code is used to prove that a program matches a machine-readable specification manually encoded from an algorithm's human-readable specification.
> <br><br/>
> 1. `symcrypt` (Microsoft)[^SymCrypt] - Source is translated to a model for an interactive (meaning semi-manual) theorem prover. Additionally, a combination of fuzzing and model-based testing is used to detect timing side-channels.
>
> Keep in mind that formal verification is not a panacea: specifications can be incomplete and implementations can deviate from models.
> The aforementioned WPA2 4-way handshake was formally verified yet still exploitable!
> Its proof failed to specify when a negotiated key should be installed, implicitly allowing multiple installations and thus nonce reset on next install [^KRACK].

## Supply-chain: Allowlist Crypto Publishers and Ban Duplicates

Programming languages with official package registries are a joy to use: easily finding and integrating 3rd-party libraries means faster delivery speed and greater focus on your problem/business domain.
But all convenience has a cost.
Here:

* **Increased attack surface** - Just one malicious crate, no matter how deep in a massive dependency graph, can compromise the entire *application*. And typo-squatting attacks indiscriminately victimize a percentage of the entire *ecosystem*.

* **Statistical weakening of memory-safety** - Dependency count likely has some correlation to amount of `unsafe` Rust code ([19% of public crates use `unsafe`[^UnsafeState]) and other-language CFFI code, and thus amount of total *unsound* code (realistically some subset of `unsafe`). Any unsound code can trigger memory safety errors at runtime, which often go undetected in production.

* **Software bloat** - Transitive dependencies tend to sprawl in number[^Bloat], causing "simple" apps to explode in objective size and complexity. Larger programs generally mean slower app startup and longer download times. Plus both routine (e.g. API upgrade) and emergency (e.g. vulnerable dependency alert) maintenance burden.

Supply-chain assurance is particularly important for cryptographic dependencies, which likely have an out-sized impact on the security properties of an overall system. Application logic higher up the stack tends to rely on crypto libraries, implicitly or explicitly.

Imagine you've been handed a strict mandate: the two requirements below *must* hold for your *entire* million-plus line monorepo.

1. **Trusted Publishers** - All direct (e.g. non-transitive) cryptographic dependencies must be sourced from a small allowlist of trusted publishers, initially only the `RustCrypto` organization[^RustCrypto].

    * ***Rationale:** Minimize both `RUSTSEC`[^RustSecDB] alert volume and backdoor introduction risk.*

    * ***Scope:** Direct dependencies only. Publishers we explicitly trust can still select their own dependencies.*

1. **No Duplicates** - All direct and indirect cryptographic dependencies must have exactly one version in-tree at any time.

    * ***Rationale:** Minimize both bloat and programmer error (e.g. unclear behavior divergence between API versions).*

    * ***Scope:** All dependencies. Duplicate bloat is likely avoidable - some crate owner should consider updating to latest.*

<br>
<body>
  <div class="diagram-row">
    <div class="diagram-col2">
      <img src="tt_supplychain_1.svg" alt="Before supply-chain policy enforcement">
      <br>
      <figcaption><i><center><b>Fig. 2:</b> No supply-chain policy. Tolerate organic dependency sprawl.</center></i></figcaption>
    </div>
    <div class="diagram-col">
      <img src="tt_supplychain_2.svg" alt="After supply-chain policy enforcement">
      <br>
      <figcaption><i><center><b>Fig. 3:</b> Policy enforced: only trusted publisher, no duplicates. Leaner app overall.</center></i></figcaption>
    </div>
  </div>
</body>

How do you enforce this policy (which nicely compliments our previous `NonceSafeAead` APIs)?
Unfortunately these specific requirements can't be encoded with `cargo deny`[^Cargodeny], a popular and mature dependency graph linter, at the time of this writing (v0.18).
We need to roll some custom kit atop `cargo_metadata`[^Cargometadata]!

Let's start with builder-pattern[^BuilderPattern] boilerplate (our public API):

```rust,ignore
{{#include ../../code_snippets/chp14/tactical_trust/supplychain_policy/src/lib.rs:builder_impl_1}}

    /// ...OMITTED: Rule 2 (Category-specific No Duplicates)...

{{#include ../../code_snippets/chp14/tactical_trust/supplychain_policy/src/lib.rs:builder_impl_2}}
```

To keep the length of this section in check, we'll omit implementation of scaffolding for the 2nd policy requirement (no duplicate cryptographic dependencies).
But the logic is mechanically similar to the first requirement and the complete, runnable &asymp;300 lines of source for both rules is at `code_snippets/chp14/tactical_trust/supplychain_policy`.

Notice that the above builder doesn't encode anything specific to *cryptographic* crates - this interface supports arbitrary categories and publishers.
Before we see what usage looks like in practice, lets dig into enforcement logic for whatever trusted publishers the user specified when initializing `cat_pubs` with a call to `allowed_category_publishers` (the below are private APIs):

```rust,ignore
{{#include ../../code_snippets/chp14/tactical_trust/supplychain_policy/src/lib.rs:policy_impl}}
```

* `fn metadata` does memoized collection of dependency metadata for the entire workspace, with all features enabled. Even if the user specifies 10 requirements for 10 different crate categories, we'll run collection exactly once (recall `Policy` field `cargo_metadata_result` is a `OnceCell`[^OnceCell]).

* `fn get_repo_publisher` parses the owner of a repository from its URL. While this logic will extract the publishing *user* or *organization* for both GitHub and GitLab URLs, be warned: we're not claiming any of the code in this supply-chain half of this section is robust enough for production usage!

    * We can't rely on the authors field[^CargometadataAuthors] of `cargo_metdata`'s `Package` struct, which could be maliciously set to impersonate a publisher. We instead use [presumably valid] URLs as a source of truth for publisher identification. PKI will be a superior long-term solution, more on this later.

* `fn run_allowed_category_publishers` is the bulk of our trusted publishers (requirement 1) logic. We identify direct dependencies of the target project (to which `Policy::new` takes a `Cargo.toml` path) and iterate that list to look for any crate which belongs to a user-specified category but isn't sourced from a user-allowed publisher for that category.

    * Crate category labels are optional, but we could extend the builder to support "allowed publisher for any or missing category" - ensuring unexpected publishers don't slip in. Our policy evaluation logic also doesn't validate user-input category names, a typo will cause checks to pass! Adding validation would be straightforward since categories are fixed[^CrateCats].

So how do we roll out enforcement of our sophisticated policy requirements (category-specific trusted publishers and duplicate elimination)?
The heavy-handed option is leveraging `build.rs` (Rust build scripts[^RustBuildScripts]):

```rust,ignore
{{#include ../../code_snippets/chp14/tactical_trust/nonce_typing/build.rs:builder_demo}}
```

Now failing builds for supply-chain policy violations probably isn't the best way to make friends with other development teams, even in a smaller organization, unless there's a strong regulatory and/or business need to do so.
Fortunately the above `Policy` builder can easily be wrapped in a CLI tool and deployed in *blocking* or *non-blocking* CI pipelines, on a workspace-specific basis.
Non-blocking failures can be centrally tracked and automatically triaged.

Our above proof-of-concept didn't accommodate exceptions (e.g. "allow this specific named duplicate, still enforce for remainder of category"), but you could quickly extend it to read individual crate/publisher names from a [version controlled and `CODEOWNERS` protected] config file.
Supporting legitimate exceptions, with documented rationale, is realistic - "perfect is the enemy of good".

> ***What are my other options for supply-chain security in Rust?***
>
> The landscape of Rust's supply-chain security tooling is, fortunately, evolving.
> Sample projects to be aware of:
>
> * **Signature-based vulnerability alerting:** `cargo audit`[^CargoAudit], a free tool to scan your dependency tree for known-vulnerable[^RustSecDB] crates, is a must-have for production CI. Although a lack of "reachability analysis" (call-graph traversal to determine if your code directly or indirectly calls a vulnerable function) does mean false positives.
> <br><br/>
> * **Heuristic-based malware detection:** The Linux Foundation has funded development[^CapslockNews] of a Rust counterpart to Go's `capslock`[^Capslock] tool. Among other usecases, `capslock` enumerates *capabilities*[^CapslockCaps] (file I/O, network connectivity, command execution, etc) for a given dependency and alerts if they suddenly change in a new version.
> <br><br/>
> * **Trusted publishers:** Future PKI initiatives[^RustPKI] may allow cryptographic identification of publishers, a big improvement over our above URL parsing. A related RFC[^RustTrustedPubs] outlines support for publishing crates from trusted infrastructure, following the footsteps of PyPI[^PyTrustedPubs]. Note PKI also means better response capability, although a real-world attack may have already succeeded by the time a build machine pulls a Certificate Revocation List (CRL).
>
> While Rust's intentionally minimal `std` library is boon for embedded development, it does encourage over-reliance on 3rd-party crates for routine tasks.
> For contrast: Go's standard library offers FIPS 140-3 compliant cryptography[^GoFIPS] with the flip of a build flag and backported a secure RNG[^GoCSPRNG] to existing programs with only a Go toolchain bump!

## Takeaway

"Trust is earned in drops and lost in buckets".
That's probably a maxim, but it feels especially true in the context of commercial software - a global competition in which any winner, perhaps outside of a few monopolists, can be dethroned at any time.

Now the technical mechanism for trust is cryptography.
Most useful cryptography is implemented and executing, whether on a tiny microcontroller or a beefy server, in the form of code.
And code is notoriously difficult to get right, especially when you're shipping a lot of it.

Software quality is as challenging to replicate reliably as it is to measure actionably, if not more so.
Our best hope is automating repeatability.
When the quality criteria is security, automation is one goal of a platform security engineering function.
Which needs to keep pace with the broader engineering organization at minimum, and ideally should accelerate all feature teams.

This first section explored bite-sized solutions to platform cryptography problems at the API (nonce reuse) and supply-chain (dependency policy) levels.
The intent is automating guardrails for *human* error, but nowadays *LLM* auto-complete increases vulnerability rate[^LLM1] [^LLM2].
The good news is that the above techniques should mitigate risks from both sources.
Compile-time checks don't care how the code was generated.

Our second and final section will have a narrower but deeper scope.
We'll explore a classic topic in trust: *information disclosure* vulnerabilities.
Part 2 grapples with technical concepts at greater length and on the cutting edge.
You're going to want a coffee for this one.

But it'll still be good fun.
Trust me.

[^MatterHeartbleed]: [*The Matter of Heartbleed*](https://dl.acm.org/doi/pdf/10.1145/2663716.2663755). Zakir Durumeric, Frank Li, James Kasten, Nicolas Weaver, Johanna Amann, Jethro Beekman, Mathias Payer, David Adrian, Michael Bailey, Vern Paxson, J. Alex Halderman (2014).

[^Ivory]: [*Ivory Tower*](https://en.wikipedia.org/wiki/Ivory_tower). Wikipedia (Accessed 2025).

[^GoodNews]: [*Good new, everyone!*](https://futurama.fandom.com/wiki/Good_news,_everyone!). Futurama Wiki (Accessed 2025).

[^RWCBook]: [***[PERSONAL FAVORITE]** Real-world Cryptography* by Dave Wong](https://amzn.to/43ov045). David Wong (2021).

[^SoatokBlog]: [*Soatok's Cryptography Blog*](https://soatok.blog/category/cryptography/). Soatok (Accessed 2025).

[^RWCConf]: [*Real World Crypto Symposium*](https://rwc.iacr.org/). IACR (Accessed 2025).

[^KRACK]: [*Key Reinstallation Attacks: Forcing Nonce Reuse in WPA2*](https://dl.acm.org/doi/pdf/10.1145/2663716.2663755). Mathy Vanhoef, Frank Piessens (2017).

[^PS3Signing]: [*PS3 Epic Fail*](https://www.youtube.com/watch?v=DUGGJpn2_zY). FailOverflow (2010).

[^TraitAead]: [*Trait `aead::Aead`*](https://docs.rs/aead/0.5.2/aead/trait.Aead.html). RustCrypto organization (Accessed 2025).

[^APIAead]: [*API `aead::Aead::encrypt`*](https://docs.rs/aead/0.5.2/aead/trait.Aead.html#tymethod.encrypt). RustCrypto organization (Accessed 2025).

[^TraitCryptoRng]: [*Trait `CryptoRng`*](https://docs.rs/rand_core/0.9.3/rand_core/trait.CryptoRng.html). RustCrypto organization (Accessed 2025).

[^BitFail]: [*Biased Nonce Sense: Lattice Attacks against Weak ECDSA Signatures in Cryptocurrencies*](https://eprint.iacr.org/2019/023.pdf). Joachim Breitner, Nadia Heninger (2019).

[^AesSIVRFC]: [*AES-GCM-SIV: Nonce Misuse-Resistant Authenticated Encryption*](https://datatracker.ietf.org/doc/html/rfc8452). S. Gueron, A. Langley,  Y. Lindell (2019).

[^AdamAEAD]: [*AEADs: getting better at symmetric cryptography*](https://www.imperialviolet.org/2015/05/16/aeads.html). Adam Langley (2015).

[^XChaCha]: [*XChaCha: eXtended-nonce ChaCha and AEAD_XChaCha20_Poly1305*](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha-03#section-2.1). S. Arciszewski (2020).

[^AWSLC]: [*aws-lc-rs*](https://crates.io/crates/aws-lc-rs). Amazon (Accessed 2025).

[^SymCrypt]: [*symcrypt*](https://crates.io/crates/symcrypt). Microsoft (Accessed 2025).

[^UnsafeState]: [*Unsafe Rust in the Wild: Notes on the Current State of Unsafe Rust*](https://rustfoundation.org/media/unsafe-rust-in-the-wild-notes-on-the-current-state-of-unsafe-rust/). Rust Foundation (2024).

[^Bloat]: [*Why Bloat Is Still Software's Biggest Vulnerability: A 2024 plea for lean software*](https://spectrum.ieee.org/lean-software-development). Bert Hubert (2024).

[^RustCrypto]: [*RustCryto*](https://github.com/rustcrypto). RustCrypto organization (Accessed 2025).

[^CargoAudit]: [*`cargo_audit`*](https://crates.io/crates/cargo-audit). Alex Gaynor, Tony Arcieri, Sergey Davidoff (Accessed 2025).

[^OnceCell]: [*`OnceCell`*](https://doc.rust-lang.org/beta/std/cell/struct.OnceCell.html). The Rust Project (Accessed 2025).

[^CrateCats]: [*Categories*](https://crates.io/categories). crates.io (Accessed 2025).

[^RustBuildScripts]: [*Rust Build Scripts*](https://doc.rust-lang.org/cargo/reference/build-script-examples.html). The Cargo Team (Accessed 2025).

[^RustSecDB]: [*The Rust Security Advisory Database*](https://rustsec.org/advisories/). Rust Secure Code Working Group (Accessed 2025).

[^Cargodeny]: [*`cargo_deny`*](https://crates.io/crates/cargo-deny). Embark Studios (Accessed 2025).

[^Cargometadata]: [*`cargo_metadata`*](https://crates.io/crates/cargo_metadata). Oliver Schneider (Accessed 2025).

[^BuilderPattern]: [*Rust Design Patterns: Builder*](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html). Rust Unofficial (Accessed 2025).

[^CargometadataAuthors]: [*`cargo_metadata::Package`*](https://docs.rs/cargo_metadata/0.20.0/cargo_metadata/struct.Package.html#structfield.authors). Oliver Schneider (Accessed 2025).

[^CapslockNews]: [*CRustabilities: Capabilities, Rust and Capslock*](https://alpha-omega.dev/blog/crustabilities-capabilities-rust-and-capslock/). Alpha-Omega (Accessed 2025).

[^Capslock]: [*capslock*](https://github.com/google/capslock). Google (Accessed 2025).

[^CapslockCaps]: [*Capabilities*](https://github.com/google/capslock/blob/main/docs/capabilities.md#capabilities). Google (Accessed 2025).

[^RustPKI]: [*Improving Supply Chain Security for Rust Through Artifact Signing*](https://rustfoundation.org/media/improving-supply-chain-security-for-rust-through-artifact-signing/). Adam Harvey (2023).

[^RustTrustedPubs]: [*Security Improvements for CI Publishing to crates.io*](https://rust-lang.github.io/rfcs/3691-trusted-publishing-cratesio.html). The Rust Project (Accessed 2025).

[^PyTrustedPubs]: [*Introducing 'Trusted Publishers'*](https://blog.pypi.org/posts/2023-04-20-introducing-trusted-publishers/). Dustin Ingram (2023).

[^GoFIPS]: [*FIPS 140-3 Compliance*](https://go.dev/doc/security/fips140). Go Project (Accessed 2025).

[^GoCSPRNG]: [*Secure Randomness in Go 1.22*](https://go.dev/blog/chacha8rand). Russ Cox, Filippo Valsorda (2024).

[^LLM1]: [*Do users write more insecure code with AI assistants?*](https://arxiv.org/pdf/2211.03622). Neil Perry, Megha Srivastava, Deepak Kumar, Dan Boneh (2023).

[^LLM2]: [*Asleep at the Keyboard? Assessing the Security of GitHub Copilot's Code Contributions*](https://dl.acm.org/doi/pdf/10.1145/3610721). Hammond Pearce, Baleegh Ahmad, Benjamin Tan, Brendan Dolan-Gavitt, Ramesh Karri (2025).