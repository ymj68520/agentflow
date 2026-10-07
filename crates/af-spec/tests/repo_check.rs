//! Whole-repository validation: `SpecCheck::repo` must pass over the real
//! AgentFlow repository, and the coverage invariants must hold (no machine-
//! testable MUST rule without coverage; the intentionally-uncovered set
//! equals the frozen exception list; totals stay unfrozen so additive
//! registry growth does not create churn).

use af_spec::SpecCheck;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn repo_report() -> af_spec::Report {
    SpecCheck::repo(repo_root()).run()
}

#[test]
fn the_real_repository_validates_cleanly() {
    let report = repo_report();
    assert!(
        report.is_ok(),
        "spec-check must pass on the real repository: {:#?}",
        report.errors
    );
    assert!(
        report.warnings.is_empty(),
        "unexpected warnings: {:?}",
        report.warnings
    );
}

#[test]
fn every_machine_testable_must_rule_is_covered() {
    let report = repo_report();
    assert_eq!(
        report.missing_coverage, 0,
        "no rule may sit in missing_coverage"
    );
    assert!(
        report.rules >= 40,
        "the rule registry unexpectedly shrank below its Phase 2B baseline"
    );
    // covered + intentionally_uncovered must account for every rule.
    assert_eq!(
        report.covered + report.intentionally_uncovered,
        report.rules
    );
}

#[test]
fn intentionally_uncovered_set_matches_the_frozen_exception_list() {
    // The frozen known set (spec/registry/coverage-summary.json) is enforced
    // byte-for-byte by the validator itself; here we pin the human-readable
    // invariant: exactly the architectural ACE1 schema-directed rule.
    let report = repo_report();
    assert_eq!(report.intentionally_uncovered, 1);
}

#[test]
fn the_registry_schema_self_hosting_holds() {
    let report = repo_report();
    // Both AFIDL modules compile and the ABI registry matches them exactly:
    // machine-primitives (aliases only, no rows) + specification-registry
    // (50 numbered members, all active).
    assert!(report.modules_checked >= 2, "spec/idl modules went missing");
    assert!(
        report.abi_active >= 50,
        "the self-hosted registry numbering went missing"
    );
    assert_eq!(
        report.abi_retired, 0,
        "no ABI number is retired in Phase 2B"
    );
    assert_eq!(report.abi_reserved, 0, "nothing is reserved in Phase 2B");
}
