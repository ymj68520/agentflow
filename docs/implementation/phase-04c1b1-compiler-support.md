# Phase 4C.1B.1 — Current Compiler Support Profile

D4 accepts only a full match with the implementation-owned current content
tuple: both registries, all four closed tables, every registered source row,
the static version token, ten role entries, role dependencies, schema pins
and Profile A attribution. It also checks each source's plain SHA-256 and its
registry path/checksum association at D3.

The full source inventory has 43 current rows. The static RootExport identity
continues to project the existing sixteen normative source IDs and exact
84 applied rows (22 AF-CONTRACT and 62 AF-SEMANTIC). Profile A's listed rows
must equal that applied set. Profiles B and C stop at D4 with
UnsupportedAttributionProfile; F03 remains OPEN/P2.

| Role | Profile declaration | Implementation status in this static operation |
| --- | --- | --- |
| static_analysis | defined at agentflow.root-export-checker@1 | Implemented |
| contract_validation | defined at agentflow.root-export-checker@1 | Implemented |
| semantic_ir_validation | defined at agentflow.root-export-checker@1 | Implemented |
| dataflow | defined at agentflow.root-export-checker@1 | Implemented |
| leaf_resolution | defined at agentflow.root-export-checker@1 | Implemented |
| control_algebra | defined at agentflow.root-export-checker@1 | Implemented |
| root_recovery | defined at agentflow.root-export-checker@1 | Implemented |
| root_export | defined at agentflow.root-export-checker@1 | Implemented |
| plan_lowering | model_missing:not_formalized | ModelMissing |
| runtime_semantics_profile | model_missing:not_formalized | ModelMissing |

Only the first eight roles are required for this static operation. A caller's
role labels do not establish that checking ran. Historical dispatch/upcasting
is unsupported; no old proof can be refreshed by replacing its version string.
The post-registration manifest is a new full content identity even though
the static checker token and its 84-rule projection remain the same.

The production result says D0-D4 current-static inputs are supported. It does
not claim that context reconstruction, Checked IR/contracts, resolver
reconstruction, RootRecovery or RootExport checking ran.
