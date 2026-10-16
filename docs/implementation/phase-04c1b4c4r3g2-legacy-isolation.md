# Three exact frozen regression domains

|Edition|Exact source HEAD|CaseId namespace|
|---|---|---|
|1|abc91d533b4d62e91226722e1d1fcaa30bd00883|512 historical (58/72/55/44/35/47/60/63/78)|
|2|51239a0045d7a6fb274bb24cdc8130f2346bf085|51 original staging|
|3|4d3bb59031795d31ecfccf29e3d9f2d89a7d9ce8|173 original successor staging|

Fresh git archives under ignored target/phase-04c1b4c4r3g2-edition{1,2,3}/source are verified against every Git blob:1732/1811/1902 files respectively. Each cargo test --all job builds from that exact manifest/source/registry/Expected domain,with independent target/cache. Pre/post source and compiler/registry receipts are retained;actual completion belongs in gate evidence,not inferred from these counts.

Current-only Cargo test routing uses new native_guard_staging tests. Old Edition2/3 test source and Expected files are unchanged;old173 execution is the exact Edition3 job,not current registry recomputation. gen-substitution-successor-staging now consumes SHA-pinned exact Edition3 registry inputs so original-directory regeneration cannot overwrite/relabel old outputs. It still executes in the original repository. This is not a mirror substitute for generator gates.

