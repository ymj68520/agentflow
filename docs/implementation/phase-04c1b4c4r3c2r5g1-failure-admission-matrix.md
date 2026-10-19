# 未来失败关闭设计：复用现有闭合类型

以下左列只是失败**含义**，不是新 Variant。确切 Registered 类型为 StructuralFailureDomain、
StructuralFailureReason、StructuralStage、StructuralBindingFailure。现有 payload 包含
domain/reason/stage/site/expected_type/observed_type/first_input_path/second_input_path。
应保留原始输入、具体 Slot/Owner 路径及冲突两端；不得 parse Rust reason string 代替字段。

| 候选失败含义 | 已有可复用 domain / reason | Stage / 区分 |
| --- | --- | --- |
| MissingNativeRelationSource | IncompleteFacts / missing_node | source_admission；Some 所需真实 A 缺失，不等于 None |
| InvalidNativeSourceAssociation | InvalidFacts / source_association | source_admission；实际 Owner/Slot/Context 不符 |
| WrongNominalSourceType | InvalidFacts / invalid_child | pre_capture_binding 或消费阶段；已知错误 target；未知 type 则 UnsupportedModel/unknown_nominal_type |
| MissingRegisteredSourceSite | IncompleteContent / missing_site | registration_admission/pre_capture_binding；未定义模型不得补站点 |
| MissingGSourceReference | IncompleteFacts / missing_node | c_validation/consumer；不发完整 Fact |
| UnreachableSourceNode | InvalidFacts / invalid_interning | pre_capture_binding；不是补根 |
| InvalidSourceDagRank | InvalidFacts / rank_dependency | pre_capture_binding；缺 rank 来源另为 IncompleteFacts/missing_rank_source |
| SourceContextMismatch | InvalidFacts / source_association | source_admission 或 c_validation；equal bytes 不修复 |
| CrossEditionSourceMismatch | UnsupportedModel / cross_edition | registration_admission；Portable 外层仍依原 InvalidIdentity/UnsupportedSemantics taxonomy |
| InvalidCSourceReference | InvalidFacts / invalid_child | c_validation；domain/type/slot/carrier 均须核对 |
| G/C 输出反向进入输入 | InvalidFacts / input_output_backedge | pre_capture_binding |
| Resource bound | ResourceLimit / resource_bound | 实际触及阶段，不返回截断成功 |
| 生产阶段尚不可用 | NotImplemented / production_not_implemented | raw_fact_capture 或既定阶段，不假装 complete checker 已执行 |

复用闭合含义已足够；不申请新 failure Variant，具体 site/type/path applicability 应在后继
正式登记中精确实例化。本轮既不扩展 payload，也不修改生产失败返回值。
当前 C 自身 NotReached/Missing/Ambiguous/FamilyMismatch/RevisionMismatch/Found/Invalid/Valid
仍按 AF-SEMANTIC-0291 区分：Source 拒绝后 NotReached；真实 Native CompiledModuleSet 已拒绝
重复/循环，Ambiguous 属 malformed metadata 前置阶段，不编造 native lookup 分支。
RevisionMismatch 仅实际提供 expected pin 时适用；当前七例 API 传 None，不虚构该分支。
Found 是中间态；Valid 必须完整 encode/decode/native equality/byte equality；None 为 absent。

这些 failure 不属于 port predicate：不得重写 PositiveFresh/Zero/Duplicate Verdict，不改
input-before-output/稳定排序/原始与排序 ordinal。Some/Missing 保留实际 Zero/Violation，
禁止完整 Raw Fact；缺证据不是空证据，也不能变成 C Valid、Unproven 结果或 RootExport。
