//! Fail-closed v1 component substitution proof.

use crate::proof::{CheckedComponentContract, DimensionProof, ProofVerdict};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityReport {
    pub dimensions: Vec<DimensionProof>,
}

impl CompatibilityReport {
    #[must_use]
    pub fn overall(&self) -> ProofVerdict {
        if self
            .dimensions
            .iter()
            .any(|item| item.verdict == ProofVerdict::Violation)
        {
            ProofVerdict::Violation
        } else if self
            .dimensions
            .iter()
            .any(|item| item.verdict == ProofVerdict::RequiresEvidence)
        {
            ProofVerdict::RequiresEvidence
        } else if self
            .dimensions
            .iter()
            .any(|item| item.verdict == ProofVerdict::Unproven)
        {
            ProofVerdict::Unproven
        } else {
            ProofVerdict::Proven
        }
    }
}

/// Checks whether `replacement` may substitute for `old` under the v1
/// provable subset. Exact semantic equality is Proven; required input/output
/// breaks are violations; other changed dimensions remain Unproven until a
/// directional proof relation is implemented for that specific pair.
pub fn check_substitution(
    replacement: &CheckedComponentContract,
    old: &CheckedComponentContract,
) -> CompatibilityReport {
    let new = &replacement.contract;
    let previous = &old.contract;
    if !std::ptr::eq(replacement.context, old.context) {
        return CompatibilityReport {
            dimensions: dimensions(
                ProofVerdict::Unproven,
                "substitution inputs were checked under different proof contexts",
            ),
        };
    }
    if new == previous {
        return CompatibilityReport {
            dimensions: dimensions(ProofVerdict::Proven, "identical checked semantic bodies"),
        };
    }
    let mut result = dimensions(
        ProofVerdict::Unproven,
        "changed dimension has no v1 positive proof for this pair",
    );
    if new.component_identity == previous.component_identity {
        set(
            &mut result,
            "identity",
            ProofVerdict::Proven,
            "exact component lineage identity",
        );
    } else {
        set(
            &mut result,
            "identity",
            ProofVerdict::Unproven,
            "component lineage differs; publication/version is not a lineage proof",
        );
    }
    let old_inputs: BTreeSet<_> = previous.inputs.iter().map(|port| port.port_no).collect();
    let new_inputs: BTreeSet<_> = new.inputs.iter().map(|port| port.port_no).collect();
    let old_outputs: BTreeSet<_> = previous.outputs.iter().map(|port| port.port_no).collect();
    let new_outputs: BTreeSet<_> = new.outputs.iter().map(|port| port.port_no).collect();
    if old_inputs == new_inputs
        && old_outputs == new_outputs
        && previous.inputs.iter().all(|old| {
            new.inputs
                .iter()
                .any(|item| item.port_no == old.port_no && item.type_ref == old.type_ref)
        })
        && previous.outputs.iter().all(|old| {
            new.outputs
                .iter()
                .any(|item| item.port_no == old.port_no && item.type_ref == old.type_ref)
        })
    {
        set(
            &mut result,
            "io",
            ProofVerdict::Proven,
            "exact equal port sets and exact nominal TypeSchemaRef pins",
        );
    } else if new_inputs.difference(&old_inputs).next().is_some()
        || old_outputs.difference(&new_outputs).next().is_some()
    {
        set(
            &mut result,
            "io",
            ProofVerdict::Violation,
            "replacement adds a required input or removes a required output",
        );
    } else {
        set(
            &mut result,
            "io",
            ProofVerdict::Unproven,
            "port set or exact nominal pin differs",
        );
    }
    set(
        &mut result,
        "failure",
        if af_failure::contains(&new.failures, &previous.failures) {
            ProofVerdict::Proven
        } else {
            ProofVerdict::Violation
        },
        "closed nominal failure-row containment",
    );

    let effect_verdict = match af_effect::contains(&new.effects, &previous.effects) {
        af_effect::Proof::Proven => ProofVerdict::Proven,
        af_effect::Proof::Unproven(_) => ProofVerdict::Unproven,
    };
    set(
        &mut result,
        "effects",
        effect_verdict,
        "exact epsilon containment; open and unknown rows remain unproven",
    );

    let namespace = |identity: &crate::model::ComponentIdentity| {
        format!("{}/{}:resource", identity.catalog, identity.name)
    };
    let resource_verdict = match af_resource::compatibility::requirement_not_stronger(
        &new.resources,
        &namespace(&new.component_identity),
        &previous.resources,
        &namespace(&previous.component_identity),
        replacement.context.resources,
    ) {
        Ok(af_resource::compatibility::ResourceCompatibilityProof::Proven) => ProofVerdict::Proven,
        Ok(af_resource::compatibility::ResourceCompatibilityProof::Unproven(_)) => {
            ProofVerdict::Unproven
        }
        Err(error)
            if error.kind() == af_resource::error::kind::RESOURCE_REQUIREMENT_STRENGTHENED =>
        {
            ProofVerdict::Violation
        }
        Err(_) => ProofVerdict::Unproven,
    };
    set(
        &mut result,
        "resources",
        resource_verdict,
        "new resource requirement must not strengthen the old candidate predicate",
    );

    let flow_result = af_flow::compatibility::flow_not_relaxed(
        &new.information_flow,
        &previous.information_flow,
        replacement.context.flow,
    );
    let flow_verdict = match flow_result {
        Err(error) if error.kind() == af_flow::error::kind::FLOW_CONTRACT_RELAXED => {
            ProofVerdict::Violation
        }
        Ok(af_flow::compatibility::FlowCompatibilityProof::Proven)
            if new.effect_flow_partitions == previous.effect_flow_partitions =>
        {
            ProofVerdict::Proven
        }
        Ok(
            af_flow::compatibility::FlowCompatibilityProof::Proven
            | af_flow::compatibility::FlowCompatibilityProof::Unproven(_),
        )
        | Err(_) => ProofVerdict::Unproven,
    };
    set(
        &mut result,
        "information-flow",
        flow_verdict,
        "flow non-relaxation uses explicit labels and exact sinks; selector partitions must be unchanged",
    );

    let usage_verdict = if new.usage == previous.usage
        && new.recovery_usage_bindings == previous.recovery_usage_bindings
    {
        ProofVerdict::Proven
    } else {
        match af_usage::compatibility::usage_not_increased(
            &new.usage,
            replacement.context.usage_meters,
            &previous.usage,
            replacement.context.usage_meters,
        ) {
            Ok(af_usage::algebra::UsageProof::Proven) => ProofVerdict::Proven,
            Ok(af_usage::algebra::UsageProof::Violation) => ProofVerdict::Violation,
            Ok(af_usage::algebra::UsageProof::Unproven(_)) | Err(_) => ProofVerdict::Unproven,
        }
    };
    set(
        &mut result,
        "usage",
        usage_verdict,
        "usage upper bounds must not increase; per-action recovery profiles must remain equal",
    );

    let recovery_result = af_recovery::compatibility::recovery_not_weakened(
        &new.recovery,
        &previous.recovery,
        replacement.context.recovery,
    );
    let recovery_verdict = match recovery_result {
        Ok(af_recovery::compatibility::RecoveryCompatibilityProof::Proven)
            if new.recovery_action_bindings == previous.recovery_action_bindings =>
        {
            ProofVerdict::Proven
        }
        Ok(af_recovery::compatibility::RecoveryCompatibilityProof::Weakened) => {
            ProofVerdict::Violation
        }
        Ok(
            af_recovery::compatibility::RecoveryCompatibilityProof::Proven
            | af_recovery::compatibility::RecoveryCompatibilityProof::Unproven,
        )
        | Err(_) => ProofVerdict::Unproven,
    };
    set(
        &mut result,
        "recovery",
        recovery_verdict,
        "recovery guarantees use their exact K relation and full action bindings",
    );

    if let Ok(proof) = af_execution::compatibility::execution_compatibility(
        &new.execution,
        &previous.execution,
        replacement.context.execution,
    ) {
        set(
            &mut result,
            "execution-feasibility",
            map_execution_verdict(proof.feasibility),
            "placement feasibility direction",
        );
        set(
            &mut result,
            "execution-security",
            map_execution_verdict(proof.security),
            "security-preservation direction",
        );
        set(
            &mut result,
            "execution",
            map_execution_verdict(proof.combined()),
            "combined execution compatibility preserves both directions",
        );
    } else {
        set(
            &mut result,
            "execution-feasibility",
            ProofVerdict::Unproven,
            "execution feasibility cannot be compared from the explicit universes",
        );
        set(
            &mut result,
            "execution-security",
            ProofVerdict::Unproven,
            "execution security cannot be compared from the explicit universes",
        );
        set(
            &mut result,
            "execution",
            ProofVerdict::Unproven,
            "execution compatibility is unproven",
        );
    }
    CompatibilityReport { dimensions: result }
}

fn map_execution_verdict(
    verdict: af_execution::compatibility::CompatibilityVerdict,
) -> ProofVerdict {
    match verdict {
        af_execution::compatibility::CompatibilityVerdict::Proven => ProofVerdict::Proven,
        af_execution::compatibility::CompatibilityVerdict::Violation => ProofVerdict::Violation,
        af_execution::compatibility::CompatibilityVerdict::Unproven => ProofVerdict::Unproven,
    }
}

fn dimensions(verdict: ProofVerdict, reason: &str) -> Vec<DimensionProof> {
    [
        "identity",
        "io",
        "failure",
        "effects",
        "resources",
        "information-flow",
        "usage",
        "recovery",
        "execution",
        "execution-feasibility",
        "execution-security",
        "cross-dimension",
    ]
    .into_iter()
    .map(|dimension| DimensionProof {
        dimension,
        verdict,
        rule_id: "AF-CONTRACT-0020",
        reference: None,
        reason: reason.into(),
        prerequisites: vec![],
    })
    .collect()
}
fn set(items: &mut [DimensionProof], dimension: &'static str, verdict: ProofVerdict, reason: &str) {
    if let Some(item) = items.iter_mut().find(|item| item.dimension == dimension) {
        if verdict_rank(verdict) > verdict_rank(item.verdict) {
            item.verdict = verdict;
            item.reason = reason.into();
        }
    }
    let overall = if items
        .iter()
        .any(|item| item.verdict == ProofVerdict::Violation)
    {
        ProofVerdict::Violation
    } else if items
        .iter()
        .any(|item| item.verdict == ProofVerdict::RequiresEvidence)
    {
        ProofVerdict::RequiresEvidence
    } else if items
        .iter()
        .any(|item| item.verdict == ProofVerdict::Unproven)
    {
        ProofVerdict::Unproven
    } else {
        ProofVerdict::Proven
    };
    if let Some(item) = items
        .iter_mut()
        .find(|item| item.dimension == "cross-dimension")
    {
        item.verdict = overall;
        item.reason = "all required substitution dimensions must prove".into();
    }
}

fn verdict_rank(verdict: ProofVerdict) -> u8 {
    match verdict {
        ProofVerdict::Proven => 0,
        ProofVerdict::Unproven => 1,
        ProofVerdict::RequiresEvidence => 2,
        ProofVerdict::Violation => 3,
    }
}
