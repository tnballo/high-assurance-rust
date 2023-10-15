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


# Dynamic Assurance (3 of 3)

You may have noticed a little module (scope of the `mod` keyword) hanging out at the bottom of `crypto_tool/rc4/src/lib.rs`:

```rust,noplaypen
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
```

This is unit test boilerplate, it was filled in when we ran `cargo new crypto_tool/rc4 --lib` earlier.
We're going to replace it with our own unit test now.

The first test we'll write is essentially a "sanity check".
At a bare minimum, our library should be able to transform plaintext into something different (presumably an encrypted form) and back into the original.
That's what this test checks for:

```rust,ignore
#[cfg(test)]
mod tests {
    use super::Rc4;

{{#include ../../code_snippets/chp2/crypto_tool/rc4/src/lib.rs:sanity_check_static_api}}
}
```

We print our initial plaintext, use `apply_keystream_static` to encrypt it and print the result, then similarly decrypt it and print the result.

* `key` is a random, 16-byte key we've chosen arbitrarily for testing purposes.

* `msg` is the raw bytes for the ASCII[^ASCII] string "Hello World!".

* `String::from_utf8(msg.to_vec()).unwrap()` converts the raw bytes into a printable string.
    * This is a fallible operation (we could have provided non-printable bytes as input!) so an "operation result" has to be "unwrapped" (`.unwrap()` is like an `assert!` here). We'll discuss `Result` and error handling in Chapter 3.

* `#[rustfmt::skip]` tells our code formatter (invoked via `cargo fmt`) not to change the indentation of the variable it appears above. It's not pertinent to this test, but you may have been curious what it's for. Rust supports configurable code formatting and linting to make style consistent for large, multi-developer codebases.

You can run this test with `cargo test` command, from the `crypto_tool/rc4` directory.
By default, `cargo test` prints only test results, not their console output, unless a test fails.

To see our `println!` statements, we need to use `cargo test -- --show-output`.
Then output will include the following:

```ignore
---- tests::sanity_check_static_api stdout ----
Plaintext (initial): Hello World!
Ciphertext: [d0, 1c, 95, d4, 40, c7, 3c, 53, 8a, 22, d9, a1]
Plaintext (decrypted): Hello World!
```

Our simple dynamic test demonstrates we have a runnable program capable of scrambling and unscrambling a message!

Note how we don't print the ciphertext as a string, since it contains non-printable characters.
We display the raw hexadecimal bytes instead.
You can take a second to write a similar test for the chunk API, `apply_keystream`, now.

1st-party unit test support, via `cargo test`, is a major strength of Rust relative to C and C++.
We didn't need to learn/configure/build/import any 3rd party test frameworks to get a modern development experience.

While our methodology is powerful, our actual test was not.
This "sanity check" doesn't actually prove that we implemented RC4 correctly - just that our code can transform the data and reverse the change.
There's a risk that the generated cipher text is incorrect for the given key, potentially in some way that makes it "crackable" - maybe an attacker can leverage some flaw and extract plaintext without knowledge of the key.

To ensure that's not the case, we need to validate our implementation dynamically.
Create a runnable test against ground truth.
For cryptographic ciphers, this often means comparing against official "test vectors" (known-good input-output pairs).

## Dynamic Validation

RC4 was, in the not-so-distant-past, a critical part of internet security.
Almost every TLS connection on the internet once used, or could choose to use, the algorithm.
Thus, a leading internet standards body - the Internet Engineering Task Force (IETF) - released official test vectors[^TestVec] to help protocol implementers validate their RC4 libraries.

We're going to leverage those official vectors now!
*Justifiable confidence* is the hallmark of *high assurance* programming.

The IETF document[^TestVec] contains over a dozen tables of test vector data.
Here's the first one:

```ignore
Key length: 40 bits.
key: 0x0102030405

DEC    0 HEX    0:  b2 39 63 05  f0 3d c0 27   cc c3 52 4a  0a 11 18 a8
DEC   16 HEX   10:  69 82 94 4f  18 fc 82 d5   89 c4 03 a4  7a 0d 09 19
DEC  240 HEX   f0:  28 cb 11 32  c9 6c e2 86   42 1d ca ad  b8 b6 9e ae
DEC  256 HEX  100:  1c fc f6 2b  03 ed db 64   1d 77 df cf  7f 8d 8c 93
DEC  496 HEX  1f0:  42 b7 d0 cd  d9 18 a8 a3   3d d5 17 81  c8 1f 40 41
DEC  512 HEX  200:  64 59 84 44  32 a7 da 92   3c fb 3e b4  98 06 61 f6
DEC  752 HEX  2f0:  ec 10 32 7b  de 2b ee fd   18 f9 27 76  80 45 7e 22
DEC  768 HEX  300:  eb 62 63 8d  4f 0b a1 fe   9f ca 20 e0  5b f8 ff 2b
DEC 1008 HEX  3f0:  45 12 90 48  e6 a0 ed 0b   56 b4 90 33  8f 07 8d a5
DEC 1024 HEX  400:  30 ab bc c7  c2 0b 01 60   9f 23 ee 2d  5f 6b b7 df
DEC 1520 HEX  5f0:  32 94 f7 44  d8 f9 79 05   07 e7 0f 62  e5 bb ce ea
DEC 1536 HEX  600:  d8 72 9d b4  18 82 25 9b   ee 4f 82 53  25 f5 a1 30
DEC 2032 HEX  7f0:  1e b1 4a 0c  13 b3 bf 47   fa 2a 0b a9  3a d4 5b 8b
DEC 2048 HEX  800:  cc 58 2f 8b  a9 f2 65 e2   b1 be 91 12  e9 75 d2 d7
DEC 3056 HEX  bf0:  f2 e3 0f 9b  d1 02 ec bf   75 aa ad e9  bc 35 c4 3c
DEC 3072 HEX  c00:  ec 0e 11 c4  79 dc 32 9d   c8 da 79 68  fe 96 56 81
DEC 4080 HEX  ff0:  06 83 26 a2  11 84 16 d2   1f 9d 04 b2  cd 1c a0 50
DEC 4096 HEX 1000:  ff 25 b5 89  95 99 67 07   e5 1f bd f0  8b 34 d8 75
```

We're given a key (line 2) and 18 samples from the keystream a valid RC4 implementation should produce (the subsequent rows).
Each sample is 16 bytes long and preceded by its offset into the keystream (given in both decimal and hex).

Translating every sample from every table into a test suite would be important for a real library, but tedious for our example.
So we'll use just the first 4 rows of the table above:

```rust,ignore
#[cfg(test)]
mod tests {
    use super::Rc4;

    // ..sanity_check_static_api() omitted..

{{#include ../../code_snippets/chp2/crypto_tool/rc4/src/lib.rs:ietf}}
}
```

* `out_buf` is an array for storing the first 272 bytes of the keystream (just enough to slice out the first four samples for comparison). It starts initialized to all zeros. Instead of initializing it in a loop, we use the shorthand `[0x0; 272]`.

    * Any byte XORed with `0x00` is itself. So encrypting a zero-buffer means we're just extracting our implementation's keystream. In any secure cipher, this keystream should be indistinguishable from a random sequence of bytes. For RC4, the values should match the official vectors.

* Each `assert_eq!` checks a slice of the keystream (a subset of `out_buf`) against the corresponding test vector (`test_stream_*`).

    * Notice we use slicing notation to grab 16-byte chunks at an offset corresponding to the document's table (e.g. `out_buf[240..256]` means bytes in the range `[240, 256)` of our `272`).

If you run `cargo test` from the `crypto_tool/rc4` directory, you should now see both unit tests pass:

```ignore
running 2 tests
test tests::ietf_40_bit_key_first_4_vectors ... ok
test tests::sanity_check_static_api ... ok
```

## Takeaway

You've now built your first piece of high assurance software (sans the RC4 algorithm itself).
Your RC4 library is:

* Fully memory-safe, hence `#![forbid(unsafe_code)]`
* Stand-alone and capable for running almost anywhere, hence `#![no_std]`
* Functionally validated, using official IETF test vectors

Before we get to the fun and tangible part - writing a command line tool that uses this library to encrypt local files - we need to take a step back and understand the limitations of all the static and dynamic assurance topics discussed so far.

> **Is Rust a good choice for cryptographic libraries?**
>
> A study of C and C++ cryptographic libraries found that only 27.2% of reported vulnerabilities were caused by flaws related to the cryptography itself, but 37.2% were memory safety issues[^CryptoStudy].
>
> Because both performance and security are core requirements, cryptography is a prime use case for Rust (pun intended).
> The language has a thriving cryptographic ecosystem.
> `rustls`[^RusTLS], a pure-Rust TLS library, is one notable project.
> In 2019, it outperformed OpenSSL by significant margins[^FastRust].

---

[^ASCII]: [*ASCII*](https://en.wikipedia.org/wiki/ASCII). Wikipedia (Accessed 2022).

[^TestVec]: [*Test Vectors for the Stream Cipher RC4*](https://datatracker.ietf.org/doc/html/rfc6229). Internet Engineering Task Force (2011).

[^CryptoStudy]: [*You Really Shouldn’t Roll Your Own Crypto: An Empirical Study of Vulnerabilities in Cryptographic Libraries*](https://arxiv.org/pdf/2107.04940.pdf). Jenny Blessing, Michael A. Specter, Daniel J. Weitzner (2021). Please note that, at the time of this writing, this paper has not yet been accepted to a peer-reviewed conference.

[^RusTLS]: [*`rustls`*](https://github.com/rustls/rustls). rustls Contributors (Accessed 2022).

[^FastRust]: [*A Rust-based TLS library outperformed OpenSSL in almost every category*](https://www.zdnet.com/article/a-rust-based-tls-library-outperformed-openssl-in-almost-every-category/). Catalin Cimpanu (2019).
