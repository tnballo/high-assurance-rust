# Operational Assurance (2 of 2)

Time for a tiny bit of client-side operational assurance.
We're going to package this chapter's code so that it "just works" in the field.

We want our `rcli` tool to run on nearly any Linux system, instantly.
Just by copying a single executable file.
No setup, no pulling in libraries with an OS-specific package manager.
We want it to work for every user, every time.

> **Is this section relevant to red teams?**
>
> Potentially.
> Operational assurance can be thought of as an abstract game played by defenders and attackers.
> Likewise, native executables can serve different agendas:
>
> * **Defense:** Performant, reliable tools for a range of hosts you manage (e.g. "assets").
>
> * **Offense:** Portable programs amenable to obfuscation[^Hellscape]. For hosts owned by your victims (e.g. "targets").

## Building a Free-standing Binary

Static binaries are a tried-and-true way to bundle a program and its dependencies.
They provide an alternative to dynamic linking, a default which locates and loads dependencies at runtime.

The idea is to take all of the executable code needed for a program, including any services typically provided by system libraries, and bake everything into one larger file.
The result is a stand-alone application.

> **Are we making an operational tradeoff?**
>
> Yes.
> For defenders, static linking complicates patching.
> Typically, an OS's package manager keeps system libraries up to date.
> And individual programs can link against a single, recent copy of the relevant library.
>
> Static linking means each individual program needs to be replaced to keep its dependencies up to date.
> We lose the ability to manage centralized copies of certain components.
>
> But static linking is great for portability and isn't readily supported by many programming languages.
> So let's see how it's done in Rust.

First, we'll verify that `rcli` is dynamically linked by default.
From the `crypto_tool/rcli` directory, run:

```ignore
cargo build --release
ldd ../target/release/rcli
```

`ldd` is a Linux command for printing shared library dependencies - those the OS distribution typically manages.
So that second command will output something like:

```ignore
linux-vdso.so.1 (0x00007ffc0196f000)
libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f09369b9000)
/lib64/ld-linux-x86-64.so.2 (0x00007f0936c8e000)
libpthread.so.0 => /lib/x86_64-linux-gnu/libpthread.so.0 (0x00007f0936996000)
libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007f093697b000)
libdl.so.2 => /lib/x86_64-linux-gnu/libdl.so.2 (0x00007f0936975000)
```

Each line represents a shared object (`.so` file) that the `rcli` tool expects to be present somewhere on the filesystem in order to function.

The second item (line starting with `libc.so.6`) is the C standard library.
Recall from this chapter's intro that our `rcli` front-end code links against parts of `libc` (e.g. for dynamic memory allocation).
Although our RC4 library does not (it's a `#![no_std]` component).

To avoid being reliant on the presence of these libraries, we can compile a static binary that will use `musl` (a tiny `libc` alternative[^MUSL]) instead:

```ignore
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

* The first command adds a new compilation target[^PlatformSupport], which is generally specified by a "target triple" in the form `{Arch}-{Vendor}-{Sys}-{ABI}`.

* The second command builds `rcli` as before, but this time for the target triple `x86_64-unknown-linux-musl`.

Now let's try `ldd` again, this time on the `musl`-target binary:

```ignore
ldd ../target/x86_64-unknown-linux-musl/release/rcli
```

The output should be:

```ignore
statically linked
```

Our second build of the `rcli` executable will "just work" on any x86_64 Linux system!
All you need to do is copy over the binary.

## Stripping Debug Info

If we want to distribute this executable, we should reduce its size by removing debug information (including symbols that allow matching to source code, something a CLI end-user won't need to do).

We can "strip" the binary of this information by adding the following `release` profile setting to the workspace's configuration file, `crypto_tool/Cargo.toml`:

```ignore
[profile.release]
strip = true
```

The setting applies to any target built with the flag `--release` (which enables optimizations).
We could've also used `strip`[^Strip], a standalone Linux utility, but we leveraged `cargo` to more cleanly integrate into the build pipeline.

> **An Alternative to `musl`**
>
> Though leveraging `musl` is a popular way to build small-ish static binaries, `musl` has its quirks.
> Particularly with regard to performance.
>
> To statically link your platform's standard C runtime ("CRT") instead[^RFC]:
>
> ```ignore
> RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-gnu
> ```
>
> Warning: unlike the `musl` route, the resulting binary might still be dynamic linked against something like `vdso`[^VDSO]. You can use `ldd` to verify.

## Takeaway

We've demonstrated building a completely free-standing tool.
Our binary will run natively, on nearly any client of given OS and ISA[^ISA].

That concludes our tour of software assurance!
In the next chapter, we'll dig into Rust proper.

---


[^Hellscape]: [*`hellscape`*](https://github.com/meme/hellscape). meme (Archived 2021).

[^MUSL]: [*musl libc*](https://musl.libc.org/). Rich Felker and contributors (Accessed 2022).

[^PlatformSupport]: [*Platform Support*](https://doc.rust-lang.org/nightly/rustc/platform-support.html). The Rust Team (Accessed 2022).

[^RFC]: [*RFC 1721*](https://rust-lang.github.io/rfcs/1721-crt-static.html). The Rust RFC Book (Accessed 2022).

[^VDSO]: [*`vdso`*](https://man7.org/linux/man-pages/man7/vdso.7.html). Linux manual (Accessed 2022).

[^ISA]: Instruction Set Architecture, e.g. x86_64.

[^Strip]: [*`strip`*](https://man7.org/linux/man-pages/man1/strip.1.html). Linux manual (Accessed 2022).

