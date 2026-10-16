# Native subject and exact existing evaluation

## Existing source,not invented task members

Native entry:crates/af-contract/src/recovery_guard.rs::analyze_action_guard_compatibility,AF-CONTRACT-0022 at spec/component/component-contract.md#component-contract-recovery-action-guard. It consumes one ComponentContract plus StaticContractProofContext,not two checked contracts and not an IR/root. Every record site and constructor is listed in the machine audit. Source report fields are all11 ActionGuardCompatibility fields;TypedGuardCountermodel has14 fields;GuardInputAssignment has3. Public getters already expose every needed native field. No production getter change is required for these five existing types.

Pipeline:canonicalize_contract → prove_canonical_contract → check_recovery_effects → native guard records → recovery dimension set_dimension arbitration → overall check_contract decision. A guard record survives in action_guard_compatibility,while set_dimension can retain only a winning dimension reason. Capture must occur at the original branch/before that reduction. Root later clones the native record,then merges guard_verdict;neither operation creates its origin.

## Frozen branch order

1. For each declared recovery_action_binding,look up forward and action by FULL AtomKey in entries:Closed.entries,Open.known,Unknown.empty. Preserve absent ordinals as None,not0. Resolve matching usage by complete action identity;missing E entry or usage returns U/InsufficientEvidence.
2. Compute recovery_forward_upper. Different atoms use declared Forward upper. Same-atom K actions use multiplier sum,divisor1+sum and shared ceiling division;foreign Forward contribution or missing peer profile returns unknown U;checked overflow or recovered upper below declared lower returns V/InsufficientEvidence.
3. Multiply effective Forward upper by activation upper then attempt upper using checked u64/Unbounded algebra,zero first. Overflow returns V/InsufficientEvidence;do not substitute an ordinary finite value or silently raise a limit. The effective bound is component-local,not Root Retry/Iterate multiplicity.
4. Both validate_guard_port calls are evaluated in the two-element Rust array BEFORE the loop selects its first error. Capture both actual outcomes,with forward-before-action arbitration preserved. MissingPort=>V,UnresolvedSchema=>U,TypeMismatch=>V. Unconditional guards need no input port. Port validation precedes zero-action shortcut.
5. If derived action upper is zero:return P/ZeroPossibleActions. Otherwise frozen guard_contained(F,A) proves every guard contained in unconditional,and InputEquals only against exact equal InputEquals. A false containment result alone is not V.
6. On failed containment,construct a type-valid input assignment and re-evaluate Forward=true/Action=false. Valid countermodel=>V/TypedCountermodel;failed construction=>U/InsufficientEvidence. Unit/Null have no alternate primitive in the frozen helper. No branch produces RequiresEvidence.

Current alternate values:Bool negation;Int64 +1 except MAX→MAX-1;Decimal0↔1 choice;String/Bytes empty/nonempty alternate (Bytes empty↔AQ);Timestamp/Duration ±1 microsecond with endpoint guard. Assignments are port-sorted by BTreeMap and preserve full schema pins. The apparent redundant same-port else-if is not a new reachable semantic branch and is not modified.

## Lifecycle and proposal — DRAFT,NOT REGISTERED

Proposed CanonicalNativeContractDeclaration={contract:complete ComponentContract,canonical_bytes:existing ComponentContract ACE1 bytes}. It is not CheckedContractContents and has NO Checked ComponentContractDigest. A canonical-valid declaration that fails AF-CONTRACT-0022 still has a legitimate diagnostic source;never manufacture a Checked token or a pin requiring that token. Lineage catalog/name is retained but is not complete declaration identity.

Proposed NativeRecoveryGuardSubject={owner_compiler:complete implementation-owned compiler content,declaration:CanonicalNativeContractDeclaration,context:ContractProofContextContents,context_roles:NativeGuardContextPartition}. Reuse the actual nine-field contract context definition,without resolver/root objects. Roles analysis/canonical_validation must name the actual same native context instance after genuine canonical validation. The future internal binding borrows that context;equal digests are not proof of aliasing. Revalidate under the intended context rather than reusing an unbound canonical wrapper. No pointer address enters semantic identity. Full cross-process context binding remains G02.

The admission requires successful native canonicalization,not complete Contract Checked. Failed canonicalization gives an explicit earlier admission rejection and NO claimed complete native fragment. The canonical source fact links the actual constituent schema/catalog validation facts,not an editable passed flag. Future negative diagnostic fragments cannot serve as checked operands in substitution or Root.

A future DiagnosticNativeFact can preserve actual evaluation for a canonical source even when check_contract returns rejection. Positive guard alone is also insufficient:the isolated zero-upper row is P but its complete contract is rejected by other constraints. Admission,guard judgment,whole-contract Checked and root authority remain four different claims.
