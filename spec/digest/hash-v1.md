# HashV1 — Digest Foundation, v1

> Status: AgentFlow HashV1 v1 repository normative source (behavior frozen in
> Phase 1; this document materializes the exact normative statements for the
> rule registry in Phase 2B). Where the frozen v1 baseline (`docs/` §24.1)
> conflicts with this document, the baseline wins.
>
> Algorithm- or registry-changing edits require an explicit specification
> compatibility decision and a frozen-corpus update
> (`spec/conformance/digest/`); silent edits are forbidden.

## 1. Positioning

HashV1 is AgentFlow v1's content-digest function for **canonical semantic
objects**. Its inputs are always canonical ACE1 bytes of a machine object, and
its output is always carried by a nominally typed [`DigestRef`](#digest-ref-domain-retained).
HashV1 is not a general-purpose file hasher: repository artifact integrity
(e.g. registry source files) uses plain `SHA256(file bytes)` and is a distinct
concept — see `spec/registry/registry-spec.md`.

<a id="digest-formula-exact-prefix"></a>

## 2. Exact formula

```text
HashV1(domain, payload) = SHA256( "AF" || 0x00 || domain || 0x00 || payload )
```

- The prefix is the exact ASCII bytes `AF` followed by a single NUL byte
  (`0x00`). The prefix must be exactly this constant; it is never derived,
  padded, or versioned.
- The domain is one of the frozen registry names (§4), encoded as its ASCII
  bytes, followed by a single NUL byte.

<a id="digest-domain-separator"></a>

The two NUL bytes are **domain separators**: they unambiguously frame the
domain between prefix and payload, so payloads that themselves contain `AF`,
NUL bytes, or even full valid preimages of other objects can never cause one
(domain, payload) pair to be reinterpreted as another. The framing is part of
the frozen formula; any implementation MUST reproduce it byte-for-byte.

## 3. Algorithm

<a id="digest-sha256-only"></a>

The one and only HashV1 algorithm in v1 is **SHA-256** (32 raw bytes). There
is no algorithm negotiation, no version parameter, and no alternative
algorithm; a `DigestRef` whose algorithm is not `sha256` is invalid
(`digest/unknown-algorithm` in the frozen negative corpus).

## 4. Domain registry

<a id="digest-domain-registry-closed"></a>

The set of HashV1 domains is **closed** and centrally registered in
`spec/tables/hash-domains.json`, mirrored by the `DigestDomain` enum in
`crates/af-core`. Domains are normative strings: implementations must never
invent domains by string concatenation, abbreviate them, or accept a domain
outside the registry (the frozen negative corpus contains
`digest/unknown-domain`). Adding a domain is a registry addition requiring an
explicit specification decision and a table update — never a silent edit.

## 5. DigestRef

A digest identity is a `DigestRef`: `(alg, domain, hex)` carried together —
the domain is **never discarded**, never guessed back, and never conflated
with a bare hex string.

<a id="digest-ref-domain-retained"></a>

A `DigestRef` retains its domain for its whole life. Dropping the domain (or
parsing a digest without knowing its domain) is a modeling error; consumers
that need to compare or re-verify a digest must use the retained domain, not
a caller-supplied one.

<a id="digest-ref-lowercase-hex"></a>

The hex field is exactly 64 **lowercase** hexadecimal characters. Uppercase
hex, wrong lengths, non-hex characters, and unknown algorithms are rejected
at construction (frozen negative corpus `digest/uppercase-hex`,
`digest/short-hex`, `digest/long-hex`, `digest/non-hex`,
`digest/unknown-algorithm`).

## 6. Domain separation invariant

<a id="digest-domain-separation"></a>

For any fixed payload, hashing under two different registry domains yields
two different digests; for any fixed domain, changing one payload byte yields
a different digest. This is a MUST invariant of the formula (not an accident
of SHA-256), and it is frozen as a conformance matrix in
`spec/conformance/digest/domain-separation.json` plus a property test over
the full registry.
