# Warmup: Environment Setup

In lieu of the usual chapter-end challenge, let's do a warm up: setting up the Rust toolchain.
What you'll need to write and compile Rust programs.
Consider this your bootstrap sequence!

This book is developed on a Linux host.
Throughout each chapter, you'll see commands you can retype (or copy/paste, note the button on the right) and execute.
In a code-block-listing format, like this:

```ignore
whoami
```

The commands assume you're using a mainstream Linux distribution.
To follow along, you have two options:

1. **Linux-native:** Install each tool introduced on your host, using its current official documentation (we won't attempt to duplicate all installation instructions here) or your distro's package manager. Most of our tools have straightforward setups, often one or two commands.

2. **Docker container:** If you're on a non-Linux platform, install Docker[^Docker] (a popular containerization tool) and use this book's provided Dockerfile[^BookDocker] to build a self-contained development environment.

> **Note: Current State of the Container**
>
> This book's container support is currently a work in progress.
> In the future, the container will be automatically tested and detailed instructions for working with it will be added to the appendix.
>
> Currently, we provide a simple Dockerfile within the book's repo[^BookDocker].

Regardless of which route you choose, you'll want to verify your toolchain is working.

## Try the Toolchain

`rustup` is the official installer and update manager for the Rust toolchain.
Among other pieces, it bundles Rust's:

* Compiler (`rustc`)
* Package manager (`cargo`)
* Standard library (`std`)

`rustup` can select toolchain components from three "channels": stable, beta, and nightly.
We'll cover the differences in Chapter 3.
It can also add components to cross-compile for other platforms.
We'll compile for an emulated ARM Cortex-M microcontroller in Chapter 8.

If not using the container, you can install `rustup` by following the instructions here:

* [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

To verify the installation, execute the command:

```ignore
rustup --version
```

Let's ensure you can compile and run a Rust program.
To create a new project named `hello_world`:

```ignore
cargo new --bin hello_world
```

Running `tree hello_world` should show the following directory layout:

```ignore
hello_world/
├── Cargo.toml
└── src
    └── main.rs

1 directory, 2 files
```

`Cargo.toml` is a build configuration file.
You won't need to edit it to compile this program.
Take a look at the contents of `main.rs`, the sole source file:

```ignore
cd hello_world
cat src/main.rs
```

A traditional "Hello, world!" program has been pre-populated:

```rust
fn main() {
    println!("Hello, world!");
}
```

To compile and run it, from with the `hello_world` directory, use:

```ignore
cargo run
```

If you see the greeting printed to the console, you've got a working toolchain!

## Add `rust-analyzer` to Your IDE of Choice

`rust-analyzer`[^RustAnalyzer] is an implementation of the Language Server Protocol[^LangProt] for Rust.
It integrates with several editors and Integrated Development Environments (IDEs) to support conveniences like:

* Syntax/warning/error highlighting
* Code completion
* "Goto" functionality for definitions
* Symbol renaming

Workflow aids like these can boost productivity.
Both for newcomers to a language and for professionals working on large codebases.
While not a requirement, we strongly recommend installing `rust-analyzer` or an equivalent (several commercial IDEs offer similar Rust support).

To get started with `rust-analyzer`, see it's manual:

* [https://rust-analyzer.github.io/manual.html](https://rust-analyzer.github.io/manual.html)

## The Optional Docker Route

Docker[^Docker] is a popular containerization tool.
Containers allow us to build and configure entire environments in a repeatable and reliable way:

* In **industry**, containers can package a networked application alongside its dependencies so it's ready to deploy on cloud infrastructure.

* In **academia**, containers bundle prototypes and analyses so that research results can be independently reproduced.

* In **open-source software**, containers automate building a project and thus reduce the entry barrier for current/future contributors.

Our motivation is closest to the open-source use case.
Instead of setting up a container to build a specific project, we'll use a container pre-setup for developing Rust projects.

### But I want to learn Rust, not Docker!

Modern software development involves multiple tools and processes.
Docker solves problems that are orthogonal to those solved by any particular programming language.

In the context of this book, it'll give you a reliable, supported environment for running examples and completing hands-on challenges.
No "it worked on my machine" issues - you'll get the same hassle-free experience whether you're running Windows, Linux, or MacOS.

It's worth at least knowing how to build and run pre-existing Docker containers.
Which is the extent of our Docker coverage.
We won't cover advanced `docker` usage or writing `Dockerfiles`.
Or Docker internals, beyond the blurb below.

> **How does Docker actually work, in nutshell?**
>
> Traditional Virtual Machines (VMs) provide isolation via duplication: they run an entire OS kernel, atop a [Type 2[^Type2]] hypervisor, atop your system's kernel.
> That's quite slow because now you're running two OS kernels, host and guest, plus glue software.
>
> By contrast, a Linux Docker container on a Linux host leverages special features of the host kernel ("control groups"[^Cgroups] and "namespaces"[^Namespaces]) to isolate the container, as if it existed on a separate filesystem.
> There's no duplication of kernels.
>
> Relative to VMs, you can run isolated applications faster (throughput) and/or fit more on a single physical machine (density).

### Getting Started with the Book's Docker Container

Platform-specific installation instructions for Docker are available here:

* [https://docs.docker.com/get-docker/](https://docs.docker.com/get-docker/)

The book's Dockerfile is available here:

* [https://github.com/tnballo/high-assurance-rust/blob/main/Dockerfile](https://github.com/tnballo/high-assurance-rust/blob/main/Dockerfile)

Your IDE of choice may offer a plugin for connecting to and/or managing containers.
For example, VSCode offers an official extension:

* [https://code.visualstudio.com/docs/remote/containers](https://code.visualstudio.com/docs/remote/containers)

## Checkpoint

Before proceeding, please ensure you've successfully compiled and run the above "Hello, world!" program.
Either natively or in a container.

We're going to write a more interesting program in the next chapter, you'll want a working toolchain to follow along.

---

[^Docker]: [*Docker overview*](https://docs.docker.com/get-started/overview/) Docker (Accessed 2022).

[^BookDocker]: [https://github.com/tnballo/high-assurance-rust/blob/main/Dockerfile](https://github.com/tnballo/high-assurance-rust/blob/main/Dockerfile)

[^RustAnalyzer]: [*`rust-analyzer`*](https://microsoft.github.io/language-server-protocol/). rust-analyzer (Accessed 2022).

[^LangProt]: [*`Language Server Protocol`*](https://microsoft.github.io/language-server-protocol/). Microsoft (Accessed 2022).

[^Type2]: [*Hypervisor: Classification*](https://en.wikipedia.org/wiki/Hypervisor#Classification). Wikipedia (Accessed 2022).

[^Cgroups]: [*Everything You Need to Know about Linux Containers, Part I: Linux Control Groups and Process Isolation*](https://www.linuxjournal.com/content/everything-you-need-know-about-linux-containers-part-i-linux-control-groups-and-process). Petros Koutoupis (2018).

[^Namespaces]: [*Everything You Need to Know about Linux Containers, Part II: Working with Linux Containers (LXC)*](https://code.visualstudio.com/docs/remote/containers). Petros Koutoupis (2018).