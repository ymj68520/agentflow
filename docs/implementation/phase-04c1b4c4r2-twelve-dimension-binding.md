# Twelve ordered result dimensions

Full expected descriptors: [dimension-registration.json](../../spec/conformance/g01-successor-staging/dimension-registration.json). Actual comparison is full JSON equality against the registered normative model; the table below is only a readable index.

|Rank|Dimension|Old operand fields (replacement uses corresponding replacement prefix)|Local comparator|Subordinate rules|Frozen local decision|
|---|---|---|---|---|---|
|0|identity|old.contents.contract.component_identity|lineage_equality|none|exact catalog AND name equality:P;otherwise:U;never:V|
|1|io|old.contents.contract.inputs, old.contents.contract.outputs|io_relation|none|equal I/O port sets and pins:P;added required input or missing old output:V;otherwise:U|
|2|failure|old.contents.contract.failures|failure_relation|AF-FAILURE-0008|new nominal failure set subset old:P;otherwise:V|
|3|effects|old.contents.contract.effects|effects_relation|AF-EFFECT-0009|existing exact effect containment:P or typed U;never manufacture V|
|4|resources|old.contents.contract.resources, old.contents.contract.component_identity|resources_relation|AF-RESOURCE-0012, AF-RESOURCE-0013, AF-RESOURCE-0018|existing non-strengthening Ok:P/U;strengthened error:V;other error:U|
|5|information-flow|old.contents.contract.information_flow, old.contents.contract.effect_flow_partitions|flow_relation|AF-INFOFLOW-0017|flow_relaxed error:V;Ok(P) and partitions equal:P;otherwise:U|
|6|usage|old.contents.contract.usage, old.contents.contract.recovery_usage_bindings|usage_relation|AF-USAGE-0013|equal usage and profiles:P;otherwise existing usage_not_increased P/V/U;Err:U;no extra fallback profile gate|
|7|recovery|old.contents.contract.recovery, old.contents.contract.recovery_action_bindings|recovery_relation|AF-RECOVERY-0013|K equivalent and action bindings equal:P;Weakened:V;otherwise:U|
|8|execution|old.contents.contract.execution|execution_combined_relation|AF-EXECUTION-0014|either aspect U:U;else either V:V;else:P;shared Err:U|
|9|execution-feasibility|old.contents.contract.execution|execution_feasibility_relation|AF-EXECUTION-0012|existing feasibility P/V/U;shared Err:U|
|10|execution-security|old.contents.contract.execution|execution_security_relation|AF-EXECUTION-0013|existing security P/V/U;shared Err:U|
|11|cross-dimension|old.contents.contract.component_identity, old.contents.contract.inputs, old.contents.contract.outputs, old.contents.contract.failures, old.contents.contract.effects, old.contents.contract.resources, old.contents.contract.information_flow, old.contents.contract.usage, old.contents.contract.recovery, old.contents.contract.execution, old.contents.contract.effect_flow_partitions, old.contents.contract.recovery_action_bindings, old.contents.contract.recovery_usage_bindings|none: final fold|none|outer fold of eleven distinct non-cross final results;V>RE>U>P;no self edge|

All use AF-CONTRACT-0020 / component-contract-substitution, source=replacement, target=old; exact coordinate, witness, error basis, applicability and dependency edges are individually present. Context invalid blocks all12; full-body equality gives12 reflexive Proven; changed-body non-cross results retain max(Unproven,local), with P<U<RE<V. Cross folds eleven distinct non-cross final results, never itself. Execution combined, feasibility and security remain three dimensions; they are not RootExport's nine dimensions.
