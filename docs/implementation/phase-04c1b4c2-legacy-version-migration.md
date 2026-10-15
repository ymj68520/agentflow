# Current-only admission and failure taxonomy

**PROPOSED — NOT REGISTERED. Successor dispatcher: NOT IMPLEMENTED.** Historical support is explicitly **not chosen**.

Validation order: bounded byte preflight/canonical closed decode (D0–D1), reference completeness (D2), internal content/checksum/schema/identity consistency (D3), exact implementation-owned edition support (D4), then fresh D5–D10+G01 checks. Earlier defects win deterministically; do not inspect missing content with an "unsupported" default. These named categories are proposed successor mappings, not claims about every current error string.

| Submitted input, assume earlier layers valid unless stated | Stage / category / successor action |
| --- | --- |
| New edition + exact complete supported46-source tuple/model | D4 admitted, fresh reconstruction/checking; no preaccepted report/capability |
| Old owner label + exact old content | D4 UnsupportedSemantics; old environment may verify separately, no new-build historical support |
| New label + complete self-consistent old registry/source content | D4 InvalidIdentity: known owner label contradicts its registered content identity |
| Old label + complete new content | D4 InvalidIdentity: known label/content binding contradicted |
| Known label + changed registry but internally consistent sources/checksums | D4 InvalidIdentity; exact registry bytes bind identity, not just semantic source equality |
| Same semantic label, changed rule statement/source and consistent checksums | D4 InvalidIdentity; no ordinary static Violation |
| Missing required normative source/content row | D2/D3 IncompleteContent; distinguish absent row from supported empty collection |
| Wrong checksum/digest/domain binding or duplicate conflicting source | D3 InvalidIdentity; plain SHA256 vs HashV1 distinction preserved |
| Unknown obligation edition in report/model content | D4 UnsupportedSemantics after canonical model-token validation |
| Well-formed but unsupported report shape/edition | D4 UnsupportedSemantics; report cannot be treated as a certificate |
| Unknown field/case, bad canonical integer/base64/NFC, duplicate field | D0/D1 InvalidCanonicalEncoding, no normalization/repair |
| Bound/overflow exceeded at any stage | true stage ResourceLimit, no partial capability |
| Unknown owner label with a complete self-consistent tuple | D4 UnsupportedSemantics, never choose latest current |
| Missing G02 getter/fact at future capture (not missing transport content) | G01 capture IncompleteFacts, no CompleteView/Checked |
| Valid supported owner in-process; assertion claims stale compiler | D10 core compiler Unproven,9 blocked records; not a D4 unsupported-owner error |

No old Portable file's label is rewritten. No RuleId/version-only, hash/count-only, or oldExpected-based equality is admissible. Current-only does not promise that a legacy valid fixture will pass new build: the required result is UnsupportedSemantics when its *complete* old tuple is valid and unsupported. Known-owner mixed content is InvalidIdentity instead.

No historical checker/support entry is required merely to classify rejection. After D3 consistency, the singleton current tuple is authoritative for admission. A known legacy label attached to the otherwise exact current registry/source/profile content is a detectable mixed tuple and returns InvalidIdentity; an internally consistent noncurrent legacy tuple returns UnsupportedSemantics without certifying historical validity. Its historical validity is tested only by the separate legacy verifier. A known current label whose complete contents differ is InvalidIdentity. Unknown labels are UnsupportedSemantics. These negative classifications cannot accept a bundle and do not use counts/hashes as identity equality or silently embed a historical validator.

No D10 report exists after D0–D4 rejection. Error categories are not proof verdicts. An early failure must not fabricate six boundary obligations or a full empty successful view. Canonical malformed bytes are not made valid before content checks.

Explicit human approval is needed to change this current-only choice or add historical code/profile support. Isolation uses a separate pinned executable/environment, not a hidden multi-version compiler.
