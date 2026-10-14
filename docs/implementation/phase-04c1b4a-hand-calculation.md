# Hand-derived cross-checks

Scope: the 22 existing D9-Control CaseIds only. D10 independent value comparison remains Pending. Historical CaseIds and Expected are unchanged. Starting HEAD: `5aa4ef7b865b9255a29c83c8375e13803d5e5ce1`.

These literal expected values were derived from the formal rules, without Rust output. The ten control-algebra operators plus INVOKE/AWAIT/CALL_SUBFLOW cover all thirteen opcodes. Additional vectors check exact HANDLE residual Phi, missing versus zero Usage, U64 overflow, Retry x Iterate, Sequence + Retry, Select + Handle, Parallel + Race and Iterate + CompensationScope. Inputs, intermediate factors and expected results are preserved below.

## sequence

```json
{
  "vector_id": "sequence",
  "input_effect_bounds": [
    [
      1,
      2
    ],
    [
      2,
      3
    ]
  ],
  "operator": "sequence",
  "body": {},
  "expected_lower": 1,
  "expected_upper": 5,
  "expected_usage_upper": 5
}
```

## select

```json
{
  "vector_id": "select",
  "input_effect_bounds": [
    [
      1,
      2
    ],
    [
      2,
      3
    ]
  ],
  "operator": "select",
  "body": {},
  "expected_lower": 0,
  "expected_upper": 3,
  "expected_usage_upper": 3
}
```

## parallel

```json
{
  "vector_id": "parallel",
  "input_effect_bounds": [
    [
      1,
      2
    ],
    [
      2,
      3
    ]
  ],
  "operator": "parallel",
  "body": {
    "mode": "all"
  },
  "expected_lower": 3,
  "expected_upper": 5,
  "expected_usage_upper": 5
}
```

## race

```json
{
  "vector_id": "race",
  "input_effect_bounds": [
    [
      1,
      2
    ],
    [
      2,
      3
    ]
  ],
  "operator": "race",
  "body": {},
  "expected_lower": 0,
  "expected_upper": 5,
  "expected_usage_upper": 5
}
```

## iterate

```json
{
  "vector_id": "iterate",
  "input_effect_bounds": [
    [
      1,
      2
    ]
  ],
  "operator": "iterate",
  "body": {
    "while": {
      "max_iterations": "4"
    }
  },
  "expected_lower": 0,
  "expected_upper": 8,
  "expected_usage_upper": 8
}
```

## retry

```json
{
  "vector_id": "retry",
  "input_effect_bounds": [
    [
      1,
      2
    ]
  ],
  "operator": "retry",
  "body": {
    "delays": [
      "0",
      "1"
    ]
  },
  "expected_lower": 1,
  "expected_upper": 6,
  "expected_usage_upper": 6
}
```

## timeout

```json
{
  "vector_id": "timeout",
  "input_effect_bounds": [
    [
      1,
      2
    ]
  ],
  "operator": "timeout",
  "body": {},
  "expected_lower": 1,
  "expected_upper": 2,
  "expected_usage_upper": 2
}
```

## handle

```json
{
  "vector_id": "handle",
  "input_effect_bounds": [
    [
      1,
      2
    ],
    [
      2,
      3
    ]
  ],
  "operator": "handle",
  "body": {
    "clauses": []
  },
  "expected_lower": 1,
  "expected_upper": 5,
  "expected_usage_upper": 5
}
```

## scope

```json
{
  "vector_id": "scope",
  "input_effect_bounds": [
    [
      1,
      2
    ]
  ],
  "operator": "scope",
  "body": {},
  "expected_lower": 1,
  "expected_upper": 2,
  "expected_usage_upper": 2
}
```

## compensation-scope

```json
{
  "vector_id": "compensation-scope",
  "input_effect_bounds": [
    [
      1,
      2
    ]
  ],
  "operator": "compensation_scope",
  "body": {},
  "expected_lower": 1,
  "expected_upper": 2,
  "expected_usage_upper": 2
}
```

## exact-handle-failure

```json
{
  "vector_id": "exact-handle-failure",
  "protected": [
    {
      "specification_id": "hand.failure@1",
      "type_name": "A",
      "schema_digest": {
        "algorithm": "sha256",
        "domain": "type-schema",
        "hex": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      }
    }
  ],
  "handled": [
    {
      "specification_id": "hand.failure@1",
      "type_name": "A",
      "schema_digest": {
        "algorithm": "sha256",
        "domain": "type-schema",
        "hex": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      }
    }
  ],
  "handler": [
    {
      "specification_id": "hand.failure@1",
      "type_name": "B",
      "schema_digest": {
        "algorithm": "sha256",
        "domain": "type-schema",
        "hex": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      }
    }
  ],
  "expected": [
    {
      "specification_id": "hand.failure@1",
      "type_name": "B",
      "schema_digest": {
        "algorithm": "sha256",
        "domain": "type-schema",
        "hex": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      }
    }
  ]
}
```

## retry-times-iterate

```json
{
  "vector_id": "retry-times-iterate",
  "leaf_upper": 2,
  "retry_attempts": 3,
  "iterations": 4,
  "expected_upper": 24
}
```

## missing-is-not-zero

```json
{
  "vector_id": "missing-is-not-zero",
  "declared_zero": 0,
  "missing": [],
  "expected_state": "unknown"
}
```

## u64-overflow

```json
{
  "vector_id": "u64-overflow",
  "max": 18446744073709551615,
  "checked_add": 1,
  "checked_multiply": 2,
  "expected": "OverflowError"
}
```

## sequence-plus-retry

```json
{
  "vector_id": "sequence-plus-retry",
  "inputs": [
    [
      1,
      2
    ],
    [
      2,
      3
    ]
  ],
  "attempts": 3,
  "expected_lower": 1,
  "expected_upper": 9
}
```

## select-plus-handle

```json
{
  "vector_id": "select-plus-handle",
  "protected_upper": 2,
  "handler_upper": 3,
  "other_branch_upper": 4,
  "expected_lower": 0,
  "expected_upper": 5
}
```

## parallel-plus-race

```json
{
  "vector_id": "parallel-plus-race",
  "loser_retained": true,
  "expected_lower": 1,
  "expected_upper": 9
}
```

## iterate-plus-compensation-scope

```json
{
  "vector_id": "iterate-plus-compensation-scope",
  "forward_retained": true,
  "no_action_declared": true,
  "iterations": 4,
  "expected_upper": 8
}
```

## invoke-leaf

```json
{
  "vector_id": "invoke-leaf",
  "declared_effect": "closed empty",
  "declared_failure": [],
  "declared_usage": {
    "forward": 0,
    "recovery": 0
  },
  "source_instances": 1,
  "rule": "AF-SEMANTIC-0112/0117"
}
```

## await-leaf

```json
{
  "vector_id": "await-leaf",
  "adapter_contract": "absent",
  "expected_effect_failure_usage_flow": "Unknown",
  "external_influence": "unknown_external_influence",
  "rule": "AF-SEMANTIC-0122"
}
```

## call-subflow-leaf

```json
{
  "vector_id": "call-subflow-leaf",
  "target_and_export": "exact declaration pins",
  "body_conformance": "requires_evidence",
  "rule": "AF-SEMANTIC-0118/0119/0120"
}
```

## source-multiset-and-per-child-r-x-k

```json
{
  "vector_id": "source-multiset-and-per-child-r-x-k",
  "same_contract": true,
  "child_ids": [
    2,
    3
  ],
  "expected_source_instances": 2,
  "requirements": "retained per child",
  "rule": "AF-SEMANTIC-0138/0139/0140/0141"
}
```

## race-flow-decision

```json
{
  "vector_id": "race-flow-decision",
  "content_authority": "none",
  "expected_decision_influence_unknown": true,
  "expected_state": "partial",
  "rule": "AF-SEMANTIC-0132/0136"
}
```

## scope-static-resource-and-runtime-obligations

```json
{
  "vector_id": "scope-static-resource-and-runtime-obligations",
  "parent_declared_child_resources": [],
  "resource": "proven",
  "authority": "unproven",
  "budget": "unproven",
  "settlement": "requires_evidence",
  "rule": "AF-SEMANTIC-0147/0148"
}
```
