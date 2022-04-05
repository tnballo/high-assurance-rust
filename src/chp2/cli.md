# DIY CLI Encryption Tool

Unit tests demonstrated that our library is correct (at least the first, non-backdoored version).
We'll now use it to build a Command Line Interface (CLI) encryption utility!

Our tool will accept two arguments, a filename and a hexadecimal encryption key, and proceed to encrypt or decrypt the file on disk.

Given the simplicity of those requirements, we could easily build our CLI tool using only Rust's standard library:

* The `std::env` module[^StdEnv] provides OS-agnostic argument parsing, among other facilities.

* The `std::fs` module[^StdFs] provides OS-agnostic filesystem input/output (e.g. reading/writing files).

Using strictly `std` would be a fine way to proceed.
If you're a purist, or you'd like to work through a problem without guidance, you can try adapting the below program to use only the standard library.
But we're going to try out Rust's 3rd-party library ecosystem.

Real-world command line applications make use of various 3rd party libraries.
Rust's CLI ecosystem offers plug-n-play (easy to build and link) argument parsing[^Clap], text coloring[^Owo], integration testing[^AssertCmd], Text-based User Interfaces (TUIs)[^Tui], etc.
A vast sea of community-maintained libraries make building and maintaining CLI apps a joy.

So let's go ahead and try out a popular library: we'll use the `clap` crate[^Clap] for argument parsing.
`clap` uses Rust's macro system to enable declarative argument parsing logic, as we'll soon see.

> **Leveraging the growing Rust ecosystem**
>
> One of Rust's killer features is `cargo`, its official package manager and build system.
> We already used `cargo` to compile and test our RC4 library.
> But `cargo`'s true value is how easy it makes leveraging libraries from the broader Rust ecosystem.
>
> In modern development, the practicality of a programming language is a function of both core language features *and* the availability of domain-specific abstractions developed/maintained by someone else - in the form of libraries (called "crates" in Rust).
>
> Software engineers need to ship quality code quickly.
> That means using existing code, when appropriate, to bootstrap our products.
> At the time of this writing, [crates.io](https://crates.io/), the official centralized repository for Rust libraries, hosts over 75,000 crates.
>
> Of course, not every crate is well-maintained, production quality, or secure.
> But we have many options from which to choose.
> And that number will continue to grow as the Rust ecosystem matures.

`cargo` will take care of downloading and building the latest version of `clap`.
All we need to do is add one line to `crypto_tool/rcli/Cargo.toml`:

```toml,ignore
[package]
name = "rcli"
version = "0.1.0"
edition = "2018"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
rc4 = { path = "../rc4" }
clap = { version = "^3", features = ["derive"] }
```

* `features = ["derive"]` enables an optional feature of the `clap` library: support for derive macros. It'll save us some boilerplate.

* `version = "^3"` tells `cargo` that our tool uses the latest `clap` version `>= 3.0.0` but `< 4.0.0`. We can assume API stability for `3.x.x` versions because Rust crates follow *semantic versioning* (semver, e.g. `MAJOR.MINOR.PATCH`)[^SemVer].

Notice that, unlike the `rc4` dependency, we don't provide a local `path` for `clap`.
`cargo` will download the source code from [crates.io](https://crates.io/) the first time we build or run our `rcli` project.

> **Can 3rd-party code be trusted?**
>
> Most software leverages 3rd-party code.
> But each external component we pull in risks introducing bugs and/or vulnerabilities into our system.
> For this reason, mature organizations have processes in place to vet dependencies and suppliers.
>
> In some contexts, it's safer to audit the source of a 3rd-party dependency and only use the pre-audited version for all our builds.
> The setup of internal repositories and build systems is specific to individual companies and teams.

## Parsing Arguments with `clap`

One of `clap`'s most convenient features is the ability to *annotate* the fields of a structure.
Each annotation, which is mechanically a Rust macro, generates code to take care of displaying and parsing arguments.

* When a user invokes our CLI tool, we get a single structure with their requested settings/operations (`Args` in the below).

* Instead of worrying about the intricacies of argument parsing, we can focus effort on our "business logic": actioning the fields of the structure to perform the requested tasks.

Let's see how this plays out.
Add the below to `crypto_tool/rcli/src/main.rs`:

```rust,ignore
use clap::Parser;

{{#include ../../code_snippets/chp2/crypto_tool/rcli/src/main.rs:clap_args}}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
```

* The `Args` has is a `struct` with 2 fields:

    * `file`, a string that will contain the path/name of the file to be en/decrypted.

    * `key`, a dynamic array of individual, space-separated strings. Each string will be a hexadecimal byte of the key.

* Highlights from `clap`'s field annotations (macros of the form `#[something(...)]`):

    * `short` - generate a short argument name (e.g. `-f` for `file`).

    * `long` - generate a long argument name (e.g. `--file` for `file`).

    * `required = true` - argument *must* be provided to run the tool.

    * `min_values = 5` - enforces RC4's minimum 5 byte (40 bit) key length.

    * `max_values = 256` - enforces RC4's maximum 256 byte (2048 bit) key length.

Our two-line `main` function currently just prints out the `Args` structure collected from user input.
The format specifier `{:?}` allows us to use a default formatter, which `Args` supports because it derives the `Debug` trait.
We'll talk about traits in the next chapter.

> **How would I learn about the annotations `clap` supports?**
>
> Rust has a built-in documentation system, `rustdoc`[^RustDoc].
> All public crates provided generated documentation, but its completeness varies by project.
> You can view `clap`'s docs at [https://docs.rs/clap](https://docs.rs/clap).

To run our work-in-progress CLI tool, as is, we can use the command `cargo run -- --help`:

```ignore
rcli
RC4 file en/decryption

USAGE:
    rcli --file <FILE_NAME> --key <HEX_BYTE(S)>...

OPTIONS:
    -f, --file <FILE_NAME>        Name of file to en/decrypt
    -h, --help                    Print help information
    -k, --key <HEX_BYTE(S)>...    En/Decryption key (hexadecimal bytes)
```

The `--` delimiter tells `cargo` to pass the remainder of the input to our CLI tool.
In this case we're only passing the flag `--help`.
Conveniently, `clap` implemented this flag for us.
Since it's a common convention for CLI tools.
Note how the comments (lines starting with `///`) for each field of the `Args` struct are used as descriptions in the help output.

But `--help` doesn't run our `main` function.
Let's try `cargo run -- --file test.txt --key 0x01 0x02 0x03 0x04 0x05` to simulate regular usage of our tool.
With the minimum five byte key length.
Our `main` will print:

```ignore
Args { file: "test.txt", key: ["0x01", "0x02", "0x03", "0x04", "0x05"] }
```

We have working argument parsing!

## Implementing the File En/Decryption Logic

All that's left is the "glue" between our RC4 library and our new CLI front-end.
Let's update `main` to (note the additional imports at the top):

```rust,ignore
{{#include ../../code_snippets/chp2/crypto_tool/rcli/src/main.rs:full_imports}}

// `Args` struct omitted, unchanged...

{{#include ../../code_snippets/chp2/crypto_tool/rcli/src/main.rs:cli_main}}
```

We've added a return type for `main`: `std::io::Result<()>`[^IORes].
We'll cover Rust's `Result` type in the next chapter.
The important part here is that all *fallible* file I/O operations within `main`'s body end with the `?` operator.
This tells the function to "short-circuit" if an operation fails and immediately return the error `Result`.

For example, say someone runs our program and provides a path to a non-existent file:

```ignore
cargo run -- --file non_existant_file.txt --key 0x01 0x02 0x03 0x04 0x05
```

The line `let mut file = File::open(args.file)?;` will fail and terminate the program with the following error:

```ignore
Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

In a production-grade tool, we could handle errors gracefully, log them, or wrap them with more user-friendly output.
If no errors are hit, we simply return an empty value (`()`, Rust's *unit type*[^Unit]) wrapped in `Ok` to indicate success.

Our new `main` function has a few more pieces worth explaining:

* **No buffering:** We read the whole file into memory at once, into the byte vector `contents`. Supporting large files could be made more efficient with a technique called *buffering*, where we only read/encrypt a small chunk at a time. This example aims for simplicity instead.

* **Optional byte prefix:** The key conversion logic uses Rust's functional-style iterators. We'll discuss iterators at length later. Note that `s.trim_start_matches("0x")` allows our user to *optionally* add the prefix `0x` to each byte. Meaning `--key 01 02 03 04 05` would have been valid and equivalent input.

* **Input validation:** Whenever you write code that parses input you don't control, that input is *untrusted*. It may be *attacker-controlled*. Validate it ASAP - before passing it along to any other component of your program or system. Our `main` uses a three step-validation:

    1. **Valid key length:** Instead of assuming our encryption library checks key length (which our RC4 implementation does!), we used annotations for the `key` field in `Arg` (e.g. `min_values = 5` and `max_values = 256`). Try `cargo run -- -f anything.txt -k 01` to see the error.

    2. **Valid hex key bytes:** `.expect("Invalid key hex byte!")` determines the error message thrown if the program receives an invalid string representation of a base-16 byte in the key input (e.g. `"0xfg"`).

    3. **Necessary file permissions:** `std::fs` uses OS facilities to ensure the user has permissions to read and write the file provided. If they don't, our program will throw an error - similar to the `Error: Os { code: 2, kind: NotFound...` case we saw earlier.

> **What should we do if validation fails?**
>
> That depends on operational context.
> By terminating our CLI tool with an actionable error message as early as possible, we're practicing **offensive programming**:
>
> * We consider failing early to be an effective first line of defense. We believe there's benefit to never tolerating certain errors.
>
> If our file encryption logic had been part of a networked service or protocol, we'd likely prioritize **availability** - keeping the system online and reachable.
> **Defensive programming** would be more appropriate:
>
> * We'd recover from failures with minimal disruption to the end-user.
>
> That could mean returning an error (via status code or protocol message) and immediately reverting to listening for new file encryption requests.
> Instead of terminating.

## Using the Tool

First, let's install our tool so that we can use it like any other command-line utility.
From the `crypto_tool/rcli` directory, run:

```ignore
cargo install --path .
```

You should now be able to run `rcli --help`.
If you want to know where the actual compiled binary is located, run `which rcli`.

To try out the tool, create a text file with a secret message:

```ignore
echo "This is a secret, don't tell anyone!" > secret.txt
```

We can use `cat` to verify the contents:

```ignore
$ cat secret.txt
This is a secret, don't tell anyone!
```

The contents will no longer be printable after we encrypt.
So let's view them with `xxd` now:

```ignore
$ xxd -g 1 secret.txt
00000000: 54 68 69 73 20 69 73 20 61 20 73 65 63 72 65 74  This is a secret
00000010: 2c 20 64 6f 6e 27 74 20 74 65 6c 6c 20 61 6e 79  , don't tell any
00000020: 6f 6e 65 21 0a                                   one!.
```

`xxd` displayed three columns:

1) **Left:** the hexadecimal offset into the file.
2) **Middle:** the raw contents of the file as a sequence of hexadecimal bytes.
3) **Right:** the ASCII[^ASCII] decoding of the raw bytes (our secret message).

The `-g 1` flag makes each byte stand alone in that middle column.

Let's encrypt the file:

```ignore
rcli -f secret.txt -k 01 02 03 04 05
```

You should see the output `Processed secret.txt`.
If we run `xxd -g 1 secret.txt` again:

```ignore
00000000: e6 51 0a 76 d0 54 b3 07 ad e3 21 2f 69 63 7d dc  .Q.v.T....!/ic}.
00000010: 45 a2 f0 20 76 db f6 f5 fd a1 6f c8 5a 6c 67 60  E.. v.....o.Zlg`
00000020: d9 e1 1d e3 87                                   .....
```

The bytes have changed because the file has been encrypted.
Our rightmost column doesn't look like a sensible ASCII string anymore.
Verify that you can decrypt the file and retrieve the original message by running `rcli` again.

> **Could our tool print "encrypting" or "decrypting" when processing the file?**
>
> As you've seen, encryption and decryption are really the same operation.
> We're XOR-ing with the keystream either way.
> In order to print a user-friendly message indicating if we've just hidden data or revealed it, we need to know the starting state of the file.
>
> Turns out that's not a trivial task!
> You have to bridge the "semantic gap" to determine if an arbitrary sequence of bytes is or isn't an encrypted file.
> That's part of this chapter's challenge.

## Major Checkpoint

We wrote a portable, memory-safe RC4 library and validated it against official test vectors.
It can be used anywhere, even bare metal embedded systems.

Leveraging the `clap` crate and Rust's standard library, we then built a simple RC4 CLI tool.
Our tool can be compiled for any major OS.

All in only 172 lines of code.
Including all tests and implementing the cryptography *from scratch*.
And that code is natively compiled - `rcli` is blazing fast.
Not bad for your first Rust program.

Take a second to soak it in.
You've already come quite far.
When you're ready, we can close out this chapter with one last topic: operational assurance.

---

[^StdEnv]: [*Module `std::env`*](https://doc.rust-lang.org/std/env/index.html). The Rust Team (Accessed 2022).

[^StdFs]: [*Module `std::fs`*](https://doc.rust-lang.org/std/fs/index.html). The Rust Team (Accessed 2022).

[^Clap]: [*Crate `clap`*](https://docs.rs/clap/3.0.7/clap/). `clap-rs` Project (Accessed 2022).

[^Owo]: [*Crate `owo-colors`*](https://docs.rs/owo-colors/3.2.0/owo_colors/). jam1garner (Accessed 2022).

[^AssertCmd]: [*Crate `assert_cmd`*](https://docs.rs/assert_cmd/latest/assert_cmd/). Ed Page (Accessed 2022).

[^Tui]: [*Crate `tui`*](https://docs.rs/tui/0.16.0/tui/). Florian Dehau (Accessed 2022).

[^SemVer]: [*Semantic Versioning 2.0.0*](https://semver.org/). Tom Preston-Werner (Accessed 2022).

[^RustDoc]: [*What is rustdoc?*](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html).

[^IORes]: [*Type Definition `std::io::Result`*](https://doc.rust-lang.org/std/io/type.Result.html). The Rust Team (Accessed 2022).

[^Unit]: [*Primitive Type unit*](https://doc.rust-lang.org/std/primitive.unit.html). The Rust Team (Accessed 2022).

[^ASCII]: [*ASCII*](https://en.wikipedia.org/wiki/ASCII). Wikipedia (Accessed 2022).


