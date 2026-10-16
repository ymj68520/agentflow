# Complete fourteen-field registered countermodel

Historical owner: agentflow.computation.proof-report.model@1; native type: af-contract::TypedGuardCountermodel; successor key is the exact historical owner/name, not a current-owner same-name replacement. Full recursive descriptors and source contents are in the selected-import manifest. Every original field maps to exactly one successor required field and every successor required field maps back to one original getter. The name/order bijection has14 unique entries;types/options and all recursive payloads are compared as full descriptors.

|Field|Exact native type|Getter / required successor member|
|---|---|---|
|source_contract|ComponentContract|source_contract() → source_contract|
|source_component|ComponentIdentity|source_component() → source_component|
|recovery_action|RecoveryActionIdentity|recovery_action() → recovery_action|
|forward_entry_ordinal|usize|forward_entry_ordinal() → forward_entry_ordinal|
|action_entry_ordinal|usize|action_entry_ordinal() → action_entry_ordinal|
|forward_guard|EffectGuard|forward_guard() → forward_guard|
|declared_action_guard|EffectGuard|declared_action_guard() → declared_action_guard|
|forward_bounds|OccurrenceBounds|forward_bounds() → forward_bounds|
|activation_upper|OccurrenceCeiling|activation_upper() → activation_upper|
|attempt_upper|OccurrenceCeiling|attempt_upper() → attempt_upper|
|action_upper|OccurrenceCeiling|action_upper() → action_upper|
|assignments|Vec<GuardInputAssignment>|assignments() → assignments|
|reachability|GuardCountermodelReachability|reachability() → reachability|
|related_rule_id|&'static str|related_rule_id() → related_rule_id|

GuardInputAssignment retains port_no:u32,type_ref:TypeSchemaRef,value:PrimitiveConstant,including exact schema pins and primitive payload. Zero ordinals are legitimate values,not absent. Native forward_bounds is the effective argument passed to the countermodel builder;declared bounds remain separately retained in subject/trace. Native14 has NO action-binding ordinal/context field;those belong to occurrence/subject and are not silently added to the historical type.

reachability is exactly WellTypedComponentInputDomain,not Root-global reachability;related_rule_id retains its exact native string value AF-CONTRACT-0022 (no invented nominal Rust RuleId wrapper). Whole source_contract remains the complete thirteen-field canonical declaration,not a Checked-only contract/pin or hash. The Int64 negative actual has port1=Int64(1),Forward true/action false,and full Contract rejection;its complete eleven-field record and fourteen-field nested witness are admitted by the new metadata descriptor. Shape admission is not semantic witness validation or Checked authority.

The diagnostic presentation adapter only maps actual canonical declaration field spelling/numeric wire presentation to the existing metadata probe format and checks closed enum spellings. It is NOT a G01 canonical encoder,Fact producer,normalizer,oracle Expected or Scope constructor. Raw before/after getter observations remain byte-identical and preserved separately. Every required field is also exercised by a missing-field mutation;independent G1 vectors remain Capture Not Executed.

