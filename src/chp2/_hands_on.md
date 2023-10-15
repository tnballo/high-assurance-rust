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


# Hands-on Challenge: Extend the CLI Encryption Tool

The `rcli` tool we wrote in this chapter is rather basic.
That's by design, it was a short showcase of Rust's features and tooling.

The goal of our first challenge is to extend this CLI encryption tool.
Some readers may prefer to tackle this challenge after finishing the next chapter, which introduces the Rust language in a more structured fashion.
Others may be itching to write code right now, and willing to pick up more Rust as they go.

Below are several suggestions for extending the tool.
You may choose one or more.
Feel free to implement your own ideas as well!

## Core Cryptography

* Switch the encryption algorithm from our RC4 implementation to a modern AEAD cipher of your choosing. Making a choice will require some research into the pros and cons of various ciphers.

    * The RustCrypto organization maintains several AEAD algorithm implementations[^AEADList], but you may find other mature libraries suitable.

    * The threat model for a hardware product may include an attacker with 24/7 physical access to a device. Can you find an algorithm and implementation that makes guarantees about timing and power side-channel resistance?[^SideChannel]

## CLI UX

* Add the ability for a user to create a new encrypted file instead of overwriting an existing file. If the user opts to overwrite an existing file, present a color-coded warning (you will want to choose a 3rd party library for coloring terminal output).

* Add the ability to recursively encrypt every file in a directory (be very careful when testing this, you'll likely want to create a new directory with dummy files!).

* Instead of printing `Processed {file_name}` to the console, update the tool to print either `Encrypted {file_name}` or `Decrypted {file_name}` (hint: is there a heuristic you can test to identify encrypted byte streams?).

* Support encryption of files too large to read into memory at once, via buffering.

## CLI Integration Testing

* Add an integration test that runs your CLI binary, providing both command line arguments and temporary files to encrypt or decrypt. You'll likely want to use one or more 3rd party libraries to set up your test harness.

* Negative tests, meaning those that check for graceful handling of invalid inputs, are a critical part of security testing. Ensure that your integration harness explicitly checks such cases.

<br>

---

[^AEADList]: [*RustCrypto: Authenticated Encryption with Associated Data (AEAD) Algorithms*](https://github.com/RustCrypto/AEADs). RustCrypto organization (Accessed 2022).

[^SideChannel]: Side-channel attacks leverage information *leaked* by a physical system (timing, power consumption, electromagnetic emissions, acoustic emissions, etc) to compromise security. In the context of cryptography, this often means extracting secret key material via an indirect means (no software bugs exploited!). Some cryptographic algorithms are designed with this threat model in mind. Their operations are carefully constructed to reduce any externally observable variance, making side-channel attacks significantly more difficult or outright impractical.
