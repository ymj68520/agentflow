//! The effect algebra: containment (⊑), accumulation (⊕), branch join (⊔)
//! and repetition (n ⊗ ε).
//!
//! All four are pure functions over canonical signatures. ⊤ (unknown)
//! contaminates upward: any composition with an unknown operand is unknown,
//! and ⊤ is contained only in ⊤. The algebra never merges unprovable cases
//! into a smaller safe envelope: whenever a relationship cannot be proven it
//! returns `Unproven` (containment) or widens conservatively (composition) —
//! an envelope that UNDERSTATES possible effects is never produced.

use crate::error::{EffectError, kind};
use crate::model::{
    AtomKey, EffectEntry, EffectGuard, EffectOpenRow, EffectQualifierIdentity, EffectResidual,
    EffectRow, EffectSignature, OccurrenceBounds, ResourceSelector,
};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Containment
// ---------------------------------------------------------------------------

/// Why a containment claim could not be proven. Never silently treated as
/// proven.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnprovenReason {
    /// An operand row is ⊤ (unprovable envelope); ⊤ ⊑ non-⊤ is unprovable.
    UnknownRow,
    /// An open row's residual can never be proven inside a closed row.
    ResidualInsideClosed,
    /// A known entry has no covering entry in the limit.
    EntryUncovered,
    /// A residual ceiling has no covering residual in the limit.
    ResidualUncovered,
}

impl UnprovenReason {
    /// Stable reason token (diagnostics and corpus observations).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            UnprovenReason::UnknownRow => "unknown_row",
            UnprovenReason::ResidualInsideClosed => "residual_inside_closed",
            UnprovenReason::EntryUncovered => "entry_uncovered",
            UnprovenReason::ResidualUncovered => "residual_uncovered",
        }
    }
}

/// The outcome of a containment proof attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proof {
    /// `a ⊑ b` was proven.
    Proven,
    /// NOT PROVEN — the relationship could not be established and must be
    /// treated as absent.
    Unproven(UnprovenReason),
}

impl Proof {
    /// `true` when proven.
    #[must_use]
    pub fn is_proven(self) -> bool {
        matches!(self, Proof::Proven)
    }
}

/// The covering identity of an entry: everything about the atom EXCEPT the
/// scope bindings, which are compared by containment (narrower ⊑ wider).
/// Operations must be exactly equal (there is no operation hierarchy) and
/// qualifier sets must be exactly equal (only equal qualifiers are
/// compatible).
struct CoverKey {
    catalog: String,
    name: String,
    port_no: u32,
    qualifiers: Vec<EffectQualifierIdentity>,
}

impl CoverKey {
    fn of(key: &AtomKey) -> Self {
        CoverKey {
            catalog: key.catalog().to_string(),
            name: key.name().to_string(),
            port_no: key.port_no(),
            qualifiers: key.qualifiers().to_vec(),
        }
    }

    fn matches(&self, other: &Self) -> bool {
        self.catalog == other.catalog
            && self.name == other.name
            && self.port_no == other.port_no
            && self.qualifiers == other.qualifiers
    }
}

fn selector_scope(selector: &ResourceSelector) -> Vec<(String, String)> {
    let mut scope: Vec<(String, String)> = selector
        .scope_bindings
        .iter()
        .map(|binding| (binding.key.clone(), binding.value.clone()))
        .collect();
    scope.sort();
    scope
}

/// `inner`'s scope is narrower than or equal to `outer`'s: every constraint
/// of `outer` appears in `inner` with the exact same value. The empty scope
/// is the widest; only it is contained in nothing.
fn scope_contained_in(inner: &[(String, String)], outer: &[(String, String)]) -> bool {
    outer.iter().all(|binding| inner.contains(binding))
}

/// `a ⊑ b`: the effects `a` may cause all fall within the envelope `b`
/// declares. Matching is deterministic: canonical entry order on both sides,
/// each `a` entry claimed by the first unused `b` slot that covers it, each
/// `b` slot claimed at most once (joint-budget refinement across several
/// `a` entries sharing one covering declaration is a documented proof limit).
#[must_use]
pub fn contains(a: &EffectSignature, b: &EffectSignature) -> Proof {
    // Canonicalize defensively; an invalid operand is simply not provable.
    let (Ok(a), Ok(b)) = (a.canonical(), b.canonical()) else {
        return Proof::Unproven(UnprovenReason::UnknownRow);
    };
    // Everything is contained in ⊤; ⊤ is contained only in ⊤.
    if b.row == EffectRow::Unknown {
        return Proof::Proven;
    }
    if a.row == EffectRow::Unknown {
        return Proof::Unproven(UnprovenReason::UnknownRow);
    }
    let (a_entries, a_residuals) = split(&a.row);
    let (b_entries, b_residuals) = split(&b.row);

    // An open row with residuals is never proven inside a closed row.
    if b_residuals.is_empty() && !a_residuals.is_empty() {
        return Proof::Unproven(UnprovenReason::ResidualInsideClosed);
    }

    let mut known_slots: Vec<Option<(&EffectEntry, CoverKey)>> = b_entries
        .iter()
        .map(|entry| Some((entry, CoverKey::of(&AtomKey::of(&entry.atom)))))
        .collect();
    let mut residual_slots: Vec<Option<&EffectResidual>> = b_residuals.iter().map(Some).collect();
    for entry in a_entries {
        let key = AtomKey::of(&entry.atom);
        let cover = CoverKey::of(&key);
        let scope = key.scope().to_vec();
        if !claim_known(&mut known_slots, &cover, &scope, entry)
            && !claim_residual(&mut residual_slots, &cover, &scope, entry)
        {
            return Proof::Unproven(UnprovenReason::EntryUncovered);
        }
    }
    // Residuals: covered only by residual slots (a known entry covers exactly
    // one operation; a residual allows unknown operations).
    for residual in a_residuals {
        let mut covered = false;
        for slot in &mut residual_slots {
            let Some(candidate) = slot.as_ref() else {
                continue;
            };
            if residual_covered_by(residual, candidate) {
                *slot = None;
                covered = true;
                break;
            }
        }
        if !covered {
            return Proof::Unproven(UnprovenReason::ResidualUncovered);
        }
    }
    Proof::Proven
}

fn residual_covered_by(inner: &EffectResidual, outer: &EffectResidual) -> bool {
    inner.catalog == outer.catalog
        && inner.selector.requirement_port_no == outer.selector.requirement_port_no
        && scope_contained_in(
            &selector_scope(&inner.selector),
            &selector_scope(&outer.selector),
        )
        && outer.bounds.covers(&inner.bounds)
}

fn claim_known(
    slots: &mut [Option<(&EffectEntry, CoverKey)>],
    cover: &CoverKey,
    scope: &[(String, String)],
    entry: &EffectEntry,
) -> bool {
    for slot in slots.iter_mut() {
        let Some((candidate, candidate_key)) = slot.as_ref() else {
            continue;
        };
        if !candidate_key.matches(cover) {
            continue;
        }
        if !scope_contained_in(scope, AtomKey::of(&candidate.atom).scope()) {
            continue;
        }
        if !candidate.bounds.covers(&entry.bounds) {
            continue;
        }
        if !guard_contained(&entry.guard, &candidate.guard) {
            continue;
        }
        *slot = None;
        return true;
    }
    false
}

fn claim_residual(
    slots: &mut [Option<&EffectResidual>],
    cover: &CoverKey,
    scope: &[(String, String)],
    entry: &EffectEntry,
) -> bool {
    for slot in slots.iter_mut() {
        let Some(candidate) = slot.as_ref() else {
            continue;
        };
        if candidate.catalog != cover.catalog
            || candidate.selector.requirement_port_no != cover.port_no
        {
            continue;
        }
        if !scope_contained_in(scope, &selector_scope(&candidate.selector)) {
            continue;
        }
        if !candidate.bounds.covers(&entry.bounds) {
            continue;
        }
        // A residual behaves as an unconditional ceiling; every guard is
        // contained in unconditional.
        *slot = None;
        return true;
    }
    false
}

/// Guard containment: conditional ⊑ unconditional and G ⊑ G are provable;
/// unconditional under a condition and two different conditions are not.
#[must_use]
pub fn guard_contained(inner: &EffectGuard, outer: &EffectGuard) -> bool {
    match (inner, outer) {
        (_, EffectGuard::Unconditional) => true,
        (EffectGuard::Unconditional, EffectGuard::InputEquals(_)) => false,
        (EffectGuard::InputEquals(a), EffectGuard::InputEquals(b)) => a == b,
    }
}

fn split(row: &EffectRow) -> (&[EffectEntry], &[EffectResidual]) {
    match row {
        EffectRow::Closed(entries) => (entries, &[]),
        EffectRow::Open(open) => (&open.known, &open.residual),
        EffectRow::Unknown => (&[], &[]),
    }
}

impl EffectSignature {
    /// The fail-closed containment assertion: `effect_envelope_unprovable`
    /// unless `self ⊑ limit` is PROVEN. This is the surface a future
    /// ComponentContract composition stage calls — never an authorization.
    ///
    /// # Errors
    /// [`EffectError`] with kind `effect_envelope_unprovable`.
    pub fn require_contained_in(&self, limit: &EffectSignature) -> Result<(), EffectError> {
        match contains(self, limit) {
            Proof::Proven => Ok(()),
            Proof::Unproven(reason) => Err(EffectError::new(
                kind::EFFECT_ENVELOPE_UNPROVABLE,
                format!(
                    "containment is not proven: {} inside {} fails ({})",
                    self.classification(),
                    limit.classification(),
                    reason.as_str()
                ),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Accumulation (⊕) — SEQUENCE / PARALLEL / RACE / compensation
// ---------------------------------------------------------------------------

/// `[l1,u1] ⊕ [l2,u2] = [l1+l2, u1+u2]` per atom key. Guards compose
/// conservatively: lower bounds accumulate only across unconditional entries
/// (a guarded entry occurs zero times when its condition is false), differing
/// guards widen to unconditional. ⊤ contaminates. Distinct atoms are kept
/// side by side; same-key entries merge (this is the ONLY merge the algebra
/// performs — declared duplicates are a validation error, not a merge).
///
/// # Errors
/// [`EffectError`] with kind `effect_arithmetic_overflow` when checked
/// occurrence arithmetic overflows; structural kinds if an operand is
/// invalid.
pub fn accumulate(
    a: &EffectSignature,
    b: &EffectSignature,
) -> Result<EffectSignature, EffectError> {
    let (a, b) = (a.canonical()?, b.canonical()?);
    if a.row == EffectRow::Unknown || b.row == EffectRow::Unknown {
        return Ok(unknown());
    }
    let mut merged: BTreeMap<AtomKey, EffectEntry> = BTreeMap::new();
    let (a_entries, a_residuals) = split(&a.row);
    let (b_entries, b_residuals) = split(&b.row);
    for entry in a_entries.iter().chain(b_entries) {
        let key = AtomKey::of(&entry.atom);
        if let Some(existing) = merged.remove(&key) {
            merged.insert(key, accumulate_one(existing, entry)?);
        } else {
            merged.insert(key, entry.clone());
        }
    }
    let residuals = merge_residuals(a_residuals, b_residuals)?;
    Ok(assemble(merged, residuals))
}

fn accumulate_one(
    existing: EffectEntry,
    incoming: &EffectEntry,
) -> Result<EffectEntry, EffectError> {
    let upper = existing.bounds.upper.checked_add(incoming.bounds.upper)?;
    let (lower, guard) = if existing.guard == incoming.guard {
        if existing.guard == EffectGuard::Unconditional {
            // Both always occur: bounds add fully.
            (
                existing.bounds.lower + incoming.bounds.lower,
                existing.guard,
            )
        } else {
            // Same condition: when it holds both occur; when not, neither.
            (0, existing.guard)
        }
    } else {
        // Unrelated conditions: the union of trigger conditions is not
        // expressible, so the merged entry widens to unconditional.
        (0, EffectGuard::Unconditional)
    };
    Ok(EffectEntry {
        atom: existing.atom,
        bounds: OccurrenceBounds { lower, upper },
        guard,
    })
}

// ---------------------------------------------------------------------------
// Branch join (⊔) — SELECT
// ---------------------------------------------------------------------------

/// The conservative envelope of two mutually exclusive branches (exactly one
/// runs). Per atom key: a key declared by BOTH branches keeps the guaranteed
/// minimum lower bound (whichever branch runs delivers at least its own
/// lower, hence at least the minimum) and the covering ceiling; a key
/// declared by only ONE branch counts as zero occurrences when the other
/// branch runs, so its interval is `[0, u]` with its own guard kept. Equal
/// conditions keep their guard; different conditions widen to unconditional
/// (their union is not expressible). ⊤ contaminates.
///
/// # Errors
/// [`EffectError`] with structural or overflow kinds.
pub fn branch_join(
    a: &EffectSignature,
    b: &EffectSignature,
) -> Result<EffectSignature, EffectError> {
    let (a, b) = (a.canonical()?, b.canonical()?);
    if a.row == EffectRow::Unknown || b.row == EffectRow::Unknown {
        return Ok(unknown());
    }
    let mut joined: BTreeMap<AtomKey, (EffectEntry, bool)> = BTreeMap::new();
    let (a_entries, a_residuals) = split(&a.row);
    let (b_entries, b_residuals) = split(&b.row);
    for entry in a_entries {
        joined.insert(AtomKey::of(&entry.atom), (entry.clone(), false));
    }
    for entry in b_entries {
        let key = AtomKey::of(&entry.atom);
        if let Some((existing, _)) = joined.remove(&key) {
            joined.insert(key, (join_one(&existing, entry), true));
        } else {
            joined.insert(
                key,
                (
                    EffectEntry {
                        atom: entry.atom.clone(),
                        guard: entry.guard.clone(),
                        bounds: OccurrenceBounds {
                            lower: 0,
                            upper: entry.bounds.upper,
                        },
                    },
                    false,
                ),
            );
        }
    }
    // A key declared by only one branch has zero occurrences when the other
    // branch runs: its guaranteed lower bound is zero.
    let joined: BTreeMap<AtomKey, EffectEntry> = joined
        .into_iter()
        .map(|(key, (mut entry, on_both_sides))| {
            if !on_both_sides {
                entry.bounds.lower = 0;
            }
            (key, entry)
        })
        .collect();
    // Residual ceilings join the same way: a one-sided residual drops to
    // `[0, u]`, both-sided residuals keep the covering ceiling.
    let mut residuals: BTreeMap<ResidualKey, EffectResidual> = BTreeMap::new();
    for residual in a_residuals.iter().chain(b_residuals) {
        let key = ResidualKey::of(residual);
        if let Some(existing) = residuals.remove(&key) {
            residuals.insert(
                key,
                EffectResidual {
                    catalog: existing.catalog,
                    selector: existing.selector,
                    bounds: OccurrenceBounds {
                        lower: 0,
                        upper: existing.bounds.upper.max(residual.bounds.upper),
                    },
                },
            );
        } else {
            residuals.insert(
                key,
                EffectResidual {
                    catalog: residual.catalog.clone(),
                    selector: residual.selector.clone(),
                    bounds: OccurrenceBounds {
                        lower: 0,
                        upper: residual.bounds.upper,
                    },
                },
            );
        }
    }
    Ok(assemble(joined, residuals.into_values().collect()))
}

/// Branch-join occurrence combination: the guaranteed lower bound across two
/// mutually exclusive branches is zero; the ceiling covers both branches.
fn join_one(existing: &EffectEntry, incoming: &EffectEntry) -> EffectEntry {
    let same_guard = existing.guard == incoming.guard;
    // A lower bound guarded by G is guaranteed only under G (zero when G
    // fails). Case A — identical guard: whichever branch runs delivers at
    // least its own lower bound under that same condition, so the minimum
    // is the tightest guaranteed conditional lower. Case B — different (and
    // not provably equivalent) guards: the merged entry widens to
    // unconditional, and an unconditional positive lower bound would claim
    // occurrences even when NEITHER condition holds, so the lower bound
    // weakens to zero.
    let lower = if same_guard {
        existing.bounds.lower.min(incoming.bounds.lower)
    } else {
        0
    };
    EffectEntry {
        atom: existing.atom.clone(),
        bounds: OccurrenceBounds {
            lower,
            upper: existing.bounds.upper.max(incoming.bounds.upper),
        },
        guard: if same_guard {
            existing.guard.clone()
        } else {
            EffectGuard::Unconditional
        },
    }
}

// ---------------------------------------------------------------------------
// Repetition (n ⊗ ε) — ITERATE
// ---------------------------------------------------------------------------

/// `n ⊗ ε` scales every occurrence interval by the statically proven
/// repetition bounds. Guards are per-occurrence conditions and do not scale.
/// If the repetition upper is unbounded, every non-zero effect ceiling
/// becomes unbounded — an unprovable repetition count never yields a finite
/// envelope.
///
/// # Errors
/// [`EffectError`] with kind `invalid_occurrence_bounds` when `reps` are
/// impossible, `effect_arithmetic_overflow` on checked-multiplication
/// overflow.
pub fn repeat(
    reps: &OccurrenceBounds,
    effect: &EffectSignature,
) -> Result<EffectSignature, EffectError> {
    reps.validate()?;
    let effect = effect.canonical()?;
    let row = match &effect.row {
        EffectRow::Closed(entries) => {
            let scaled = scale_entries(entries, reps)?;
            EffectRow::Closed(scaled)
        }
        EffectRow::Open(open) => {
            let known = scale_entries(&open.known, reps)?;
            let mut residual = Vec::with_capacity(open.residual.len());
            for candidate in &open.residual {
                let bounds = candidate.bounds.checked_mul(reps)?;
                if !bounds.cannot_occur() {
                    residual.push(EffectResidual {
                        catalog: candidate.catalog.clone(),
                        selector: candidate.selector.clone(),
                        bounds,
                    });
                }
            }
            if known.is_empty() && residual.is_empty() {
                EffectRow::Closed(Vec::new())
            } else {
                EffectRow::Open(EffectOpenRow { known, residual })
            }
        }
        EffectRow::Unknown => EffectRow::Unknown,
    };
    Ok(EffectSignature { row })
}

fn scale_entries(
    entries: &[EffectEntry],
    reps: &OccurrenceBounds,
) -> Result<Vec<EffectEntry>, EffectError> {
    let mut scaled = Vec::with_capacity(entries.len());
    for entry in entries {
        let bounds = entry.bounds.checked_mul(reps)?;
        if !bounds.cannot_occur() {
            scaled.push(EffectEntry {
                atom: entry.atom.clone(),
                bounds,
                guard: entry.guard.clone(),
            });
        }
    }
    Ok(scaled)
}

// ---------------------------------------------------------------------------
// Shared assembly
// ---------------------------------------------------------------------------

/// Canonical residual identity: catalog, port and exact scope set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResidualKey {
    catalog: String,
    port_no: u32,
    scope: Vec<(String, String)>,
}

impl ResidualKey {
    fn of(residual: &EffectResidual) -> Self {
        ResidualKey {
            catalog: residual.catalog.clone(),
            port_no: residual.selector.requirement_port_no,
            scope: selector_scope(&residual.selector),
        }
    }
}

type EntryMap = BTreeMap<AtomKey, EffectEntry>;

fn merge_residuals(
    a: &[EffectResidual],
    b: &[EffectResidual],
) -> Result<Vec<EffectResidual>, EffectError> {
    let mut merged: BTreeMap<ResidualKey, EffectResidual> = BTreeMap::new();
    for residual in a.iter().chain(b) {
        let key = ResidualKey::of(residual);
        match merged.remove(&key) {
            Some(existing) => {
                let bounds = existing.bounds.checked_add(&residual.bounds)?;
                merged.insert(
                    key,
                    EffectResidual {
                        catalog: existing.catalog,
                        selector: existing.selector,
                        bounds,
                    },
                );
            }
            None => {
                merged.insert(key, residual.clone());
            }
        }
    }
    Ok(merged.into_values().collect())
}

fn assemble(entries: EntryMap, residuals: Vec<EffectResidual>) -> EffectSignature {
    let known: Vec<EffectEntry> = entries.into_values().collect();
    let row = if residuals.is_empty() {
        EffectRow::Closed(known)
    } else {
        EffectRow::Open(EffectOpenRow {
            known,
            residual: residuals,
        })
    };
    EffectSignature { row }
}

fn unknown() -> EffectSignature {
    EffectSignature {
        row: EffectRow::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EffectRow, OccurrenceCeiling};

    fn sig(json: &str) -> EffectSignature {
        serde_json::from_str(json).expect("valid signature JSON")
    }

    const CATALOG: &str = "agentflow.test.capability-catalog@1";

    fn entry_json(op: &str, port: u32, repo: Option<&str>, lower: u64, upper: u64) -> String {
        let scope: Vec<serde_json::Value> = match repo {
            Some(repo) => vec![serde_json::json!({"key": "repo", "value": repo})],
            None => Vec::new(),
        };
        serde_json::json!({
            "atom": {
                "operation": {"catalog": CATALOG, "name": op},
                "resource_selector": {
                    "requirement_port_no": port,
                    "scope_bindings": scope
                },
                "qualifiers": []
            },
            "bounds": {"lower": lower, "upper": {"finite": upper}},
            "guard": "unconditional"
        })
        .to_string()
    }

    fn closed(entries: &[String]) -> EffectSignature {
        let list = entries.join(",");
        sig(&format!(r#"{{"row": {{"closed": [{list}]}}}}"#))
    }

    #[test]
    fn accumulate_adds_bounds_and_keeps_distinct_atoms() {
        let a = closed(&[entry_json("github.issue.create", 1, Some("foo/bar"), 1, 2)]);
        let b = closed(&[
            entry_json("github.issue.create", 1, Some("foo/bar"), 1, 3),
            entry_json("github.issue.read", 1, Some("foo/bar"), 0, 1),
        ]);
        let sum = accumulate(&a, &b).unwrap();
        let EffectRow::Closed(entries) = &sum.row else {
            panic!("closed expected");
        };
        assert_eq!(entries.len(), 2);
        let create = entries
            .iter()
            .find(|entry| entry.atom.operation.name == "github.issue.create")
            .unwrap();
        assert_eq!(create.bounds.lower, 2);
        assert_eq!(create.bounds.upper, OccurrenceCeiling::Finite(5));
    }

    #[test]
    fn branch_join_is_idempotent_and_zeroes_one_sided_lowers() {
        // A key declared by both branches keeps the guaranteed minimum
        // lower bound; join with itself is the identity.
        let a = closed(&[entry_json("github.issue.create", 1, None, 2, 4)]);
        let joined = branch_join(&a, &a).unwrap();
        let EffectRow::Closed(entries) = &joined.row else {
            panic!("closed expected");
        };
        assert_eq!(entries[0].bounds.lower, 2);
        assert_eq!(entries[0].bounds.upper, OccurrenceCeiling::Finite(4));
        // A one-sided key (absent in the pure branch) drops to [0, u].
        let joined_with_pure = branch_join(
            &a,
            &EffectSignature {
                row: EffectRow::Closed(Vec::new()),
            },
        )
        .unwrap();
        let EffectRow::Closed(entries) = &joined_with_pure.row else {
            panic!("closed expected");
        };
        assert_eq!(entries[0].bounds.lower, 0);
        assert_eq!(entries[0].bounds.upper, OccurrenceCeiling::Finite(4));
    }

    #[test]
    fn unknown_contaminates_and_is_not_pure() {
        let top = sig(r#"{"row": "unknown"}"#);
        let a = closed(&[entry_json("github.issue.create", 1, None, 1, 1)]);
        assert!(!top.is_pure());
        assert_eq!(top.classification(), "unknown");
        let sum = accumulate(&a, &top).unwrap();
        assert_eq!(sum.classification(), "unknown");
        assert_eq!(
            contains(&top, &a),
            Proof::Unproven(UnprovenReason::UnknownRow)
        );
        assert_eq!(contains(&a, &top), Proof::Proven);
    }

    #[test]
    fn containment_needs_scope_and_bounds() {
        let narrow = closed(&[entry_json("github.issue.create", 1, Some("foo/bar"), 1, 2)]);
        let wide = closed(&[entry_json("github.issue.create", 1, None, 0, 5)]);
        let other_op = closed(&[entry_json("github.issue.read", 1, Some("foo/bar"), 1, 2)]);
        assert_eq!(contains(&narrow, &wide), Proof::Proven);
        assert_eq!(contains(&narrow, &narrow), Proof::Proven, "reflexivity");
        assert_eq!(
            contains(&wide, &narrow),
            Proof::Unproven(UnprovenReason::EntryUncovered),
            "an unconstrained scope is not proven inside a narrowed one"
        );
        assert_eq!(
            contains(&other_op, &wide),
            Proof::Unproven(UnprovenReason::EntryUncovered)
        );
        let too_many = closed(&[entry_json("github.issue.create", 1, Some("foo/bar"), 1, 6)]);
        assert_eq!(
            contains(&too_many, &wide),
            Proof::Unproven(UnprovenReason::EntryUncovered)
        );
    }

    fn guarded_json(op: &str, port: u32, expected: &str, lower: u64, upper: u64) -> String {
        serde_json::json!({
            "atom": {
                "operation": {"catalog": CATALOG, "name": op},
                "resource_selector": {
                    "requirement_port_no": port,
                    "scope_bindings": []
                },
                "qualifiers": []
            },
            "bounds": {"lower": lower, "upper": {"finite": upper}},
            "guard": {"input_equals": {
                "port_no": 1,
                "expected": {"string": expected}
            }}
        })
        .to_string()
    }

    fn guard_of(signature: &EffectSignature) -> &EffectGuard {
        let EffectRow::Closed(entries) = &signature.row else {
            panic!("closed expected");
        };
        &entries[0].guard
    }

    #[test]
    fn differing_guards_weaken_the_lower_bound_to_zero() {
        // The task's key case: guard(input=="a") [1,2] joined with
        // guard(input=="b") [1,3] must widen to unconditional [0,3]. The
        // conditions' union is not expressible, and an unconditional
        // positive lower bound would claim occurrences when NEITHER
        // condition holds.
        let a = closed(&[guarded_json("github.issue.create", 1, "a", 1, 2)]);
        let b = closed(&[guarded_json("github.issue.create", 1, "b", 1, 3)]);
        let joined = branch_join(&a, &b).unwrap();
        let EffectRow::Closed(entries) = &joined.row else {
            panic!("closed expected");
        };
        assert_eq!(entries[0].bounds.lower, 0);
        assert_eq!(entries[0].bounds.upper, OccurrenceCeiling::Finite(3));
        assert_eq!(entries[0].guard, EffectGuard::Unconditional);
        // The unconditional-vs-conditional mix weakens the same way: the
        // guarded branch may run with its condition false (zero effects).
        let uncond = closed(&[entry_json("github.issue.create", 1, None, 1, 2)]);
        let guarded = closed(&[guarded_json("github.issue.create", 1, "a", 2, 4)]);
        let mixed = branch_join(&uncond, &guarded).unwrap();
        let EffectRow::Closed(entries) = &mixed.row else {
            panic!("closed expected");
        };
        assert_eq!(entries[0].bounds.lower, 0);
        assert_eq!(entries[0].bounds.upper, OccurrenceCeiling::Finite(4));
        assert_eq!(entries[0].guard, EffectGuard::Unconditional);
    }

    #[test]
    fn identical_conditional_guards_keep_the_tight_lower_bound() {
        // guard(input=="a") [2,4] joined with guard(input=="a") [3,5]
        // keeps the conditional minimum [2,5] under the same condition.
        let a = closed(&[guarded_json("github.issue.create", 1, "a", 2, 4)]);
        let b = closed(&[guarded_json("github.issue.create", 1, "a", 3, 5)]);
        let joined = branch_join(&a, &b).unwrap();
        let EffectRow::Closed(entries) = &joined.row else {
            panic!("closed expected");
        };
        assert_eq!(entries[0].bounds.lower, 2);
        assert_eq!(entries[0].bounds.upper, OccurrenceCeiling::Finite(5));
        assert_eq!(
            guard_of(&joined),
            &EffectGuard::InputEquals(crate::EffectGuardInputEquals {
                port_no: 1,
                expected: af_semantic::PrimitiveConstant::String("a".into()),
            })
        );
    }

    #[test]
    fn repeat_scales_and_zero_repeats_are_pure() {
        let a = closed(&[entry_json("github.issue.create", 1, None, 1, 2)]);
        let three = repeat(&OccurrenceBounds::finite(3, 3).unwrap(), &a).unwrap();
        let EffectRow::Closed(entries) = &three.row else {
            panic!("closed expected");
        };
        assert_eq!(entries[0].bounds.lower, 3);
        assert_eq!(entries[0].bounds.upper, OccurrenceCeiling::Finite(6));
        let zero = repeat(&OccurrenceBounds::finite(0, 0).unwrap(), &a).unwrap();
        assert!(zero.is_pure());
        let unbounded = repeat(
            &OccurrenceBounds::new(0, OccurrenceCeiling::Unbounded).unwrap(),
            &a,
        )
        .unwrap();
        let EffectRow::Closed(entries) = &unbounded.row else {
            panic!("closed expected");
        };
        assert_eq!(entries[0].bounds.upper, OccurrenceCeiling::Unbounded);
    }

    #[test]
    fn overflow_is_an_explicit_error() {
        let big = closed(&[entry_json("github.issue.create", 1, None, 1, u64::MAX - 1)]);
        let overflow = repeat(&OccurrenceBounds::finite(4, 4).unwrap(), &big);
        let error = overflow.unwrap_err();
        assert_eq!(error.kind(), crate::error::kind::EFFECT_ARITHMETIC_OVERFLOW);
    }

    #[test]
    fn retry_stays_conservative() {
        let a = closed(&[entry_json("github.issue.create", 1, None, 1, 2)]);
        let retry = crate::control::retry_envelope(&a, 3).unwrap();
        assert!(!retry.logical_narrowing_proven);
        assert_eq!(retry.attempts.lower, 1);
        assert_eq!(retry.attempts.upper, OccurrenceCeiling::Finite(3));
        let EffectRow::Closed(entries) = &retry.logical.row else {
            panic!("closed expected");
        };
        assert_eq!(entries[0].bounds.upper, OccurrenceCeiling::Finite(6));
    }
}
