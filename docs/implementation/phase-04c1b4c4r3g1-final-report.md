# R3-G1 — Forty-one-point adjudication report

Design decision:Option C / proposed independent native_recovery_guard binding with constrained historical native imports. Outcome A is DESIGN COMPLETE only;registration/capture remain pending. Exact final HEAD and actual final generator completion are reported at delivery,without changing these frozen documents.

1. 起始／完整当前代码门禁HEAD均为 5bee948a033cfbf1233c65448e39d0c344993b52；最终提交完整HEAD在交付时列出，避免收据自引用。
2. AF-CONTRACT-0022原生语义：单Contract的Forward Guard对实际action E Guard的局部包含／类型化反例判断，不产生Root或运行时权威。
3. 原生入口、11个record字面位置、countermodel builder/constructor均在machine audit列出；validate/canonical/root消费者的实际函数与信息丢失点见native-subject与draft。
4. 隔离诊断11个原生场景：合法P、Int64 typed V、Unit U、zero P及结构V／缺源U等；正反例都保留完整声明来源，另有56条Owner矩阵／4条嵌套控制。
5. Int64负例canonical验证通过，但check_contract在contract_validation正式拒绝；没有Checked Contract。
6. 对五类候选作12项语义标准比较，并逐一审查所有9个Source变体；不是仅依赖名称或API拒绝。
7. 未证明存在完整可复用绑定，不满足Outcome B；负例没有IR也没有Checked-only pin，现有Source不能合法偷换。
8. model@1确实注册声明五类Guard类型及完整字段／variants；不能说历史从未注册。
9. model@2完整继承历史对象；五类不在当前518活跃表。1593refs保持；静态ref可达488与ModelType可分派518区分。
10. 当前ModelType/ModelRef仅按current-owner exact-name或真实owner/name键查types；历史payload不自动成为可Claim import。
11. 拒绝是安全fail-closed的暂存可用性限制，不是不存在历史类型的证明；旧Scope不能按同名当成新Scope。未来精确Import／materializer修复与Predicate绑定是两件事。
12. TypedGuardCountermodel全部14字段及精确native/logical类型、Source getter、owner、canonical规则与Expected规则逐行列出。
13. 14字段无损投影设计完整；保留所有assignment三字段、有效Forward bounds、本地multipliers、reachability和规则，不用exists/count/hash/Debug代替。
14. Native Source为完整CanonicalNativeContractDeclaration及真实binding；不要求负例获得CheckedContractContents。
15. Forward/action E ordinals与action-binding ordinal严格分离；Closed/Open.known选择与absent/not_queried/present均显式。
16. 采用实际九字段Native Contract Context；analysis/canonical_validation两角色实际同实例绑定，禁止digest→pointer证明。
17. IR-free Origin锚定Canonical声明与真实binding/reference应用；重复调用不造新Origin，Root观察不倒灌。
18. 草案精确定义Source/Target/Coordinate及typed NativeGuardMemberPath，无dummy IR/node0或Whole Context替代成员。
19. 唯一推荐独立native_recovery_guard profile，七个Case、一个新native kind／六个非context clause，context复用proof_context；全部DRAFT未注册。
20. 区分guard_containment调用结果与guard_evaluation原生算法最终结果；保留真实P/U/V与四类basis，不新增RequiresEvidence候选。
21. 反例assignment来自实际primitive域，Forward真／Action假；局部类型可达不等于Root-global可达。
22. 设计精确Native prerequisite扩展、完整typed refs、consumer／observation关系；不自引用，不以RuleId代EvidenceRef。
23. Native Fact、Root call-site/source、RootRecovery aggregate三层身份不同，不能改写原始stage/source/origin。
24. 同Native origin可有多个Root observations；不同真实Root Source Occurrence仍是不同root事实，不能重复Native计数。
25. 完整比较A现有绑定／B单独历史Import／C独立Native封闭绑定／D缩小范围；A不足，B仅技术组成，D未选。
26. 唯一推荐C，受限复用历史类型；以正式注册、全部闭包／准入／迁移和后续生产验证为条件。
27. Native Recovery Guard Predicate Amendment Draft含可测试NG-* MUST、闭合数据、生命周期、Owner及Conformance要求；标签不是已分配RuleId。
28. 所有新增闭合意义要求新model/report/obligation/analysis完整内容身份，版本号待独立注册阶段分配；不修改当前@3/@2/@2完整字节。
29. 48个新设计向量全部Not Executed（16/12/12/8），独立于11个诊断和旧R1的48向量。
30. 未来Python Oracle设计从声明／context／冻结算法推导全字段；正例和负例均手算完整Native结果，负例同时保留full check拒绝。
31. G01 Capture／Normalizer未实现，完整Scope阻塞；G02 OPEN/P1，G03 OPEN/P2；未实施D10 full-field。
32. F01/F02 CLOSED；F03/F04 OPEN/P2；同Entry witness、同源Root关联、Root-global reachability边界不变。
33. B01/B02及B03–B07继续BLOCKING。
34. 新审查结论：Owner安全可用性限制、结构V无assignment反例、eager双输入检查后first-error选择、Witness与Scope DAG归属需明确；无已确认false-Proven/P0。
35. 起始1879个非忽略文件逐字节冻结；Normative／registry／775 ABI／17 AFIDL／14 HashV1／旧Expected均不变。
36. fresh fmt/clippy/test/spec-check全部退出0，当前336测试通过／0失败忽略过滤，173stagingCase保留；50/452/451/1/0。
37. 当前源码HEAD为5bee948；精确旧Edition2 HEAD51239a0045d7a6fb274bb24cdc8130f2346bf085、Edition1 HEADabc91d533b4d62e91226722e1d1fcaa30bd00883，本阶段均未重跑，旧日志不冒充fresh pass。
38. 原目录现有32个gen-*.py最终完整连续双轮按交付的真实退出码与字节图记录；所有新增设计文件先纳入基线，运行期间和之后不改非忽略文件。
39. 本阶段裁决目标为Outcome A — DESIGN COMPLETE／AMENDMENT DESIGN ACCEPTED，不是Outcome B既有绑定已验证，也不是注册或T3完成；须最终冻结与生成器门禁实际通过后交付。
40. 具备提交独立注册授权的设计条件；尚无注册权限，也没有恢复R3／T4准入；注册验收后仍须单独授权R3。
41. 分类本地提交只读审计／设计／门禁证据；最终完整HEAD与clean状态交付时核验，不push，不自动修metadata或进入任何后续阶段。
