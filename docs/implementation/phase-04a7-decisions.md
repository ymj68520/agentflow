# Phase 4A.7 Decision Register

| Decision | Frozen choice |
| --- | --- |
| Capability identity | Full `catalog` + nominal `name`; explicit universe only |
| Version policy | Exact recognized versions; no latest or lexical compatibility |
| Attribute vocabulary | NFC key/value pairs checked against the universe vocabulary |
| Quantity model | Unsigned `cpu_millicores`, `memory_bytes`, `gpu_count`; nominal capacity only |
| Sandbox assurance | Closed `basic < strong` descriptor level; not enforcement or attestation |
| Trust matching | Exact nominal `TrustDomainRef`; no ambient or provider inference |
| Region/residency | Separate explicit execution and data-residency lists |
| Runtime profile | Exact family/profile identity plus explicit feature membership |
| Implementation ABI | Exact existing `TypeSchemaRef`; no ComponentContract digest |
| Hard/soft separation | Preferences are closed and ignored by eligibility |
| Profile trust assumption | `StaticExecutorProfile` is trusted input to a conditional static proof |
| Eligibility vs placement | `ProvenAgainstDescriptor` is not availability, admission, or placement |
| Feasibility direction | Old eligible profiles must remain eligible for the new requirement |
| Security direction | Broader placement does not prove security preservation |
| Control aggregation | No blind aggregation across SEQUENCE/PARALLEL or other control forms |
| Authority boundary | X never creates grants, credentials, leases, reservations, or runtime authority |
| Hash boundary | No new HashV1 domain and no X digest |

## Deferred to Phase 4B

ComponentContract composition, F/R/ε/U/K cross-dimension checking, trusted
worker advertisements, admission/attestation, scheduler selection, live
capacity, placement topology, Run ownership region, runtime sandbox
enforcement, and recovery-capability evidence remain deferred.
