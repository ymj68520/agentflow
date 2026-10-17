# R3-CTX-REL-01 — native relation schema pin binding

Outcome C / P1 / BLOCKING. Related Fact production adaptation stopped. No
Specification, Registry, Descriptor, Rank, native pin type or port verdict is
changed. This is a source-binding gap, not a confirmed false-Proven.

## First unbound field

`RawValidationSubject.context.contents.relation_snapshots.effect_resource.some.pin.schema`

| Source | Actual declared type/content |
| --- | --- |
| model@5 `RelationSnapshotContent.pin.schema` | `af-semantic::TypeSchemaRef`, with specification_id, type_name and schema_digest |
| native `af-contract::RelationCatalogPin.schema` | `String`, storing the schema-family SpecificationId |
| native `RelationCatalogSnapshot<T>` | schema, revision, canonical_bytes, catalog, pin; no construction schema definition/digest |

The registered Context type reuses this same RelationSnapshotContent descriptor
in its three named optional slots. Only effect_resource is dynamically exercised
by this new diagnostic; the other slots are not counted as new executed cases.

Frozen authorities: model@5 types RelationSnapshotContent,
ContractProofContextContents, af-semantic::TypeSchemaRef; AF-SEMANTIC-0198 and
AF-SEMANTIC-0262 require complete actual context contents/association, not a
placeholder. AF-SEMANTIC-0265/0266 require complete source/basis/witness links.
`spec/component/relation-catalogs.md` retains schema/revision/content/checksum
identity; it does not register a native-family-string → exact TypeSchemaRef
conversion or a choice of construction versus current-context schema.

## Real minimal diagnostic

The diagnostic uses public production constructors and genuine capture:

1. Compile the unchanged17 registered AFIDL source inputs as module set A.
2. In memory only, change one relation root field from
   `supported_pairs:list<SupportedEffectResourcePair>` to `list<string>`;
   compile module set B. No normative source file is written.
3. Under A and B, construct an empty EffectResourceCompatibilityCatalog snapshot
   at the same family/revision using `RelationCatalogSnapshot::new`.
4. The root TypeSchemaDigests differ. The complete native snapshots compare
   equal: same typed empty catalog, family, revision, canonical bytes and pin.
   Empty lists encode identically under the two element shapes.
5. Put each snapshot into otherwise equal nine-member current contexts using A.
   Independently call `capture_raw_ports` with the same real occurrence-bound
   input-zero declaration and each context's own same-object role pair.
   Both sealed captures succeed and the existing complete native-subject
   comparison reports equal inputs. This does not infer object aliasing from
   equal contents; the two distinct contexts each satisfy their own alias check.
6. A third context omits the relation module and supplies empty schema/resource
   universes, while retaining the genuine snapshot and complete ComponentContract
   target module. Raw capture still reaches its real zero-port violation.
7. Direct native pin/content metadata fails the registered RelationSnapshotContent
   shape at schema: string cannot satisfy the required TypeSchemaRef record.

All three actual port judgments are Violation with genuine zero witnesses, as
the independent input-derived port semantics require. No reason string is parsed.
These are binding diagnostics, not complete Fact captures or new Proven facts.

`tools/oracle/raw_relation_schema_binding.py` independently compiles A/B and
derives complete TypeSchema ACE1 bytes/pins, catalog bytes/checksum and source
input Expected. Rust Actual is only compared against that independently computed
Expected. Nine diagnostic omission mutations reject; these are not Full-field
Comparator mutations. Default strict exit stays2; named diagnostic mode alone
can exit0 for its much narrower assertion.

## What this proves, and what it does not

This proves that the actual native stored pin is not the formal typed pin, and
construction schema identity cannot in general be recovered from the native
snapshot's stored values. The missing-current-schema case also prevents assuming
that its definition is always an available member of the actual context.

It does NOT prove that current native values should distinguish construction
history. They deliberately compare equal under their existing semantics. It does
NOT assert that every formal projection must encode construction identity.
Rather, frozen registration does not say which source/meaning supplies the
additional type name and digest. Choosing a current/context/global definition
would require an explicit binding rule; treating it as recovered history is
unsupported. A Source/Origin or T4 encoder cannot manufacture the missing rule.

Possible adjudication routes, NOT selected or implemented:

- Preserve the native family pin with an explicitly registered typed-snapshot-kind
  binding, rather than pretending it is already a TypeSchemaRef.
- Retain a construction schema/pin in a separately approved successor source
  contract, with exact legacy isolation and missing-history behavior.
- Define fresh binding/revalidation against the actual current Context schema,
  including missing/mismatched schema rejection and its distinct meaning.

No route, new edition, native type change or Registry amendment is authorized by
this delivery. The required decision is the precise source and meaning of
RelationSnapshotContent.pin.schema for native snapshots, including absent current
schema and unrecoverable construction schema.

## Impact and stop boundary

The complete Subject is referenced by Raw source/target, basis, witness,
contradiction and evidence/primary. Filling those roots with an invented schema
pin would fake nested completeness. None relations in the old fixture do not
establish the missing Some-snapshot binding. No None/empty/default replacement is
made. Remaining750 primary root instances stay unclosed, and full nested/relation
counts remain undetermined. Strict total stays nonzero.

Only diagnostic tests, instance inventories, independent verification and
regression/denial evidence were added. No Raw full Fact constructor, other T3
Producer, T4 normalizer, Scope encoder or capability activation was introduced.
