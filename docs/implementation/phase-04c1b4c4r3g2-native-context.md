# Nine-member native context and two roles

ContractProofContextContents retains compiled_modules,type_schemas,effect_catalogs,resource_types,flow_catalogs,usage_meters,recovery_operations,execution_universes,relation_snapshots. Resource/flow/usage/recovery/execution universes and complete relations retain their full typed nested payloads,not counts. There is no native resolver_snapshot placeholder.

analysis rank0 and canonical_validation rank1 each occur exactly once,in the same nonempty partition class: [[analysis,canonical_validation]]. Metadata rejects split,reversed,duplicate and missing roles. This is a shape/partition check,not proof that caller data corresponds to one object. Actual lifetime-bound native object identity remains mandatory;equal bytes or hashes do not prove aliasing. Addresses are not serialized. Complete cross-process Scope remains Pending G02.

