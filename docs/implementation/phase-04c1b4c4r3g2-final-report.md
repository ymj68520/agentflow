# G2 — Forty-seven-point final acceptance report

Registered staging only. Actual final generator completion and final full HEAD are delivery-time facts;this document does not manufacture them. Read with gate-evidence.json,legacy-evidence.json,registration-audit.json and the original-directory generator log/maps.

1. 起始完整 HEAD 4d3bb59031795d31ecfccf29e3d9f2d89a7d9ce8；完整稳定代码门禁 HEAD ff04beb2020e49d2214e25bbe0ab7d50616e2aa4；最终完整 HEAD 在交付时读取，避免文件自引用。
2. 正式注册可安全暂存。Registered、NotImplemented、NotAuthorized、release NotAuthorized 独立；无可调用 supported=true 路由，不满足 Atomic Activation Dependency。
3. 独立 native_recovery_guard Profile，正式源 agentflow.component.native-recovery-guard.semantics@1；不是 Root/Substitution/Preflight/exact_k_identity 别名。
4. 七个正式 Case 依 rank0–6：canonical_source、context_association、action_sources、action_bounds、guard_inputs、guard_containment、guard_evaluation；全部 contract_validation/static_declaration。仅前两项为 Native mandatory_preflight，不改变 Root 原五项。
5. 六个非 Context Clause 使用 native_guard；context_association 复用 direct.proof_context；Kind 为 native_recovery_guard_evidence 或 proof_context，Dimension 为 supporting(精确 PredicateId)。
6. NativeRecoveryGuardSubject 完整四字段 owner_compiler、declaration、context、context_roles；输入不包含生成结果或 Checked-only pin。
7. CanonicalNativeContractDeclaration 包含完整十三字段 ComponentContract 与原始 canonical_bytes；需真实 canonical 验证。负例可以 canonical-valid 但 whole-check rejected；无历史 raw 来源时只允许明确重新验证，不伪造历史。
8. Context 保留 compiled_modules/type_schemas/effect_catalogs/resource_types/flow_catalogs/usage_meters/recovery_operations/execution_universes/relation_snapshots 九成员；analysis0/canonical_validation1 同实际对象。分区形状检查不等于对象同一性证明，完整 Scope Pending G02。
9. Forward/Action E occurrence 保留 ordinal+完整 EffectEntry；Usage occurrence 保留 ordinal+完整 RecoveryUsageBinding；另有独立 action_binding_ordinal/full action identity，不混淆三类 ordinal。
10. Resolution 闭合为 not_queried/absent/present(T)，分别物化 Effect/Usage/Ceiling/Port；present ordinal0 是合法值，不是缺失。
11. EffectRowKind 为 closed/open_known/unknown；MemberPath 的已知 E 选择只允许 closed/open_known。Unknown 不能制造已知空 E。
12. IR-free Source/Target/Coordinate 精确指向 Native subject/binding/真实 E、usage、port/member；不注入 caller IR、root node、虚构 resolver。
13. Native Origin 来自 canonical declaration 的真实 recovery_action_bindings[i] 和 forward/action reference 应用；Case/role/member 区分来源；时间、UUID、Rust 行、调用次数和 Root observation path 不构成 Origin。
14. NG-SOURCES→AF-SEMANTIC-0249：完整 AtomKey 的两个 E lookup 均保留实际结果，再保留 Usage 查询；不从 reason 还原缺失、不将缺失当 pure/unconditional/zero。
15. NG-BOUNDS→0250：五种 typed failure 为 same_atom_product_overflow、same_atom_sum_overflow、same_atom_divisor_overflow、same_atom_lower_conflict、action_upper_overflow，全部完整 payload；顺序 trace、declared/effective bounds、同 Atom peers、零/Unbounded/overflow 规则精确，无 Root 倍率。
16. NG-INPUTS→0251：unconditional_not_required/valid/missing_port/unresolved_schema/type_mismatch；valid 保留完整 port/guard/SchemaEntry/StructuralType，失败 step 区分 resolve/expand，不伪造解析成功。
17. 两个 port helper eager 执行；选中的第一项 Forward-before-Action 错误控制原 verdict，不对 helper verdict 求 max；port 验证在 zero shortcut 之前。
18. NG-CONTAINMENT→0252：真实 guard_contained true→P、false→U；zero/unavailable 跳过时无虚构比较；false 不是 Typed Countermodel 或 V。
19. NG-EVALUATION→0253 注册十二 Branch：missing_entries/missing_usage/unisolated_forward/same_atom_bounds_failure/action_upper_overflow/missing_port/schema_unresolved/type_mismatch/zero_potential/frozen_containment/typed_countermodel/no_countermodel。
20. 原生 ActionGuardCompatibility 完整十一字段及四 basis 均保留；reason 仅 diagnostic。Native algorithm evaluated 不等于 containment kernel called；InsufficientEvidence 也可能是结构 V，没有新增 RequiresEvidence 候选。
21. 五类历史 Guard Type 确实在 model@1 声明：ActionGuardCompatibility、GuardCompatibilityBasis、TypedGuardCountermodel、GuardInputAssignment、GuardCountermodelReachability；不称历史未注册。
22. 精确 historical owner/name/full source/full descriptor 受限导入：五根原87项闭包，额外 SchemaEntry 支持依赖10项，共97项。Unknown owner/type、未选择 old Scope、current owner 同名/qualified spoof 和未知历史字段 token 拒绝；无历史 Owner 白名单。
23. Countermodel 全14字段：source_contract/source_component/recovery_action/forward_entry_ordinal/action_entry_ordinal/forward_guard/declared_action_guard/forward_bounds/activation_upper/attempt_upper/action_upper/assignments/reachability/related_rule_id；双向字段/type 映射见 field-bijection 与完整 manifest。
24. GuardInputAssignment 完整 port_no/type_ref/value 三字段，schema pin 与 primitive 值不丢失；不是 exists/count/hash/Debug 代替。
25. NativeGuardWitness 区分 full input_countermodel、missing_port、constant_type_mismatch、bound_failure、非递归 propagated_failure；结构 V 不伪造输入赋值；缺必需 witness 的未来 Capture 必须 IncompleteFacts。
26. Native prerequisite 六种 canonical_source/context_association/action_sources_ready/action_bounds_ready/guard_input_valid(role)/containment_outcome(expected)，保留真实 supporting fact、status、original stage/source/origin、EvidenceRef.native_guard_member 与全部 consumers；false query 不改 U 为 P。
27. Native Fact、Root call-site observation、RootRecovery aggregate 三层身份分离；一个 Native Origin 可多 Root edges，不重复 Native 来源或 Root 倍率；F02 Checked-source body/context/pin 关联保持。
28. Native Subject 是 Scope INPUT；records/assignments/witness/facts 是 OUTPUT。NativeGuardFragment 完整非 Root typed shape 已注册，但 normalizer/完整 Scope 未实现；各自有限无环 DAG rank space，不把生成 witness 放回 Scope。
29. 新 Analysis@4、Report@3、Obligation@3、Model@3 与 NativeProfile semantics@1 经实际冲突检查分配；完整 compiler identity 包含54所有源、完整 registries/tables、28 selected sources 与163 applied rules；后者不是传递依赖闭包。
30. 实际新增4 Specification、16 RuleId（AF-SEMANTIC-0245–0260）；总54/468，reference-covered467、intentional exception1、missing0。NG-* 草案义务均映射到实际 Source/anchor/Rule/descriptor/conformance。
31. 旧50 Specification 与452 Rule 按完整 JSON 对象逐条相等，全部旧规范源字节保持；model@1/@2 内容与顺序冻结，新图仅在新 tuple 扩展。
32. 新增889个 Registration Conformance CaseId：844 shape/owner/field/variant/parent +20 独立手写 roster/field/order laws +16 Rule +9 Portable。526项 field/variant inventory 覆盖全部 Native 与97导入依赖；不是889个 Cargo test 或 Typed Capture Case。
33. 真实14场景在精确起始/当前构建逐字节相同，canonical 均成功；合法正例保持 Checked，Typed/结构负例仍拒绝。完整实际11-field record（含负例14-field witness）通过新元数据形状检查；不冒充新 Capture 或独立语义 Oracle。
34. Edition1 精确 abc91d533b4d62e91226722e1d1fcaa30bd00883 新鲜执行512 CaseIds；382 Cargo passes、0 failed/ignored，144子进程选择过滤非跳过；compiler16 selected/84 applied、registry43/404；1732 Git blobs 回归后精确。
35. Edition2 精确51239a0045d7a6fb274bb24cdc8130f2346bf085 新鲜执行51 CaseIds；335 Cargo passes、0 failed/ignored/filtered；compiler20/138、registry46/436；1811 Git blobs 回归后精确。
36. Edition3 精确 G2 起始4d3bb59031795d31ecfccf29e3d9f2d89a7d9ce8 新鲜执行173 CaseIds；336 Cargo passes、0 failed/ignored/filtered；compiler24/149、registry50/452；1902 Git blobs 回归后精确，不用新全局 Registry 重算。
37. New Profile 全入口拒绝正向 RootExport：direct analyzer/checker/唯一 private construction boundary/Portable D4；Native-as-root UnsupportedModel、complete profile admission NotImplemented。默认大源 scalar D0 ResourceLimit 与显式有界 D4 另测。
38. G01 Native Typed Producer/T3 Collector/Normalizer/complete fragment 均 NOT IMPLEMENTED；G02 OPEN/P1、G03 OPEN/P2。G1 48向量仍 Capture Not Executed，新 Capture CaseIds=0。
39. F01/F02 CLOSED；F03/F04 OPEN/P2；本地 well-typed input reachability 不升级 Root-global，rule references 不等于依赖闭包。
40. B01/B02 与 B03–B07 继续 BLOCKING；未因注册或形状成功关闭后续证明义务。
41. 新 Findings：G2-LIMIT-01 完整 bundle 超默认 scalar 上限，安全早拒绝，生产默认未放宽；选择历史字段 token 的闭包检查已补齐；无确认 false-Proven/P0。初始 authoring/test/diagnostic 失败日志保留，未拼接作通过门禁。
42. ABI775、AFIDL17、HashV1 domains14、SemanticIR/ComponentContract 意义、digest goldens、所有旧 Expected/D9 full-field/D10 gap ledger/512/51/173资产冻结。
43. 完整稳定 ff04beb 源码门禁 fmt/clippy/test/spec-check 全退出0；337 Cargo tests，0失败/忽略/过滤；当前54/468/467/1/0。三套历史 spec-check 分别43/404/403/1/0、46/436/435/1/0、50/452/451/1/0，均0。
44. 原目录全部33个 gen-*.py 的最终连续双轮在文档提交后运行。交付核验真实 exit0、全非忽略路径与字节 baseline=round1=round2=final；实际耗时/文件数/日志与最终 HEAD 在交付提供，无隔离替代，无运行中/之后非忽略编辑。
45. 只有第44项真实最终门禁成功并 clean 后，才接受 Outcome A — COMPLETE / REGISTERED STAGING；不是生产证明激活。任何实际失败都必须降为未完成，不凭此静态报告推断成功。
46. 满足单独申请恢复 R3 的正式规范前置条件；没有恢复实施的自动授权。T3-MODEL-01 为 FORMAL BINDING REGISTERED，Producer/Collector/Normalizer 未实现，Positive RootExport NOT AUTHORIZED。
47. 分类提交：b6b1577ecd09672e7bd71bc8307303823172ef10（formal registration/import/corpus），ff04beb2020e49d2214e25bbe0ab7d50616e2aa4（owner admission/denial/tests）；文档证据最终完整 HEAD 交付时核验。仅本地、不 push，最终 clean 后停止，不进入 R3/T4–T8 或后续系统。

