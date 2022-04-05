# Interface-relevant Traits

The library we're building is an alternative to two collections in Rust's standard library: `BTreeSet`[^BTreeSet] and `BTreeMap`[^BTreeMap].
Our goal is to provide the same well-known, idiomatic APIs but with maximal safety (for any system) and bare metal portability (for firmware and/or tiny microcontrollers).

To get there, we need to understand a bit about the design of the standard library's APIs.
Specifically the *traits* these APIs bind to their *generic* arguments.
These design decisions shape an interplay of usability and resource management.

API design is a concern orthogonal to the algorithms of our particular data structure, so let's tackle it first.
To achieve feature-parity with the standard library, we'll deepen our understanding of how Rust works "under-the-hood".

> **What are generics and traits, again?**
>
> We introduced the concepts and syntax in chapter 3.
> To jog your memory:
>
> * **Generics** (e.g. `T` standing in for concrete type `u64` or `u32`) eliminate the need for code duplication. A single function's source code can be used, by the compiler, to generate one machine-code equivalent for each concrete type that function is called with (monomorphization).
>
> * **Traits** (e.g `T: Ord` for a type that can be sorted and compared) define behavior shared among different types. They're similar to interfaces and abstract bases classes of other languages.
>
> We often combine the two by binding traits to generic arguments and/or return values.
> This allows us to write a single function that our users can leverage for any [generic] type implementing some behavior (one or more specific traits). Even custom types that haven't been invented yet!

## The Map `get` API

A *map* (aka an *associative array* or *symbol table*) is a data structure that stores key-value pairs.
Keys are unique and values can be quickly looked up by key.

Rust's `BTreeMap`[^BTreeMap] is an *ordered map*, it supports any key type that has a notion of *total order*[^TotalOrder].
Colloquially, that means keys can be compared with logical operators (`>`, `<=`, `==`, etc) and sorted.
Because they implement the `Ord` trait.
If keys can't be ordered but are hashable, you'd want to use a `HashMap`[^HashMap] instead.

Say we want to perform to a lookup in an ordered map - to get the value associated with a given key, if any.
A `get` method should take a reference to a key as input, and return an `Option` (containing a value reference for the `Some` case, when the key is found).

That's how the standard library works, here's the official example[^BTreeMapGet1]:

```rust,noplaypen
use std::collections::BTreeMap;

let mut map = BTreeMap::new();
map.insert(1, "a");
assert_eq!(map.get(&1), Some(&"a"));
assert_eq!(map.get(&2), None);
```

Based on the above, you might expect the `get` method's signature to look like this for `BTreeMap<K, V>`:

```rust,ignore
/// Returns a reference to the value corresponding to the key.
pub fn get(&self, key: &K) -> Option<&V>
where
    K: Ord
{
    // ...function body here...
}
```

But it doesn't.
The real `get` method has this signature[^BTreeMapGet2]:

```rust,ignore
/// Returns a reference to the value corresponding to the key.
pub fn get<Q>(&self, key: &Q) -> Option<&V>
where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    // ...function body here...
}
```

Why are there two different generics types at play?
And what are all those strange looking trait bounds?

Let's build our way up to answering those questions by explaining each trait individually.
If you can understand this API, you're well on your way to understanding idiomatic use of traits in general.

### The `Ord` Trait

`Ord`[^Ord] is the simplest of the three traits in `get`'s signature, and one we've already discussed.
When a type implements `Ord`, it can be ordered[^TotalOrd].
We can compare one value of this type to another and determine if the two are equal or if one is greater than the other.
This enables us to sort values.

Back in Chapter 3, we implemented the `Ord` trait for a structure representing an OS process.
This allowed sorting a list of processes by a specific definition of priority (the process's current state, in our case).

### The `Sized` Trait

`Sized`[^Sized] is something known as a *marker trait*.
Unlike `Ord`, there's no "interface" methods to implement because `Sized` has no behavior of its own.
Marker traits *mark a property* instead of *specifying behavior* (plot twist!).

The trait bounds `T: Sized` tells the compiler that all values of type `T` have the same size in memory and that this size is known at compile time[^Sizedness].
For example, a `u32` is always 4 bytes long.

Here's where things get interesting: `T: ?Sized`, the binding used in the above signature (note the leading `?`), means values of type `T` are *optionally sized* - they *may or may not* be `Sized`.
Doesn't that seem weirdly ambiguous?

Turns out the ambiguity buys flexibility without introducing any UB.
The standard library designers wanted to handle both the common case and the exception.
The majority of types in Rust are sized, but a handful aren't.
Examples of unsized types include:

* **Slices:** An slice, `[T]`, can contain zero or more contiguous `T`s - thus different slice values could have different sizes.

    * Note that `&[T]`, a reference to a slice, is always the size of a pointer.

* **Trait Objects:** Rust has a mechanism for *dynamic dispatch*[^DynDis]. The `dyn` keyword indicates a *trait object*: a value that implements a given trait. That value can be *any type* and have *any size*, so long as it implements the trait.

    * `Box<dyn Error>`, for example, is a pointer to an instance of any type implementing `Error` trait.

Now values *stored* in a collection, like `BTreeMap` must be `Sized`.
Otherwise we wouldn't know how to store them in memory.

But because `get` supports both sized and unsized types as a parameter (`Q: ?Sized`), *searching* `BTreeMap` is flexible.
We can find values associated with *sized keys* using *unsized keys* of a corresponding type.
We'll see a concrete example toward the end of this section.

### The `Borrow` Trait

A type that implements `Borrow<T>`[^Borrow] can *borrow* a *reference*, `&T`.
Unlike the similar trait `AsRef`[^AsRef], `Borrow` requires that the borrowed `&T` have the same comparison and hash semantics as `T`.
Sounds relevant to `BTreeMap` (lookup via a sequence of comparisons) and `HashMap`s (lookup via a hash), right?

It very much is - the `Borrow` trait is designed to make collection lookups easier and more efficient.
In fact, the standard library includes a "blanket implementation" (always pre-implemented trait) for all types `T` to be able to borrow themselves (meaning we get `T: Borrow<T>` for "free").

This enables key lookups without having to create a copy of the key in memory.
So no need for additional heap allocations if searching a `BTreeMap<String, T>` by a key of type `&str`.

## Putting it all together

To really grok how `Ord`, `Sized`, and `Borrow` impact API usage in combination, let's walk through an example.

Say we store 8-byte hexspeak[^Hexspeak] words, e.g. values of type `[u8; 8]`, in a set.
We later get a list of user-provided hexspeak words of varying sizes, e.g. values of type `Vec<u8>`.
Some may be 8 bytes long, others may not.

We want to be able to use the `get` method to check if any user-provided words are already in our set.
Luckily, `get` lets us search for *slices* (unsized `[u8]`).
We can use the arbitrarily-sized, user-provided words as search keys for our set of fixed-size (8-byte words)!

```rust,noplaypen
use std::collections::BTreeSet;

// Two hexspeak words
let bad_code: [u8; 8] = [0xB, 0xA, 0xA, 0xD, 0xC, 0x0, 0xD, 0xE];
let bad_food: [u8; 8] = [0xB, 0xA, 0xA, 0xD, 0xF, 0x0, 0x0, 0xD];

// Note we're about to store uniformly sized values in our set
assert_eq!(std::mem::size_of_val(&bad_code), 8);
assert_eq!(std::mem::size_of_val(&bad_food), 8);

// Store the two words in our set
let mut set = BTreeSet::new();
set.insert(bad_code);
set.insert(bad_food);

// Vec<u8> is sized, it's actually a fat pointer to a heap buffer.
// But slices of the vec are unsized! For example:
//     &my_vec[0..5] is the first 5 elements
//     &my_vec[1..] is all but the first element
//     &my_vec[..] is all elements
let bad_food_vec: Vec<u8> = vec![0xB, 0xA, 0xA, 0xD, 0xF, 0x0, 0x0, 0xD];
let bad_dude_vec: Vec<u8> = vec![0xB, 0xA, 0xA, 0xD, 0xD, 0x0, 0x0, 0xD];
let cafe_bad_food_vec: Vec<u8> = vec![
    0xC, 0xA, 0xF, 0xE, 0xB, 0xA, 0xA, 0xD, 0xF, 0x0, 0x0, 0xD
];

// Search for a [u8; 8] present
assert_eq!(
    set.get(&bad_food_vec[..]),         // 0xBAADFOOD
    Some(&[0xB, 0xA, 0xA, 0xD, 0xF, 0x0, 0x0, 0xD])
);

// Search for a [u8; 4] not present
assert_eq!(
    set.get(&bad_food_vec[..4]),        // 0xBAAD
    None
);

// Search for an [u8; 8] not present
assert_eq!(
    set.get(&bad_dude_vec[..]),         // 0xBAADDUDE
    None
);

// Search for a [u8; 8] present
assert_eq!(
    set.get(&cafe_bad_food_vec[4..]),   // 0xBAADF00D
    Some(&[0xB, 0xA, 0xA, 0xD, 0xF, 0x0, 0x0, 0xD]),
);

// Search for a [u8; 12] not present
assert_eq!(
    set.get(&cafe_bad_food_vec[..]),    // 0xCAFEBAADF00D
    None
);
```

So what happened to enable searching fixed-length set elements (`[u8; 8]`) using arbitrary length (`[u8]`) keys?
Consider what the compiler converted our generic `get` callsites into, through the magic of monomorphization:

```rust,ignore
pub fn get(&self, key: &[u8]) -> Option<&[u8; 8]>
{
    // ...function body of code in our compiled binary...
}
```

* **`Ord` and `Sized`** - The trait bound `Q: Ord + ?Sized` means we're free to search using an arbitrarily sized slice, so long as the slice's contents can be sorted. `[u8]` meets that criteria. In the above, we converted user-provided vectors into slices.

* **`Ord` and `Borrow`** - The trait bound `K: Borrow<Q> + Ord` enables that conversion. We can search using any key that can borrow the aforementioned arbitrarily-sized-and-sortable slice. A `Vec` can view its elements as a contiguous slice, regardless of how many are stored. Since `Vec<T>` implements `Borrow<[T]>`, `Vec` can also borrow that slice from itself (no data copied!). Thus `&my_vec[..]` (slicing notation shorthand for `my_vec.as_slice()`) lets us pass in an `&[u8]` key to search for.

In conclusion, `BTreeMaps`'s `get` combines three traits (`Ord`, `?Sized`, and `Borrow`) to enable flexible, efficient APIs.

### Taking It a Step Further: The `Default` Trait

The library we build will bring a fourth trait into the mix: `Default`[^Default].
Like it sounds, this trait is for types that have a default value.
For example:

* The default for `isize` is `0`.

* The default for `Option` is `None`.

* The default for any dynamic collection (`Vec`, `BTreeSet`, `HashMap`, etc) is an empty instance of that collection.

Our API will look like this:

```rust,ignore
/// Returns a reference to the value corresponding to the key.
pub fn get<Q>(&self, key: &Q) -> Option<&V>
where
    K: Borrow<Q> + Ord + Default,
    Q: Ord + Default + ?Sized,
{
    // ...function body here...
}
```

Don't worry, it's easier to use than to read.
Yet the choice to require `Default` for keys and values is restrictive, users of our library have to ensure the trait is implemented for any custom type they want to store in one of our collections.

Why enforce that sort of limitation?
`Default` is like a "no argument constructor", it *ensures* that values of a type are *always safely initialized*.

It's a requirement for elements stored in `tinyvec`[^TinyVec], the 3rd party `#![forbid(unsafe_code)]` library we used for our arena allocator in the previous chapter.
So the `Default` restriction is inherited from a dependency.

Imposing it is an assurance tradeoff.
We ask a little more of our users in exchange for a 100% safe binary, the guarantee that all our code and all dependencies of our code (e.g. the full library *supply chain*) maximizes memory safety.

If you are morally opposed to requiring `Default` and want to remain exactly API-compatible with the standard library, feel free to swap `tinyvec` for `smallvec`[^SmallVec] in your allocator now and adjust all non-test code for the remainder of this book.
`smallvec` is another stack-based `Vec` alternative.
It's used in Mozilla's Servo browser engine.

Unfortunately, `smallvec` contains `unsafe` code.
Security researchers have discovered multiple memory safety vulnerabilities in `smallvec`, for which CVEs have been assigned (e.g. CVE-2021-25900, CVE-2019-15554, CVE-2018-20991, etc).

While `smallvec` is popular and well-vetted, we can make no guarantee about the number of *undiscovered* memory safety vulnerabilities still present.
`tinyvec`, by contrast, will never fall victim to any memory corruption attacks - it's `#![forbid(unsafe_code)]`.

> **Any other traits should I know about?**
>
> There isn't an official list of traits every Rust programmer should know.
> But you'll almost certainly run into three traits related to memory allocation and deallocation: `Clone`, `Copy`, and `Drop`.
> We've touched on some of these before, but they're worth revisiting.
>
> * `Clone`[^TraitClone] defines *deep* copy logic. `Clone` types must be `Sized`. Cloning could be expensive if the original needs to be recursively traversed - we have to allocate a counterpart to everything it owns.
>
>   * For example, copying `Vec<String>` means copying each `String`. `String` is a `Vec<u8>` internally, so copying each `String` means copying each `u8`. That was only 2 levels of recursion, but `Clone` could require arbitrarily many.
>
>   * If your code is littered with `my_structure.clone()` calls, removing them *might* be a "low-hanging fruit" performance optimization. If you can refactor flows of ownership to process primarily references (e.g. replace `String` with `&str`), you *might* save a precious time and memory. "Might" stems from the fact that performance optimizations need to be data driven, not premature. We'll cover micro-benchmarking in Chapter 12.
>
> * `Copy`[^TraitCopy] is a marker trait for types that can be fully cloned with only a *shallow* byte-by-byte copy. That means there's no pointers to follow or external resources to duplicate a handle to.
>
>   * Consider `isize`, the platform-specific signed integer type. If we duplicate the small chunk of fixed-sized, consecutive bytes that encodes the integer's value then we get a complete replica of the original.
>
>   * `Copy` should be implemented sparingly. It means the assignment operator, `=`, will copy bytes (implicit "copy semantics") instead of just transferring ownership ("move semantics").
>
> * `Drop`[^TraitDrop] defines a "destructor". User-definable deallocation logic, called when a variable of the implementing type goes out of scope. All memory and shared resources must be freed. Types that implement `Copy` are not allowed to implement `Drop` (these should be mutually exclusive - bitwise copyable memory can be bitwise erased).
>
>   * Note that if the scope of a variable's binding depends on conditional statements, move semantics will be tracked at runtime. The value can be moved here and there, based on which branch is taken, as the program runs. But in the end Rust will only drop it *once* - when the last-moved location goes out of scope.

## Takeaway

As mere users of the standard library's `BTreeSet`/`BTreeMap`, the nuances of `Ord`, `?Sized`, and `Borrow` would likely be lost on us.
We could have long and prosperous careers without ever having to think about why a map `get` signature looks like it does.

But as designers and implementers of an API-compatible alternative, we want to empower our users with the same flexible abstractions the standard library provides.
That entails understanding these traits and how they interact.

The hexspeak example above wouldn't even have compiled if the standard library used the more intuitive signature we started this section with (`pub fn get<K: Ord>(&self, key: &K) -> Option<&V>`).
So the complexity we've covered has a major payoff: the same code seamlessly supports a broader range of use cases.

With all that trait binding background behind us, we know how and why specific interfaces are designed a certain way.
Now let's tackle the logic backing them: the core operations of our self-balancing scapegoat tree.

---

[^BTreeSet]: [*Struct `std::collections::BTreeSet`*](https://doc.rust-lang.org/stable/std/collections/struct.BTreeSet.html). The Rust Team (Accessed 2022).

[^BTreeMap]: [*Struct `std::collections::BTreeMap`*](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html). The Rust Team (Accessed 2022).

[^TotalOrder]: [Total order](https://en.wikipedia.org/wiki/Total_order) Wikipedia (Accessed 2022).

[^HashMap]: [*Struct `std::collections::HashMap`*](https://doc.rust-lang.org/std/collections/struct.HashMap.html). The Rust Team (Accessed 2022).

[^BTreeMapGet1]: [*`BTreeMap` `get` API example*](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#examples-3). The Rust Team (Accessed 2022).

[^BTreeMapGet2]: [*`BTreeMap` `get` API*](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#method.get). The Rust Team (Accessed 2022).

[^Ord]: [*Trait `std::cmp::Ord`*](https://doc.rust-lang.org/std/cmp/trait.Ord.html). The Rust Team (Accessed 2022).

[^TotalOrd]: [Total order](https://en.wikipedia.org/wiki/Total_order) Wikipedia (Accessed 2022).

[^Sized]: [*Trait `std::marker::Sized`*](https://doc.rust-lang.org/std/marker/trait.Sized.html). The Rust Team (Accessed 2022).

[^Sizedness]: [*Sizedness in Rust*](https://github.com/pretzelhammer/rust-blog/blob/master/posts/sizedness-in-rust.md).  pretzelhammer (2020).

[^DynDis]: [Dynamic dispatch](https://en.wikipedia.org/wiki/Dynamic_dispatch). Wikipedia (Accessed 2022). One use case for dynamic dispatch is enabling *heterogeneous collections*. `Vec<Box<dyn Error>>`, for example, allows us to store a vector of `Error` objects, potentially of varying types.

[^Borrow]: [*Trait `std::borrow::Borrow`*](https://doc.rust-lang.org/std/borrow/trait.Borrow.html). The Rust Team (Accessed 2022).

[^AsRef]: [*Trait `std::convert::AsRef`*](https://doc.rust-lang.org/std/convert/trait.AsRef.html). The Rust Team (Accessed 2022).

[^Hexspeak]: [Hexspeak](https://en.wikipedia.org/wiki/Hexspeak). Wikipedia (Accessed 2022).

[^Default]: [*Trait `std::default::Default`*](https://doc.rust-lang.org/std/default/trait.Default.html). The Rust Team (Accessed 2022).

[^TinyVec]: [*`tinyvec`*](https://crates.io/crates/tinyvec). Lokathor (Accessed 2022).

[^SmallVec]: [*`smallvec`*](https://crates.io/crates/smallvec). Simon Sapin, Ms2ger, Servo project (Accessed 2022).

[^TraitClone]: [*Trait `std::clone::Clone`*](https://doc.rust-lang.org/std/clone/trait.Clone.html). The Rust Team (Accessed 2022).

[^TraitCopy]: [*Trait `std::marker::Copy`*](https://doc.rust-lang.org/std/marker/trait.Copy.html). The Rust Team (Accessed 2022).

[^TraitDrop]: [*Trait `std::ops::Drop`*](https://doc.rust-lang.org/std/ops/trait.Drop.html). The Rust Team (Accessed 2022).