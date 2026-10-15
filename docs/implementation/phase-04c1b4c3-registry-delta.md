# Audited registry and freeze delta

| Surface | Before | After | Actual constraint |
| --- | --- | --- | --- |
| Specification rows |43|46|all43 old full records equal,+3 new |
| Rule rows |404|436|all404 old full records equal,+32 new |
| Reference-covered /exception /missing |403/1/0|435/1/0|mechanical resolvable associations;not full G01 algorithm closure |
| Active ABI /AFIDL /HashV1 domains |775/17/14|775/17/14|old bytes/meanings unchanged |
| Historical CaseIds |512|512|58+72+55+44+35+47+60+63+78 |
| Old conformance files |1144|1144 unchanged|raw-byte equality verified |
| Selected /applied edition attribution |16/84|20/138|new exact edition;neither list is transitive closure |

| SpecificationId | Normative source | Full plain SHA256 |
| --- | --- | --- |
| agentflow.computation.proof-obligation.semantics@1 | spec/computation/proof-obligation-semantics.md | c8d4cf135c28db286b0a598c3fb69814dece936fcf888ce3e46296a336186f5c |
| agentflow.computation.proof-report.model@1 | spec/tables/proof-report-model-v1.md | e23ca47c05efe9f7ec22177011982ac47ca0cdc8642173de56a90e6e29a93099 |
| agentflow.portable.proof.inputs.semantics@2 | spec/portable/portable-proof-inputs-v2.md | 751b46630198e82b99e58eb9ee31ab7c95edd373a1c938e2c1edd470f54dc8c0 |

Exact old row equality is asserted before registry generation and checked in real Rust conformance,not inferred from counts. Every old normative source binding/content is unchanged. New rule rows are exactly [0197–0228](phase-04c1b4c3-rule-allocation.md),including all32 case refs,statements and anchors. No old optional metadata is dropped.

Authorized code changes:edition metadata/admission and constructor gate,Portable current-only staging/error taxonomy,test routing and new scoped tests,explicit frozen historical generator inputs/guards,new data generator/materializer. No Typed Fact producer,normalizer,G02/G03 fix,D10 full oracle,wire schema or digest algorithm/domain change occurred.

The whole registry file byte identity changes intentionally. New compilers/fixtures bind the new full bytes;old exact input identities do not equal the new identity and are never relabeled.
