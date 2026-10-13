# Phase 4C.1B.3 — Adversarial Conformance

`tools/gen-portable-root-export-revalidation-corpus.py` is an independent Python fixture/oracle generator. It uses portable declared content and independent checked arithmetic/identity expectations; it never calls Rust, reads Rust `Debug`, or obtains Expected from actual projections.

The production runners and comparator-mutation checks are in `crates/af-portable/tests/root_export_revalidation.rs`.

## Oracle closure status

The independent Python generator fully predicts the declared positive semantic view, all D9 operator/closure facts, tested K arithmetic/provenance, and the targeted P30/P31 and capability properties. It does not yet derive every E/Phi/U/F/X evidence row and every RootExport witness field for all 78 inputs. Consequently, the corpus count and passing production runner do not satisfy the task's stricter full-field independent-oracle criterion; this remains an explicit Conformance-Incomplete item.

The 78 unique CaseIds are partitioned as follows:

| Group | Cases |
|---|---:|
| D9 control/opcode and structural variants | 22 |
| D9 RootRecovery and multiplicity | 16 |
| D10, including positive, mapping gaps, P30/P31 | 26 |
| Context/resolver/compiler provenance | 8 |
| Fresh producer/consumer process cases | 6 |
| **Total** | **78** |

The full positive is compared field-by-field against the independent semantic view. Twenty comparator mutations independently alter one projected field each and must be detected. Production input mutations are separate and execute the real decoder, reconstructor, and checker.

Historical conformance remains a separate frozen set: 434 production-differential cases from earlier phases. New D9/D10 cases are not counted as historical cases.
