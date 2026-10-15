# No positive capability path

Audited production paths:

1. current_registered_analysis_edition validates exact source/rule/model bindings and returns only Registered/NotImplemented/NotAuthorized states. Enum variants cannot represent Supported/Activated in this staging; can_issue_capability is false.
2. analyze_root_export_assertion immediately returns AF-SEMANTIC-0244 edition-admission failure before old proof evaluation. Placeholders are not G01 comparison facts.
3. check_root_export_assertion uses that analyzer and separately checks implementation-owned can_issue_capability, no admission failure and Proven before the sole CheckedRootExportCompatibility construction. Its struct field is private; public APIs expose no alternate constructor or mutation path.
4. Portable D4 reconstructs/compares complete registered content, then independently denies support. No SupportedPortableStaticInputs advances to D5–D10; caller fields reject at D0.
5. admit_registered_profile returns NotImplemented for known profiles and UnsupportedModel for unknown/standalone-as-root; it constructs nothing. validate_registered_model_shape returns unit, not a Fact, fragment, CompleteView, Scope or capability.

Tests:10 native admissions,10 real Portable D0–D4 inputs,6 profile denial shapes, plus35 real existing direct root fixture invocations (some component-invalid fixtures reject before root analysis). All paths either reject or return diagnostic-only incomplete reports. Exact same standalone contracts remain valid local substitution evidence only in the unchanged comparator; they cannot become root/body/runtime authority. Historical Edition1 tests use a different exact source build and confer no current-edition capability.

Code audit: root_export.rs private CheckedRootExportCompatibility, current_registered_analysis_edition, analyze_root_export_assertion, check_root_export_assertion; af-portable/support.rs D4; registered_model.rs profile/shape APIs. No unsafe activation, trusted FactSeed, normalizer or new supported enum variant was introduced.
