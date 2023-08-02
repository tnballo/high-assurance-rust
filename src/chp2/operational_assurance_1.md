# Operational Assurance (1 of 2)

Unfavorably comparing software engineering to civil engineering has become something of a cliche.
The oft-posed provocation is along the lines of:

> If we can reliably build correct bridges, such that they don't collapse out from under us when we cross them, how come we can't reliably write correct desktop applications, such that they don't crash or contain security holes?

The implication is that civil engineers continually meet stringent standards, while software engineers can't achieve the same mastery of craft.
This line of reasoning seems convincing on the surface.
We've all used buggy software, but few of us have ever seen a bridge collapse[^PGH].

Don't worry, us software folks can still save face.
The argument has at least two flaws:

* **It ignores the malicious actor.** Software is under relentless attack, potentially by skilled adversaries. Be it a power user seeking to jailbreak a device or a cyber criminal attempting to monetize a malware campaign. Withstanding these adversaries is a fundamental design requirement for any networked system. Bridges, on the other hand, don't need to be resistant to demolition crews or arsonists by design.

* **It relies on a false equivalence.** Software libraries and frameworks change at a much faster pace than construction materials and architectural techniques. A bridge requires maintenance over a time span measured in years, but a software system can add new functionality in two week sprints. This dynamism only heightens security and reliability risk.

Thus far, this chapter has treated software assurance like building a really sturdy bridge.
Outside of discussing memory safety and general correctness, we haven't addressed what it actually takes to combat malicious actors in the real world (akin to the first flaw above).
We've also implicitly assumed that security is a stationary target (akin to the second flaw above).

## The Bigger Picture

Static analysis and dynamic testing are purely preventative measures.
In reality, most software requires on-going, in-field security support.
Satisfying Rust's pedantic compiler, undergoing manual review, and passing an exhaustive dynamic test suite - these are all just prerequisites to *shipping* high assurance products.

Once running in a production environment, those products need to be *supported* for the entirety of their lifecycles.
**Operational assurance** becomes the name of the game.
We need to be able to respond to malicious actors in a constantly changing environment.

> **Are we talking about OPSEC?**
>
> **Operational Security (OPSEC)** was originally a Vietnam-era military term[^OpSec].
> In modern IT parlance, it refers broadly to measures which prevent sensitive information from leaking to adversaries.
> Consider password management as an example:
>
> * **Re-use the same, easy-to-remember password across multiple sites?**
>
>    * That's **bad OPSEC**, you're leaking valid credentials to otherwise isolated systems.
>
>    * If any single site doesn't use password storage best practices (salted hash[^PassCheat], etc) and gets compromised, an attacker could reuse your password to hijack several of your accounts (assuming email/username reuse).
>
> * **Use a unique, long, and randomly-generated password for each website?**
>
>   * That's **good OPSEC**, you're not leaking credentials.
>
>   * Any single-site breach will have no impact on the security of your other accounts.
>
> **Operational assurance** is a broader concept, with goals beyond confidentiality of sensitive data.
> You can think of OPSEC as a subset of operational assurance.
> Returning to our previous example:
>
> * **Always enable Multi-Factor Authentication (MFA) and review logs in addition to using random, unique passwords?**
>
>   * That's **good operational assurance**. You strengthen authentication, perform historical audits regularly, and protect confidentiality.
>
>   * If you notice login attempts from an IP in an odd geo-location appear in historical data, it could indicate a breach attempt.
>

## Breaking Down Operational Assurance

Operational measures run the gambit from "structured logging"[^Log] to "remote attestation"[^RemoteAttest].
It's difficult to capture the entire spread of tools and technologies, so we'll break it down into three broad categories and provide non-exhaustive examples.

### System Lifecycle

Processes and tools for keeping a product or service up-to-date.
Falling under the relatively new umbrella of "DevSecOps"[^DevSecOps].
Examples include:

* Security scanning in Continuous Integration / Continuous Deployment (CI/CD)
* Encrypted, authenticated over-the-air firmware update
* Versioned, fault-tolerant, distributed infrastructure (e.g. containerized microservices)
* Asset inventory

### Host-based Support

Technologies to protect individual machines, whether clients used by a company's employees or servers running customer-facing services.
Examples include:

* Sandboxing
* Hardened memory allocators
* Endpoint Detection and Response (EDR) tooling
* Application-specific best-practice configuration

### Network-based Support

Technologies to protect corporate networks from remote attacks and limit the movement of attackers that do manage to gain a foothold.
Examples include:

* Secure API gateways
* Web Application Firewalls (WAFs)
* Security Information and Event Management (SIEM) systems
* Virtual Private Network (VPN) infrastructure and Zero-Trust architecture

> **How does operational assurance relate to our library?**
>
> The central focus of this book is writing a robust data structure library.
> Easy to lose sight of, with all the ground we cover in early chapters!
> Toward the end of our journey, we'll gain practical experience in the system lifecycle component of operational assurance.
>
> By developing bindings that make our library usable from other programming languages, namely C and Python, we'll simulate the process of integrating fast, secure Rust components into an existing codebase.
>
> Maybe you have the opportunity to write new features in Rust for performance reasons.
> Or maybe you can incrementally replace security-critical yet memory-unsafe components with Rust equivalents.
> Either way, bindings to Rust code allow a team to "harden" (improve the security posture of) a large system over time.

## Takeaway

This book is biased toward a "product security" perspective.
The goal of this section is to give you a taste of the bigger picture, the "enterprise security" perspective.
Our discussion of software security wouldn't be complete without this broader context.

The next section will expose you to an application deployment option for a native client, an aspect of the system lifecycle category.
We're going to make our `rcli` program free-standing, so that it will reliably run for any end-user.

---

[^PGH]: As an aside, an author of this book actually walked across a Pittsburgh bridge less than 24 hours before [its collapse](https://web.archive.org/web/20220131022216/https://www.nytimes.com/2022/01/28/us/pittsburgh-bridge-collapse-biden.html). Aging and poorly maintained infrastructure is a serious problem in the United States, despite the relative wealth of the country.

[^OpSec]: [*Operations security*](https://en.wikipedia.org/wiki/Operations_security). Wikipedia (Accessed 2022).

[^PassCheat]: [*Password Storage Cheat Sheet*](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html). OWASP Foundation (Accessed 2022).

[^Log]: Logging errors (systems events where some "bad" condition is hit) in a structured format can help you diagnose issues encountered in a production environment. Or more effectively respond to a security incident.

[^RemoteAttest]: Attestation is a process for proving that a specific machine is running a specific set of pre-approved software - not software inserted or modified by an attacker. If you're interested in this topic, you may want to look into what a "Trusted Platform Module" (TPM) does and how it's used by Windows 11.

[^DevSecOps]: [*What is DevSecOps?*](https://www.redhat.com/en/topics/devops/what-is-devsecops). Redhat (2018).
