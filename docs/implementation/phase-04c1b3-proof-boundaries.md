# Phase 4C.1B.3 — Proof Boundaries

A successful D10 result means only static RootExport compatibility under the exact current checked inputs. The process-local compatibility capability is returned only by the formal checker and is not represented in Portable Bundle bytes.

The scoped return type is `PortableStaticProofAnalysis` in `crates/af-portable/src/pipeline.rs`; static/runtime surface restrictions are enforced by `crates/af-portable/tests/static_surface.rs`.

The portable analysis projection is not a capability and cannot be rehydrated into one. No certificate-verification API, positive-subproof cache, origin approval, publisher authentication, or rule-complete attribution is introduced.

Body conformance and implementation conformance remain RequiresEvidence. Publisher authentication and runtime authority remain NotEstablished. The implementation does not create EffectGrant, ResourceHandle, BudgetReservation, EffectReceipt, UsageReceipt, worker admission, recovery command, deployment, plan, journal, scheduler, or runtime behavior.
