# G2-A — registration / activation separation

Baseline: `8dc4f508c2f97b431053528ab14d2bd9263b3737`.
This checkpoint precedes Registry mutation and successor number allocation.

| Surface | Exact current enforcement | Registration-only decision |
|---|---|---|
| Native edition | `af-computation/src/root_export.rs`: EditionSupport has only NotImplemented; Activation and Release have only NotAuthorized | adding sources/types cannot introduce a supported/authorized state |
| Native analyzer | `analyze_root_export_assertion` returns before proof evaluation when `can_issue_capability()` is false | retain early refusal |
| Direct/private checked boundary | `check_root_export_assertion` additionally checks current edition, no admission failure and Proven before the sole private constructor | retain all three guards and private report field |
| Alternate internal Context | existing fixture calls analyzer and direct checker with selected Context | current-library default/alternate diagnostic remains a separate refusal gate |
| Portable routing | `af-portable/src/support.rs`: exact current version, full content/tuple/roles/tables, then separate capability refusal | add an exact successor profile, never whitelist it as implemented |
| Default D0 / bounded D4 | `DecoderLimits::default` versus explicit bounded tests | limits unchanged; each failure stage must execute independently |
| Publication / release | native release enum cannot represent authorization; Portable cannot return Supported inputs past refused D4 | no publication/runtime path added |
| ModelType / ModelRef | current relation shape API is model@6; frozen legacy shape API is model@5 | add a distinct structural catalogue/API; never relabel those catalogues as successor data |
| Compiler identity | registered sources and selected attributed contents are checked independently of activation | update complete successor content identity only after closed type/rule review |

Conclusion: **registration is separable from activation**. Registration may be
Registered while production binder, full Raw capture, normalizer and Scope stay
NotImplemented/incomplete, and all positive capability paths remain denied.

Explicit implementation dependency: relation_descriptor_integrity currently
compares frozen model@6 header fields with mutable current constants. Successor
registration must not use that old-catalogue check as its own integrity check.
Legacy metadata ownership must remain explicit; no A equality, C lookup/validation
algorithm, native Source association or port predicate is changed to achieve this.
All old complete-edition tests execute from their exact source archives, not with
successor Registry contents. Current staging adds only non-authoritative checks.

G2-A permits proceeding to SB mapping and source-derived site census. It does not
assert the later successor gates have passed or authorize a production binder.
