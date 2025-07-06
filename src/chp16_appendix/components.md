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


# Fundamentals: Component-Based Design

Assume an Application Programming Interface (API) includes one or more of the below pieces:

* Data type definitions (structures, enums, etc)
* Function declarations
* Constants
* Specifications for remote request-response formats, such as:
    * REST (Representational State Transfer)[^REST]
    * gRPC (Google Remote Procedure Call)[^GRPC]

"Deep modules", according to John Ousterhout[^DeepMod], are those in which well-designed APIs hide, or abstract, an iceberg of underlying complexity.

Deep modules often have the advantage of making codebases easier to maintain and refactor.
Good abstractions also make APIs straightforward to learn initially and use correctly.
Whether the interface is for an external customer or for another component within the codebase.

Depth becomes especially important once we consider the cumulative complexity of a large system with multiple *components*.

In most contexts, *module* and *component* are synonymous.
But for our purposes, a component is composed of one or more modules.
So a component is a bigger (potentially multi-module) piece.
We'll soon see diagrams that make this distinction clearer.

> **What's a real-world example of a "deep module"?**
>
> Ousterhout cites *system calls*[^DeepMod], the OS mechanism by which userspace applications request hardware-related and/or privileged services, as a prototypical example.
>
> A small handful of calls abstract the gory details of, say, writing a file to a physical hard disk of a particular variety and manufactured by a particular vendor.
> The OS provides a small and stable API *externally*, while being free to manage the inherent complexity *internally*.
>
> One could argue that the core project of this book is, like many other dynamic collections, also a deep module.
> We provide an API-compatible alternative to a standard library collection, but abstract away the specifics of the underlying data structure and its memory management strategy.

So, when leveraging any code organization facility, be it Rust's module system or some other language's equivalent, our goal is to create "deep" modules and collect them into components with "loose" coupling.
Meaning well-isolated pieces that offer rich functionality via a small API surface.

In general, this approach results in a codebase that's easier to work with for new team members and easier to improve for everyone.
That means less time firefighting and more time shipping new features.
And, by keeping complexity in check, we also reduce security and reliability risks.

## Effective Componentization

Depth tends to, naturally, minimize coupling and maximize cohesion. Defined as[^Ccord]:

* **Coupling:** a measure of interdependency between APIs.

    * E.g. Mutual reliance on the same custom data types in public signatures. Or private, global shared state.

* **Cohesion:** a measure of the commonality between individual elements of an API.

    * E.g. Do functions exposed by a module have a clear logical relationship to each other? If so, the module has high cohesion.

While low coupling, high cohesion components are generally desirable, they may not always be practical.
For example, a centralized piece of functionality can be more easily replaced with a faster algorithm or a more secure implementation.
But centralization sometimes increases coupling.

Similarly, an API that's overly-specific is hard to future-proof - new requirements can mean a breaking change.
But if an API is too general, it likely requires cumbersome wrappers to meet current, specific needs without reducing cohesion.

## Visualizing Component-based Design

Components whose modules have fewer and simpler public APIs often entail less stability burden and lower chance of misuse.
Such components help us more effectively compose large, ambitious, multi-component systems.

Visually, that entails moving away from **fragile systems** where components expose and rely on each other's internals:

</br>
<p align="center">
  <img width="100%" src="mod_shallow.svg">
  <figure>
  <figcaption><center>Fragile: shallow components with high/complex coupling.</center></figcaption><br>
  </figure>
</p>

And toward **agile systems** that abstract away internal complexity (while delivering the same functionality):

</br>
<p align="center">
  <img width="100%" src="mod_deep.svg">
  <figure>
  <figcaption><center>Agile: deep components with low/loose coupling.</center></figcaption><br>
  </figure>
</p>

Here, "agile" means a codebase that's easy to onboard for, extend, and refactor.
Not Agile[^Agile], the umbrella term for a set of software development frameworks.

Note how, in both designs, the number of modules within each component (six) didn't change.
We're not removing functionality, just external-facing complexity.
The end user's cognitive load is reduced.
Total "work done" isn't.

## The Importance of Planning and Iterating

Complexity is the enemy of both productivity and security.
But the first iteration of a feature to hit production likely won't be elegantly crafted.
Aiming for perfection is unrealistic in most commercial contexts.

Instead, we can aim to make our first version *well-designed*.
That may mean using our organization's or team's current quality bar as a watermark.
And striving to push it a bit higher while still delivering on time.

Now the first architecture is sometimes the one a system gets stuck with for its entire lifecycle.
So budgeting design time up front can pay significant dividends.
For production infrastructure, the result could be a reduced number of 3:00am phone calls for outages and breaches.
But the average case, planned maintenance, is also lower cost for well-designed systems.

The best designs for high-value systems are almost always a result of iteration.
When we have the opportunity to significantly refactor an existing system, or create a successor from scratch, we can apply lessons learned.
So even if you can't justify a sweeping change today, it's worth noting current limitations for tomorrow.

## Takeaway

Low-complexity systems tend to be more reliable, maintainable, and secure.
Keeping complexity in check typically means designing for low coupling and high cohesion.
Deep modules lend themselves well to both goals.

[^REST]: [What is a REST API?](https://www.redhat.com/en/topics/api/what-is-a-rest-api). RedHat (2020).

[^GRPC]: [Core concepts, architecture and lifecycle](https://grpc.io/docs/what-is-grpc/core-concepts/). Google (Accessed 2022).

[^DeepMod]: [***[PERSONAL FAVORITE]** A Philosophy of Software Design*](https://amzn.to/3DbaPrp). John Ousterhout (2021).

[^Ccord]: [***[PERSONAL FAVORITE]** Effective C: An Introduction to Professional C Programming*](https://amzn.to/3wBuNu7). Robert Seacord (2020).

[^Agile]: [*What is Agile?*](https://www.agilealliance.org/agile101/). Agile Alliance (Accessed 2022).
