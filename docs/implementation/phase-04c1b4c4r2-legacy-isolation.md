# Three separate proof domains

|Domain|Exact source HEAD|Expected/identity|Cases|
|---|---|---|---|
|Edition1|abc91d533b4d62e91226722e1d1fcaa30bd00883|Exact original source/registry/Expected, owner1|512 historical|
|Edition2|51239a0045d7a6fb274bb24cdc8130f2346bf085|Exact pre-R2 source46/rules436, owner2, shape1/obligation1|51 staging|
|Successor|Code-gate HEAD in gate-report|New g01-successor-staging namespace, owner3, shape2/obligation2|173 registration/staging|

Edition1 source archive: target/phase-04c1b4c3-legacy-abc91d5/source;1732 Git blobs, tree42b548ec66a22813f07b77abb5385707d43256cc. Edition2 source archive: target/phase-04c1b4c4r2-edition2/source;1811 blobs, tree7468f57b5c3acefe2dc941957fb34aadb3e000f9. Each was freshly tested with separate build target/cache and verified against every Git blob before and after execution. These archives validate historical semantics, not the original-directory generator gate.

Edition1 sets:58/72/55/44/35/47/60/63/78=512 (229 core+82 native+201 Portable). Edition2:32 descriptor/rule registrations+10 native admissions+9 Portable admissions=51, plus35 supplemental direct observations. Existing D9 full-field38 and D10 gap/comparator ledgers stay in their original domains, not successor closure.

Old Edition2 generator now uses two exact old INPUT snapshots with independent hardcoded SHA pins. All calculations execute in the original directory; every existing old output is compared and drift/missing output aborts instead of rewriting Expected. This is not copied Expected or an alternate generator directory. Current test-target routing chooses successor_staging; old Edition2 test files and goldens remain byte-exact and execute under the pinned exact build. Historical Edition1 routing remains as previously pinned.
