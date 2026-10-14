# Independent comparator mutations

Scope: the 22 existing D9-Control CaseIds only. D10 independent value comparison remains Pending. Historical CaseIds and Expected are unchanged. Starting HEAD: `5aa4ef7b865b9255a29c83c8375e13803d5e5ce1`.

The previous 68 Actual-baseline self-tests are retained as comparator integrity evidence. This phase adds 42 mutations whose baselines are independent frozen Expected; 32 use original Control sidecars and 10 use independently generated rich input-mutation expectations. One semantic field changes per mutation. Each must fail typed shape/relations or exact comparison and report its deep path. Both-side omission and unregistered additions are also tested.

| Mutation | Expected source | Typed path |
| --- | --- | --- |
| root-node | d9-opcode-invoke | d9_root.root_control.computation |
| root-opcode | d9-opcode-invoke | d9_root.root_control.operator |
| child-order | d9-opcode-sequence | d9_root.root_control.direct_children |
| child-source-node | d9-opcode-sequence | d9_root.root_control.children.0.computation |
| leaf-contract-pin | d9-opcode-invoke | d9_root.root_control.leaf_summary.target.contract_digest.hex |
| leaf-call-site | d9-opcode-invoke | d9_root.root_control.leaf_summary.computation |
| leaf-usage-limit | d9-opcode-invoke | d9_root.root_control.leaf_summary.dimensions.usage.0.upper.finite |
| root-usage-upper | d9-opcode-invoke | d9_root.root_control.usage.value.coordinates.0.bound.upper |
| meter-unit | d9-opcode-invoke | context.usage_meters.0.unit.name |
| meter-identity | d9-opcode-invoke | d9_root.root_control.usage.value.coordinates.0.meter.name |
| recovery-phase | d9-opcode-invoke | d9_root.root_dimensions.usage.value.coordinates.1.phase |
| recovery-source-node | d9-opcode-invoke | d9_root.root_control.recovery_sources.contributions.0.source_node |
| execution-child | d9-opcode-invoke | d9_root.root_control.execution_sources.contributions.0.source_node |
| execution-sandbox | d9-opcode-invoke | d9_root.root_control.leaf_summary.dimensions.execution.sandbox |
| flow-influence | d9-opcode-invoke | d9_root.root_control.information_flow.value.known_may_influence.influence_unknown |
| flow-source | d9-opcode-invoke | d9_root.root_control.information_flow.value.sources.0.source_node |
| context-association | d9-opcode-invoke | d9_root.root_control.proof_context.same_as_root |
| closure-node | d9-opcode-invoke | d9_root.closure.computation_order |
| closure-leaf | d9-opcode-invoke | d9_root.closure.leaf_order |
| closure-occurrences | d9-opcode-invoke | d9_root.closure.known_effect_occurrences_accounted |
| root-k-source-closure | d9-opcode-invoke | d9_root_recovery.source_closure.action_bindings_accounted |
| root-k-verdict | d9-opcode-invoke | d9_root_recovery.overall_verdict |
| scope-owner-child | d9-opcode-scope | d9_root.root_control.scope_boundary.child |
| retry-factor | d9-opcode-retry-four-attempts | d9_root.root_control.operator_semantics.max_attempts |
| retry-delay-order | d9-opcode-retry-four-attempts | d9_root.root_control.operator_semantics.delay_micros_in_order |
| selector-witness | d9-opcode-select | d9_root.root_control.interface.select_decision.exact_case |
| timeout-duration | d9-opcode-timeout | d9_root.root_control.operator_semantics.static_duration_micros |
| subflow-export-pin | d9-opcode-call_subflow | d9_root.root_control.leaf_summary.target.exported_contract_digest.hex |
| compensation-obligation | d9-opcode-compensation_scope | d9_root.root_control.compensation_scope.recovery_blocked_outcome |
| await-verdict-promotion | d9-opcode-await | d9_root.root_control.effects.proofs.0.verdict |
| await-option-presence | d9-opcode-await | d9_root.root_control.leaf_summary.await_summary |
| proof-precondition | d9-opcode-invoke | d9_root.root_control.effects.proofs.0.prerequisites |
| effect-qualifier | spec/conformance/portable-root-export-revalidation/input-mutations/effect-operation-changed.expected.json | d9_root.root_control.effects.value.entries.0.atom.qualifiers.0.member |
| effect-selector | spec/conformance/portable-root-export-revalidation/input-mutations/effect-operation-changed.expected.json | d9_root.root_control.effects.value.entries.0.atom.resource_selector.scope_bindings.0.value |
| effect-guard-value | spec/conformance/portable-root-export-revalidation/input-mutations/effect-guard-changed.expected.json | d9_root.root_control.effects.value.entries.0.guard.input_equals.expected.int64 |
| effect-upper | spec/conformance/portable-root-export-revalidation/input-mutations/effect-operation-changed.expected.json | d9_root.root_control.effects.value.entries.0.bounds.upper.finite |
| effect-occurrence-source | spec/conformance/portable-root-export-revalidation/input-mutations/effect-operation-changed.expected.json | d9_root.root_control.effect_occurrences.0.computation |
| failure-nominal-pin | spec/conformance/portable-root-export-revalidation/input-mutations/failure-member-changed.expected.json | d9_root.root_control.leaf_summary.dimensions.failures.0.type_name |
| resource-slot-coordinate | spec/conformance/portable-root-export-revalidation/input-mutations/resource-slot-changed.expected.json | d9_root.root_control.resources.contributions.0.occurrences.0.mapping.resource_slot |
| resource-type-pin | spec/conformance/portable-root-export-revalidation/input-mutations/resource-slot-changed.expected.json | d9_root.root_control.resources.contributions.0.occurrences.0.mapping.required_type.schema_digest.hex |
| execution-capability-identity | spec/conformance/portable-root-export-revalidation/input-mutations/execution-capability-changed.expected.json | d9_root.root_control.execution_sources.contributions.0.sources.0.requirements.capabilities.0.capability.name |
| failure-source-call-site | spec/conformance/portable-root-export-revalidation/input-mutations/failure-member-changed.expected.json | d9_root.root_control.leaf_summary.computation |
