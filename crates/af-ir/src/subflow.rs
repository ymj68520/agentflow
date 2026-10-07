//! Phase 3D subflow interface derivation and CALL_SUBFLOW static checking.
//!
//! A CALL_SUBFLOW pins its target by an exact `SemanticIrDigest` (the
//! Phase 3B contract table already enforces `kind == subflow` and
//! `domain == semantic-ir`). This module closes the loop: the pinned digest
//! is looked up in an explicit, immutable, in-memory [`SubflowIrUniverse`],
//! the target's **derived** port interface is compared against the caller's
//! bindings, and any divergence is a stable compile-time error.
//!
//! # Everything here is derived
//!
//! The target Semantic IR already owns its public surface: `run_parameters`
//! are the inputs, `workflow_outputs` are the outputs, and every declared
//! `ResourceSlot` is a resource requirement (the slot is the contract
//! surface; an unused slot is an authoring defect to fix before publishing,
//! not something this layer hides). [`SubflowInterface`] is a plain Rust
//! projection of those three tables — it never enters AFIDL, the ABI
//! registry or the `SemanticIrDigest`, and no new Hash domain exists.
//!
//! # Port ordinals are external coordinates
//!
//! Input ports enumerate the target's run parameters in `RunParameterId`
//! order, output ports enumerate the workflow outputs in canonical NFC-name
//! order and resource ports enumerate the resource slots in `ResourceSlotId`
//! order — each 1-based, each an independent namespace. An ordinal is a
//! derived interface coordinate, **not** an alias for a target-local id.
//!
//! # Cross-document type equality
//!
//! A `SemanticTypeId` is a document-local DAG coordinate, so numeric
//! equality across caller and target proves nothing (and disqualifies
//! nothing). Ports and bindings are compared as [`StructuralType`] values:
//! primitives by keyword, schema leaves by exact `TypeSchemaRef`
//! (specification id, type name **and** digest), containers recursively.
//! Structural equality suffices; no `SemanticTypeDigest` exists.
//!
//! # Pass ordering
//!
//! ```text
//! SemanticIrDocument::check            (Phase 3B)
//!         ↓ CheckedIrDocument
//! CheckedIrDocument::analyze_dataflow  (Phase 3C)
//!         ↓ SemanticDataflowAnalysis
//! CheckedIrDocument::check_subflow_interfaces  (Phase 3D, this module)
//! ```
//!
//! Phase 3B violations never reach this pass; Phase 3C violations never
//! reach the interface check ([`SubflowIrUniverse::insert`] runs the
//! analysis itself, so an invalid target is uninsertable). A document with
//! no CALL_SUBFLOW passes against an empty universe, and the target's own
//! transitive subflow closure is deliberately not validated here — that is
//! Resolver / LockGraph territory, as is any subflow-graph cycle question.

use crate::check::CheckedIrDocument;
use crate::dataflow::SemanticDataflowAnalysis;
use crate::error::{IrError, kind};
use crate::model::{ComputationBody, ContractRefEntry, SemanticIrDocument};
use af_semantic::{
    DigestRefValue, ExpressionDef, ResourceSlot, SemanticTypeNode, SemanticTypeNodeBody,
    StructuralType, TypeSchemaRef, ValueSlot,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

// ---------------------------------------------------------------------------
// Derived interface
// ---------------------------------------------------------------------------

/// One value port (input or output) of the derived subflow interface.
/// `name` is diagnostic/inspection metadata carried over from the target
/// table; bindings use `port_no` only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubflowValuePort {
    port_no: u32,
    name: String,
    semantic_type: StructuralType,
}

impl SubflowValuePort {
    /// The 1-based external port ordinal.
    pub fn port_no(&self) -> u32 {
        self.port_no
    }

    /// The target-local name (diagnostic metadata, never a wire field).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The port's structurally resolved type.
    pub fn semantic_type(&self) -> &StructuralType {
        &self.semantic_type
    }
}

impl fmt::Display for SubflowValuePort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "port#{} {} {}",
            self.port_no, self.name, self.semantic_type
        )
    }
}

/// One resource requirement port of the derived subflow interface.
/// `source_resource_slot` is the target-local slot id, carried for
/// diagnostics only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubflowResourcePort {
    port_no: u32,
    source_resource_slot: u32,
    requirement_type: TypeSchemaRef,
}

impl SubflowResourcePort {
    /// The 1-based external port ordinal.
    pub fn port_no(&self) -> u32 {
        self.port_no
    }

    /// The target-local `ResourceSlotId` (diagnostic metadata).
    pub fn source_resource_slot(&self) -> u32 {
        self.source_resource_slot
    }

    /// The exact pinned requirement schema.
    pub fn requirement_type(&self) -> &TypeSchemaRef {
        &self.requirement_type
    }
}

impl fmt::Display for SubflowResourcePort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "port#{} slot#{} {}",
            self.port_no, self.source_resource_slot, self.requirement_type
        )
    }
}

/// The derived static interface of one exact target Semantic IR: inputs from
/// `run_parameters`, outputs from `workflow_outputs`, resources from every
/// declared `ResourceSlot`. Derived, deterministic, and never durable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubflowInterface {
    inputs: Vec<SubflowValuePort>,
    outputs: Vec<SubflowValuePort>,
    resources: Vec<SubflowResourcePort>,
}

impl SubflowInterface {
    /// The input ports, in derived port order (`1..=len`).
    pub fn inputs(&self) -> &[SubflowValuePort] {
        &self.inputs
    }

    /// The output ports, in derived port order (`1..=len`).
    pub fn outputs(&self) -> &[SubflowValuePort] {
        &self.outputs
    }

    /// The resource requirement ports, in derived port order (`1..=len`).
    pub fn resources(&self) -> &[SubflowResourcePort] {
        &self.resources
    }
}

/// Expands one document-local `SemanticTypeId` into its finite
/// [`StructuralType`]. The DAG is acyclic (Phase 3A validated), so the
/// recursion is total; the memo only makes repeated expansions cheap.
fn expand_type(
    types: &BTreeMap<u32, &SemanticTypeNode>,
    id: u32,
    memo: &mut BTreeMap<u32, StructuralType>,
) -> StructuralType {
    if let Some(resolved) = memo.get(&id) {
        return resolved.clone();
    }
    let node = types
        .get(&id)
        .expect("every referenced type id exists after Phase 3B validation");
    let resolved = match &node.body {
        SemanticTypeNodeBody::Primitive(primitive) => StructuralType::Primitive(*primitive),
        SemanticTypeNodeBody::Schema(reference) => StructuralType::Schema(reference.clone()),
        SemanticTypeNodeBody::List(inner) => {
            StructuralType::List(Box::new(expand_type(types, inner.raw(), memo)))
        }
        SemanticTypeNodeBody::MapString(inner) => {
            StructuralType::MapString(Box::new(expand_type(types, inner.raw(), memo)))
        }
        SemanticTypeNodeBody::Option(inner) => {
            StructuralType::Option(Box::new(expand_type(types, inner.raw(), memo)))
        }
    };
    memo.insert(id, resolved.clone());
    resolved
}

/// Derives the subflow interface of a validated document. The mapping is
/// frozen: run parameters by `RunParameterId` ascending, workflow outputs by
/// canonical NFC-name bytes, resource slots by `ResourceSlotId` ascending,
/// each enumerated 1-based into its own independent port namespace. Source
/// table order never matters: the canonical keys decide.
pub fn derive_subflow_interface(document: &SemanticIrDocument) -> SubflowInterface {
    let types: BTreeMap<u32, &SemanticTypeNode> = document
        .types
        .iter()
        .map(|node| (node.id.raw(), node))
        .collect();
    let expressions: BTreeMap<u32, &ExpressionDef> = document
        .expressions
        .iter()
        .map(|def| (def.id.raw(), def))
        .collect();
    let mut memo = BTreeMap::new();

    let mut parameters: Vec<_> = document.run_parameters.iter().collect();
    parameters.sort_by_key(|parameter| parameter.id);
    let inputs = parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| SubflowValuePort {
            port_no: u32::try_from(index + 1).expect("interface ports fit u32"),
            name: parameter.name.clone(),
            semantic_type: expand_type(&types, parameter.type_id.raw(), &mut memo),
        })
        .collect();

    let mut outputs_by_name: Vec<_> = document.workflow_outputs.iter().collect();
    outputs_by_name.sort_by_key(|output| {
        unicode_normalization::UnicodeNormalization::nfc(output.name.chars())
            .collect::<String>()
            .into_bytes()
    });
    let outputs = outputs_by_name
        .iter()
        .enumerate()
        .map(|(index, output)| {
            let result_type = expressions
                .get(&output.expression.raw())
                .expect("workflow output expressions exist after Phase 3B validation")
                .result_type_id;
            SubflowValuePort {
                port_no: u32::try_from(index + 1).expect("interface ports fit u32"),
                name: output.name.clone(),
                semantic_type: expand_type(&types, result_type.raw(), &mut memo),
            }
        })
        .collect();

    let mut slots: Vec<_> = document.resources.iter().collect();
    slots.sort_by_key(|slot| slot.id);
    let resources = slots
        .iter()
        .enumerate()
        .map(|(index, slot)| SubflowResourcePort {
            port_no: u32::try_from(index + 1).expect("interface ports fit u32"),
            source_resource_slot: slot.id.raw(),
            requirement_type: slot.requirement_type.clone(),
        })
        .collect();

    SubflowInterface {
        inputs,
        outputs,
        resources,
    }
}

// ---------------------------------------------------------------------------
// Explicit target universe
// ---------------------------------------------------------------------------

/// One validated target: the digest computed **from** the target itself, the
/// derived interface, and the canonical bytes (for the duplicate-insert
/// invariant). Analysis metadata only.
#[derive(Debug, Clone)]
pub struct SubflowTarget {
    digest: DigestRefValue,
    interface: SubflowInterface,
    canonical_bytes: Vec<u8>,
}

impl SubflowTarget {
    /// The exact `DigestRef` (algorithm `sha256`, domain `semantic-ir`).
    pub fn digest(&self) -> &DigestRefValue {
        &self.digest
    }

    /// The derived interface.
    pub fn interface(&self) -> &SubflowInterface {
        &self.interface
    }
}

/// The explicit, immutable-during-checking, in-memory universe of exact
/// subflow targets: `SemanticIrDigest → validated target`. There is no
/// filesystem search, no registry-latest, no network, no package download
/// and no deployment lookup — a digest either is here or the call fails
/// closed.
#[derive(Debug, Clone, Default)]
pub struct SubflowIrUniverse {
    targets: HashMap<DigestRefValue, SubflowTarget>,
}

impl SubflowIrUniverse {
    /// An empty universe. Legal for documents without CALL_SUBFLOW.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts one target. The digest is computed from the target document
    /// itself — callers never pass a trusted digest string. The universe
    /// accepts only fully validated IR: Phase 3B by the
    /// [`CheckedIrDocument`] type, Phase 3C by running the scoped dataflow
    /// analysis here (an invalid target is uninsertable, fail closed).
    ///
    /// Re-inserting the same canonical bytes is idempotent; the same digest
    /// over different bytes is an invariant violation and is rejected, never
    /// silently overwritten.
    ///
    /// # Errors
    /// [`IrError`] from the Phase 3C analysis, the canonical encoding, or
    /// `subflow_target_collision` on a digest/bytes divergence.
    pub fn insert(
        &mut self,
        target: &CheckedIrDocument<'_>,
        module_set: &af_idl::CompiledModuleSet,
    ) -> Result<DigestRefValue, IrError> {
        // Phase 3C validity is a prerequisite, enforced right here.
        let analysis: SemanticDataflowAnalysis = target.analyze_dataflow()?;
        let _ = analysis;
        let canonical_bytes = target.source().canonical_bytes(module_set)?;
        let digest = target.source().digest(module_set)?;
        let key = DigestRefValue {
            algorithm: "sha256".into(),
            domain: digest.domain().as_str().into(),
            hex: digest.hex().to_string(),
        };
        if let Some(existing) = self.targets.get(&key) {
            if existing.canonical_bytes == canonical_bytes {
                return Ok(key);
            }
            return Err(IrError::rule(
                kind::SUBFLOW_TARGET_COLLISION,
                format!(
                    "digest {} is already registered over different canonical bytes",
                    key.hex
                ),
            ));
        }
        let interface = derive_subflow_interface(target.source());
        self.targets.insert(
            key.clone(),
            SubflowTarget {
                digest: key.clone(),
                interface,
                canonical_bytes,
            },
        );
        Ok(key)
    }

    /// The number of registered targets.
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    /// `true` when no target is registered.
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// The registered target for an exact digest, if present. No fallback of
    /// any kind exists.
    pub fn target(&self, digest: &DigestRefValue) -> Option<&SubflowTarget> {
        self.targets.get(digest)
    }

    /// The derived interface for an exact digest, if present.
    pub fn interface(&self, digest: &DigestRefValue) -> Option<&SubflowInterface> {
        self.targets.get(digest).map(|target| &target.interface)
    }

    /// All registered digests, sorted by hex (a conformance projection
    /// ordering only; the universe itself is unordered).
    pub fn digests(&self) -> Vec<&DigestRefValue> {
        let mut digests: Vec<&DigestRefValue> = self.targets.keys().collect();
        digests.sort_by(|a, b| a.hex.cmp(&b.hex));
        digests
    }
}

// ---------------------------------------------------------------------------
// CALL_SUBFLOW static checking
// ---------------------------------------------------------------------------

impl CheckedIrDocument<'_> {
    /// Phase 3D: checks every CALL_SUBFLOW of this document against the
    /// explicitly supplied target universe. `dataflow` must be this
    /// document's own Phase 3C analysis (the pass runs strictly after it).
    ///
    /// For each call, in ascending computation id order: the pinned digest
    /// must resolve exactly in the universe (`unknown_subflow_target`,
    /// no latest fallback); the input, output and resource binding port
    /// sets must exactly equal the derived target port sets
    /// (`subflow_*_port_mismatch`); and every bound type must equal the
    /// target port type structurally (`subflow_*_type_mismatch`).
    ///
    /// # Errors
    /// The first [`IrError`] in that order.
    pub fn check_subflow_interfaces(
        &self,
        dataflow: &SemanticDataflowAnalysis,
        universe: &SubflowIrUniverse,
    ) -> Result<(), IrError> {
        let _ = dataflow; // pass-ordering contract: 3C precedes 3D
        check_subflows(self.source(), universe)
    }
}

fn check_subflows(
    document: &SemanticIrDocument,
    universe: &SubflowIrUniverse,
) -> Result<(), IrError> {
    let types: BTreeMap<u32, &SemanticTypeNode> = document
        .types
        .iter()
        .map(|node| (node.id.raw(), node))
        .collect();
    let expressions: BTreeMap<u32, &ExpressionDef> = document
        .expressions
        .iter()
        .map(|def| (def.id.raw(), def))
        .collect();
    let values: BTreeMap<u32, &ValueSlot> = document
        .values
        .iter()
        .map(|slot| (slot.id.raw(), slot))
        .collect();
    let resources: BTreeMap<u32, &ResourceSlot> = document
        .resources
        .iter()
        .map(|slot| (slot.id.raw(), slot))
        .collect();
    let contracts: BTreeMap<u32, &ContractRefEntry> = document
        .contracts
        .iter()
        .map(|entry| (entry.id.raw(), entry))
        .collect();
    let mut memo = BTreeMap::new();

    let mut computations: Vec<_> = document.computations.iter().collect();
    computations.sort_by_key(|computation| computation.id);
    for computation in computations {
        let ComputationBody::CallSubflow(body) = &computation.body else {
            continue;
        };
        // The contract exists, has kind `subflow` and pins the semantic-ir
        // domain — all Phase 3B rules. Phase 3D adds: the digest must
        // resolve exactly in the supplied universe.
        let contract = contracts
            .get(&body.contract.raw())
            .expect("contract references validated before Phase 3D");
        let Some(target) = universe.target(&contract.digest) else {
            return Err(IrError::rule(
                kind::UNKNOWN_SUBFLOW_TARGET,
                format!(
                    "computation {}: contract {} pins SemanticIrDigest {}, which is not in \
                     the supplied subflow universe (there is no latest fallback)",
                    computation.id, contract.id, contract.digest.hex
                ),
            ));
        };
        let interface = &target.interface;

        // Inputs: exact port set, then per-port structural types.
        let caller_inputs: BTreeSet<u32> = body
            .inputs
            .iter()
            .map(|binding| binding.port_no.raw())
            .collect();
        let expected_inputs: BTreeSet<u32> =
            (1..=u32::try_from(interface.inputs.len()).expect("interface ports fit u32")).collect();
        if caller_inputs != expected_inputs {
            return Err(IrError::rule(
                kind::SUBFLOW_INPUT_PORT_MISMATCH,
                format!(
                    "computation {}: the target interface has input ports {:?}, the call \
                     binds {:?} (missing: {:?}, unexpected: {:?})",
                    computation.id,
                    sorted(&expected_inputs),
                    sorted(&caller_inputs),
                    sorted(
                        &expected_inputs
                            .difference(&caller_inputs)
                            .copied()
                            .collect()
                    ),
                    sorted(
                        &caller_inputs
                            .difference(&expected_inputs)
                            .copied()
                            .collect()
                    ),
                ),
            ));
        }
        let mut inputs: Vec<_> = body.inputs.iter().collect();
        inputs.sort_by_key(|binding| binding.port_no);
        for binding in inputs {
            let actual = expand_type(
                &types,
                expressions
                    .get(&binding.expression.raw())
                    .expect("input expressions exist after Phase 3B validation")
                    .result_type_id
                    .raw(),
                &mut memo,
            );
            let expected = &interface.inputs[(binding.port_no.raw() - 1) as usize];
            if actual != expected.semantic_type {
                return Err(IrError::rule(
                    kind::SUBFLOW_INPUT_TYPE_MISMATCH,
                    format!(
                        "computation {}: input {} expects {}, the bound expression has {}",
                        computation.id,
                        binding.port_no,
                        expected.semantic_type.canonical_string(),
                        actual.canonical_string(),
                    ),
                ));
            }
        }

        // Outputs: the caller's declared output interface is complete —
        // exact port set, then per-port structural types.
        let caller_outputs: BTreeSet<u32> = computation
            .outputs
            .iter()
            .map(|binding| binding.port_no.raw())
            .collect();
        let expected_outputs: BTreeSet<u32> = (1..=u32::try_from(interface.outputs.len())
            .expect("interface ports fit u32"))
            .collect();
        if caller_outputs != expected_outputs {
            return Err(IrError::rule(
                kind::SUBFLOW_OUTPUT_PORT_MISMATCH,
                format!(
                    "computation {}: the target interface has output ports {:?}, the call \
                     declares {:?} (missing: {:?}, unexpected: {:?}); an unconsumed output \
                     stays an unused slot, never a partial contract",
                    computation.id,
                    sorted(&expected_outputs),
                    sorted(&caller_outputs),
                    sorted(
                        &expected_outputs
                            .difference(&caller_outputs)
                            .copied()
                            .collect()
                    ),
                    sorted(
                        &caller_outputs
                            .difference(&expected_outputs)
                            .copied()
                            .collect()
                    ),
                ),
            ));
        }
        let mut outputs: Vec<_> = computation.outputs.iter().collect();
        outputs.sort_by_key(|binding| binding.port_no);
        for binding in outputs {
            let actual = expand_type(
                &types,
                values
                    .get(&binding.value_slot.raw())
                    .expect("output slots exist after Phase 3B validation")
                    .type_id
                    .raw(),
                &mut memo,
            );
            let expected = &interface.outputs[(binding.port_no.raw() - 1) as usize];
            if actual != expected.semantic_type {
                return Err(IrError::rule(
                    kind::SUBFLOW_OUTPUT_TYPE_MISMATCH,
                    format!(
                        "computation {}: output {} expects {}, the declared slot has {}",
                        computation.id,
                        binding.port_no,
                        expected.semantic_type.canonical_string(),
                        actual.canonical_string(),
                    ),
                ));
            }
        }

        // Resources: every declared target slot is part of the interface;
        // exact port set, then exact requirement-schema equality.
        let caller_resources: BTreeSet<u32> = body
            .resources
            .iter()
            .map(|binding| binding.port_no.raw())
            .collect();
        let expected_resources: BTreeSet<u32> = (1..=u32::try_from(interface.resources.len())
            .expect("interface ports fit u32"))
            .collect();
        if caller_resources != expected_resources {
            return Err(IrError::rule(
                kind::SUBFLOW_RESOURCE_PORT_MISMATCH,
                format!(
                    "computation {}: the target interface has resource ports {:?}, the \
                     call binds {:?} (missing: {:?}, unexpected: {:?})",
                    computation.id,
                    sorted(&expected_resources),
                    sorted(&caller_resources),
                    sorted(
                        &expected_resources
                            .difference(&caller_resources)
                            .copied()
                            .collect()
                    ),
                    sorted(
                        &caller_resources
                            .difference(&expected_resources)
                            .copied()
                            .collect()
                    ),
                ),
            ));
        }
        let mut resource_uses: Vec<_> = body.resources.iter().collect();
        resource_uses.sort_by_key(|binding| binding.port_no);
        for binding in resource_uses {
            let actual = &resources
                .get(&binding.resource_slot.raw())
                .expect("resource slots exist after Phase 3B validation")
                .requirement_type;
            let expected = &interface.resources[(binding.port_no.raw() - 1) as usize];
            if actual != &expected.requirement_type {
                return Err(IrError::rule(
                    kind::SUBFLOW_RESOURCE_TYPE_MISMATCH,
                    format!(
                        "computation {}: resource {} expects {}, the bound slot requires {}",
                        computation.id, binding.port_no, expected.requirement_type, actual
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn sorted(set: &BTreeSet<u32>) -> Vec<u32> {
    set.iter().copied().collect()
}
