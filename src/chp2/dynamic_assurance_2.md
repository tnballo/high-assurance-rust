# Dynamic Assurance (2 of 3)

Like any stream cipher, RC4 needs to generate a *keystream* and bitwise XOR it with *plaintext* to create *ciphertext*. That's how encryption works.

* *Keystream* - data that is reproducible but indistinguishable from random.

* *Plaintext* - unencrypted data.

* *Ciphertext* - encrypted data.

Keystream generation is implemented using a buffer to represent *cipher state*.
Mechanically, RC4's cipher state is a 256 byte array, named `s`, and indexed with two variables, `i` and `j`.
Our first step is creating a structure to store this ever-changing state and the current values of its indexes.
We'll want to add the following at the top of `crypto_tool/rc4/src/lib.rs`:

```rust,ignore
{{#include ../../code_snippets/chp2/crypto_tool/rc4/src/lib.rs:Rc4}}
```

* The first 2 lines are *attributes*: they communicate with the compiler to configure our project.

* `#![cfg_attr(not(test), no_std)]` is a conditional attribute. It applies to the whole crate and informs the compiler that, unless doing a `test` build, our library makes no assumptions about the system it's going to run on.

    * `no_std` roughly translates to "don't depend on a standard library or runtime support being available". Although this restricts us to a set of core Rust features, it makes our code portable for embedded use cases: firmware, bootloaders, kernels, etc. We'll discuss `#![no_std]` more thoroughly in Chapter 4.

* `#![forbid(unsafe_code)]` is an unconditional attribute. It again applies to the entire crate, telling the compiler to *ensure* the library has no `unsafe` code blocks. This allows our code to maximize Rust's memory safety guarantees, even if we refactor it or add new features later.

    * We'll discuss `unsafe` throughout the book, but won't use this keyword in our main project.

* `#[derive(Debug)]` is a *derive macro* for something called a *trait* (definition of shared behavior, explained in Chapter 3). Macros generate additional code. Writing macros is an advanced topic, but you can leverage existing macros even as a beginner[^BeginMacro].

    * Notice how `#[derive(Debug)]` sits atop the `Rc4` structure? It only applies to this structure, telling the compiler how to pretty print its contents to a console[^TraitDebug]. Using this macro makes our stream cipher convenient to visually debug in test builds.

* The `Rc4` structure is the most important part of the above code. Though not an *object* in the traditional sense[^Obj], our structure encapsulates private data and we're going to define methods that operate on that data next. `Rc4`'s three fields are:

    * `s`: cipher state, an array of 256 bytes (unsigned, 8-bit integers - hence `u8`).

    * `i`: "incrementing" index for key stream generation.

    * `j`: "jumping" index for key stream generation.

We're now ready to implement the two halves of RC4's logic: KSA and PRGA.

> **WARNING! RC4 is insecure.**
>
> Real-world projects need to select a well-audited implementation of a modern, well-tested cipher.
> Remember, we've chosen RC4 for this chapter's example because it's relatively easy to implement.
> RC4 isn't suitable for professional projects.

## 1. The Key-Scheduling Algorithm (KSA)

The goal of RC4's KSA step is initializing the cipher state array by computing a *permutation* influenced by a variable-length (40 to 2,048 bit) secret key.

It's best to put this logic in `Rc4`'s constructor.
So that a library user doesn't have to remember to call a special initialization function before encrypting data.
The cipher instance returned by the constructor will already be initialized.

> **Function-related Terminology**
>
> This section will use two technical terms.
> The concepts aren't unique to Rust, but the terms have specific meaning in Rust programs:[^AssocMeth]
>
> * **Associated function:** A function that is defined on a structure, but *does not* take `&self` (reference to instance of structure) as its first parameter. It doesn't read or write structure fields.
>
> * **Method:** A function defined on a structure that *does* take `&self` or `&mut self` as the first parameter. It reads and/or writes fields on a specific instance of a structure.

By convention, Rust constructors are *associated functions* (no `self` parameter) named `new` that return an instance of the structure being constructed.

Let's add one that performs KSA, right below the `Rc4` structure definition.
Notice we define `new` inside an `impl Rc4` block, tying it to the structure of the same name:

```rust,ignore
impl Rc4 {
{{#include ../../code_snippets/chp2/crypto_tool/rc4/src/lib.rs:new}}
}
```

The above code might make you a little uncomfortable.
That's OK.
Learning any new language involves squinting at code you don't fully understand.
And that's usually not a great feeling.

To make matters worse, cryptographic code is just its own weird thing - regardless of the implementation language.
Let's double down and try to make sense of it:

* `new` takes a single parameter, `key`, which is a *reference to a slice of bytes*. This signature makes passing in key data efficient[^SliceEff] and flexible[^SliceFlex]. We'll cover slices in Chapter 3.

* The `assert!` statement, another macro, ensures the user of our API provides a key of valid length. If not, our program will terminate at this line. That's an aggressive way to handle errors. We'll talk about other options later.

* `let mut rc4 = ...` creates a *mutable* instance of our `Rc4` structure with all fields zero initialized. Variables are immutable by default in Rust. But we'll be setting up cipher state (the `s` array), we need the `mut` keyword here.

* The next bit of code, a `for` loop identity permutation[^IDPerm], is just a fancy way to set `s[0] = 0, s[1] = 1, s[2] = 2, ..., s[255] = 255`. It uses *iterators*. We'll implement our own iterators in Chapter 10, so let's not dwell on the syntax right now.

* The subsequent `for` loop *further permutes* the cipher state `s`. Three details worth pointing out:

	* We have to use the `wrapping_add` function instead of the addition operator (`+`) in cryptographic code because we want *integer overflow* (explanation coming in Chapter 3) to emulate modular arithmetic[^ModArith].

    * Have you ever swapped two variables using a third (probably named `temp`)? If your answer is "good God, a hundred times" then you'll appreciate how `swap` is a built-in method for arrays in Rust.

    * Indexes are always register-width unsigned integers in Rust. So, in the call to `swap`, we promote `j` (a lowly `u8`) to a `usize` with the `as` keyword. Think of this minor detail as a "safe cast"[^Cast].

* The final line of the `new` function returns an initialized instance of an `Rc4` structure. Rust functions don't need the `return` keyword unless you want to return early (e.g. halfway through the function body) for some reason.

    * The return type of the function (specified right after `->`) is `Self`. Because `new` is inside an `impl Rc4` block, this is shorthand for returning an instance of an `Rc4` structure.

Visualizing a round of permutation might make the concept more tangible.
Every loop iteration, `i` and `j` change (with `j` being influenced by the key) and `rc4.s.swap(i, j as usize)` just switches two values within `s`:

<br>
<p align="center">
  <img width="100%" src="rc4_1.svg">
  <figure>
  <br>
  <figcaption><center>Visualization of swapping <i><b>s[i]</i></b> and <i><b>s[j]</i></b></center></figcaption><br>
  </figure>
</p>

## 2. Pseudo-Random Generation Algorithm (PRGA)

The `new` function creates and initializes an instance of the `Rc4` cipher.
We need another function that uses an `Rc4` instance to generate a keystream.
Once we have a keystream, we can encrypt data with it.

`prga_next` is our keystream generation function, it outputs a single keystream byte each time it's called.
We'll add it right after the `new` function, inside the same `impl Rc4` block.

Unlike the `new` *associated function*, `prga_next` is a *method*.
Methods always take a reference to `self`, an instance of the structure they're being called on, as their first parameter.

```rust,ignore
impl Rc4 {
    // ..new() definition omitted..

{{#include ../../code_snippets/chp2/crypto_tool/rc4/src/lib.rs:prga_next}}
}
```

This function performs similar operations to the `new` function, so we don't need to go over it in detail.
We're concerned with getting a taste of Rust, not with the specific operations RC4's design dictates.
There is, however, one detail worth pointing out:

* `prga_next`'s sole parameter is `&mut self`, a *mutable reference* to the `Rc4` structure on which it will be called. We need the `mut` keyword here again because this function makes changes to an `Rc4` struct - it writes indexes `i` and `j`, and swaps bytes inside the cipher state buffer `s`.

As an aside - we can visualize that second-to-last line, `let k = ...`, like so:[^RC4Wiki]

<br>
<p align="center">
  <img width="100%" src="rc4_2.svg">
  <figure>
  <br>
  <figcaption><center>Visualization of <i><b>k = s[(s[i] + s[j]) mod 256]</i></b></center></figcaption><br>
  </figure>
</p>

## 3. {En,De}cryption

### The Classic Flexible Interface

We implement encryption by XORing each `prga_next` output byte (keystream) with each byte of the plaintext.
Since XOR is reversible, the same function also works for decryption!

```rust,ignore
impl Rc4 {
    // ..new() definition omitted..

{{#include ../../code_snippets/chp2/crypto_tool/rc4/src/lib.rs:apply_keystream}}

    // ..prga_next() definition omitted..
}
```

Implementing encryption within a *method* maximizes flexibility: if we receive data in [potentially variable length] chunks, a single instance of `Rc4` can perform "running" encryption across multiple chunks like so (the below is an API usage example, not part of our `Rc4` implementation):

```rust,ignore
let key = [0x1, 0x2, 0x3, 0x4, 0x5];

let msg_1 = [0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
let msg_2 = [0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21]; // " World!"

// Encrypt in-place
let mut rc4 = Rc4::new(&key);
rc4.apply_keystream(&mut msg_1);
rc4.apply_keystream(&mut msg_2);

// Decrypt in-place
let mut rc4 = Rc4::new(&key);
rc4.apply_keystream(&mut msg_1);
rc4.apply_keystream(&mut msg_2);
```

Most real-world stream cipher libraries use an API like this one.
But it entails subtle complexity: `rc4` is stateful and must be re-constructed prior to decryption with `new`.
Moreover, the order of parameters to `apply_keystream` matters - decryption would produce the incorrect result if we accidentally called `rc4.apply_keystream(&mut msg_2)` before `rc4.apply_keystream(&mut msg_1)` in the above.

### Making the Common Case Easier

Implementing encryption within an *associated function* provides a simpler interface, so long as all the data is in memory at once.
Which might be the case reasonably often.
Notice it's really just a wrapper that hides state from the caller:

```rust,ignore
impl Rc4 {
    // ..new() definition omitted..

    // ..apply_keystream() definition omitted..

{{#include ../../code_snippets/chp2/crypto_tool/rc4/src/lib.rs:apply_keystream_static}}

    // ..prga_next() definition omitted..
}
```

Now we can en/decrypt with a single method call, no need to worry about the state of an `Rc4` instance (API usage example below):

```rust,ignore
let key = [0x1, 0x2, 0x3, 0x4, 0x5];

let msg = [
    0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f,
    0x72, 0x6c, 0x64, 0x21,
]; // "Hello World!"

// Encrypt in-place
Rc4::apply_keystream_static(&key, &mut msg);

// Decrypt in-place
Rc4::apply_keystream_static(&key, &mut msg);
```

With our two en/decryption functions done, we've now finished implementation.
Time for validation.
Cryptography software really needs to be correct, we can't stop here.
Let's put this code through its paces!

> **How can encryption and decryption be the same operation?**
>
> In short, because XOR is both reversible and, due to the nature of the keystream, unpredictable:
>
> * First, `cipher_text = plain_text ^ key_stream` (encryption).
>
> * Then, `plain_text = cipher_text ^ key_stream` (decryption).
>
> * The key stream can flip any bit in the plaintext as if by 50/50 random chance.
>
> For a more mathematically principled treatment, we recommend the proof on page 32 of Paar and Pelzl's *Understanding Cryptography*[^UnderstandingCrypto].
> While it's a university textbook, the formalisms are lightweight and precise.
> It's an excellent introduction to the field of cryptography.
> And the book is supplemented by free video lectures[^UnderstandingCryptoVideo].

---

[^BeginMacro]: Unlike C macros, Rust macros are *hygienic*: they won't cause subtle problems by capturing identifiers. This is part of what makes them so easy to use. In fact, `println!` is a macro. So you already used a macro when running the "Hello world!" program at the end of Chapter 1.

[^TraitDebug]: [*Trait `std::fmt::Debug`*](https://doc.rust-lang.org/std/fmt/trait.Debug.html). The Rust Team (Accessed 2022).

[^Obj]: In Rust, shared behavior is defined by *trait composition*, not by *object-oriented inheritance*. There's no "class hierarchy", like in C++ or Java. We'll cover traits in Chapter 3.

[^AssocMeth]: Technically, per the Rust reference[^RustRef], "Associated functions are functions associated with a type" and "Associated functions whose first parameter is named `self` are called methods...". But that's pretty in the weeds. We treat *associated functions* and *methods* as distinct in this section for clarity.

[^SliceEff]: Slice references are "fat pointers" (tuple of pointer and element count), they allow us to pass variable-length data without copying it (recall "pass-by-reference", from when we first talked about pointers).

[^SliceFlex]: Slices are flexible because different kinds of collections (say, a fixed-size array or dynamically-sized vector) can be "viewed" through a slice. So you'll encounter them often in idiomatic Rust code.

[^IDPerm]: [*Neutral element and inverses*](https://en.wikipedia.org/wiki/Permutation_group#Neutral_element_and_inverses). Wikipedia (Accessed 2022).

[^ModArith]: [*Modular arithmetic*](https://en.wikipedia.org/wiki/Modular_arithmetic). Wikipedia (Accessed 2022).

[^Cast]: There are best practices related to casting in Rust. Namely using traits `From` and `Into` for *infallible* conversions between types, and `TryFrom` and `TryInto` for *fallible* conversions. We'll discus this topic in detail later.

[^RC4Wiki]: [*RC4*](https://en.wikipedia.org/wiki/RC4). Wikipedia (Accessed 2022).

[^UnderstandingCrypto]: [***[PERSONAL FAVORITE]** Understanding Cryptography*](https://amzn.to/3IEYuNd). Christof Paar, Jan Pelzl (2009).

[^UnderstandingCryptoVideo]: [*Online Cryptography Course*](https://www.crypto-textbook.com/movies.php). Christof Paar, Jan Pelzl (2009).

[^RustRef]: [*The Rust Reference: Associated Items*](https://doc.rust-lang.org/reference/items/associated-items.html). The Rust Team (2021).
