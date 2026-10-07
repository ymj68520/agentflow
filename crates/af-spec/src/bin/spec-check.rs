//! `spec-check` — validates the AgentFlow specification registries.
//!
//! `cargo run -p af-spec --bin spec-check -- check`
//!
//! Check-only: never writes, rewrites or "fixes" anything. Any validation
//! error exits non-zero so CI can gate on it.

use af_spec::SpecCheck;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("check") | None => {}
        Some(other) => {
            eprintln!("unknown command {other:?}; usage: spec-check [check]");
            return ExitCode::from(2);
        }
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let report = SpecCheck::repo(root).run();

    println!("specifications: {}", report.specifications);
    println!("normative rules: {}", report.rules);
    println!("rules covered: {}", report.covered);
    println!(
        "rules intentionally uncovered: {}",
        report.intentionally_uncovered
    );
    println!("rules missing coverage: {}", report.missing_coverage);
    println!("ABI assignments:");
    println!("  active: {}", report.abi_active);
    println!("  retired: {}", report.abi_retired);
    println!("  reserved: {}", report.abi_reserved);
    println!("AFIDL modules checked: {}", report.modules_checked);
    println!("cross-reference errors: {}", report.errors.len());
    for warning in &report.warnings {
        println!("warning: {warning}");
    }
    for error in &report.errors {
        eprintln!("error [{}]: {error}", error.kind());
    }
    if report.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
