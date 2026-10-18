# Seven exact source islands / no migration by relabeling

All seven jobs execute freshly in this phase. Each source is checked against
the exact Git tree's blob SHA1 and complete raw SHA-256 map before and after its
own unfiltered Cargo invocation. No job uses the successor Registry or Expected.

| Island | Exact HEAD | Registered CaseIds |
|---|---|---:|
| Edition1 |abc91d533b4d62e91226722e1d1fcaa30bd00883|512|
| Edition2 |51239a0045d7a6fb274bb24cdc8130f2346bf085|51|
| Edition3 |4d3bb59031795d31ecfccf29e3d9f2d89a7d9ce8|173|
| Analysis@4 |875e6e41f16845c78d4f6d31573826c547d48c07|889|
| Analysis@5 |e623aab705c5f8269807201e112a32e67e6869bf|262|
| Analysis@6 |2ad4359410c087462f8366f1ddb5c38bebfef681|254|
| Frozen previous Analysis@7 |8dc4f508c2f97b431053528ab14d2bd9263b3737|534|

The first six reuse existing exact source islands/build caches but execute new
processes and retain new phase logs. Analysis@7 is extracted from its exact Git
archive, not copied from a later mutable tree; its full534-case independent
receipt executes in that island. Cached compilation is not reused test results.
Exact commands/exits, source maps, pass records and auditor logs are retained in
the phase's history-N files and historical-regressions/runtime JSON.

The512 primary Edition1 IDs are82 native RootExport/soundness +229 control/
advanced-control/scope/root-recovery +201 portable codec/reconstruction/revalidation.
Leaf and nested sidecar cases additionally execute in the unfiltered build but
are not added to that historical primary denominator. Edition2's51 IDs are9
Portable +10 native admission +32 registration. The other five exact manifests
directly enumerate173/889/262/254/534 unique IDs. A separate source census verifies
these aggregations and complete ID hashes; no single nonexistent512-ID manifest
is claimed. All manifests belong to their respective frozen source island.

Old CaseIds are never counted as new Binder captures. Previous tests sensitive
to global current identity remain in their exact islands. The current build
keeps IR/control/native regressions and its own structural/Portable staging tests.
Native53/F1 and unchanged51 native rows are additionally independently checked.

The current global registries append4 specifications/13 rules; old66/509 whole
records remain equal. Raw original Registry bytes are retained as analysis7-*
snapshots. The previous relation generator reads those snapshots so mandatory
current generator runs cannot rewrite old Expected using new globals. Existing
old Oracles/Expected/normative sources stay byte-exact.

Canonical DAGs/ranks and complete legal old Nodes stay under old exact editions.
Old borrowed/content-only data lacks new source/site/rank/association conditions
and is not migrated by a name, checksum, reason or numeric-rank reuse. Fresh full
declarations may later be freshly admitted/bound only after separate approval.
Unavailable provenance remains unavailable. No old source has a new label.
