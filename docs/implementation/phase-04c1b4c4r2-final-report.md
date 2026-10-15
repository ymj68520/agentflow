# R2 — Forty-point final report

This report follows the requested order. Actual full-code and registration-docs double-round gates passed;see gate-report.md and gate-receipt.json. The additional final-tree post-commit pair and exact final HEAD are reported at delivery without changing these frozen documents.

1. 起始 HEAD `51239a0045d7a6fb274bb24cdc8130f2346bf085`；完整代码门禁 HEAD `635616cea0916429161a5a353dc9a4090834589d`；最终完整 HEAD 由交付时 git rev-parse HEAD 给出（本文件不能含自身提交哈希固定点）。
2. 可安全独立暂存：生产支持、激活、release 均受实现控制，Native／Portable／私有构造器分别拒绝正向能力。
3. 独立 PredicateFamily/Profile 为 standalone_substitution，不借用既有 Case 或将 Scope 类别误作 Family。
4. 27 个精确 Case 逐条见 closed-case-roster.md；各有独立完整字段 Expected／注册测试。
5. 26 个非 Context Clause 封闭登记；context_association 单独采用 direct:proof_context；checked_operand 为 old/replacement 精确参数化。
6. 12 维按冻结顺序逐条登记并完整字段比较；详见 twelve-dimension-binding.md，不压缩到 RootExport 九维。
7. Old／Replacement 保留完整13字段声明、canonical bytes、exact pin、catalog/name、实际验证来源与上下文。
8. Context 三角色 analysis／old_checked／replacement_checked；analysis 别名 replacement；原生 std::ptr::eq 不变；元数据形状不证明真实对象关联。
9. lineage_equality 的 P/U 与 result_identity 分离；谱系不等不成为 V，changed body 即使局部谱系 P 仍最终 identity U。
10. Exact canonical body shortcut：共享 Context 下12个 reflexive P，不伪造11个 comparator 执行事实。
11. Changed-body 路径保留 U floor、全部局部候选和 outer max；cross 只折叠11个非自身结果。
12. Execution 的 combined 内层保留 U-over-V；feasibility/security 独立；共享 Error 不冒充3次成功比较。
13. Usage fallback 不增加 recovery-profile equality gate；Recovery P 仍要求 K equivalent 与 action binding 相等；Weakened 为 V，其余 unsupported 为 U。
14. Status／Evaluation／Blocked 封闭；上下文不成立后12个 Blocked 不冒充 evaluated U；ProofVerdict 未扩展。
15. Source／Target／Coordinate 追加精确 standalone 变体、成员路径与 Option；不虚构 caller IR／root／resolver。
16. ApplicationOrigin 为完整声明式 ApplicationDocument 和实际 applications[index] 路径；assertion_path 已知为空；禁止 UUID／时间／probe／源代码行。
17. 真实原始验证先决条件保留 stage/path；Reference 为 typed evidence coordinate；一个来源多观察，独立声明不同 origin。
18. SubstitutionBasis／UnprovenBasis／SubstitutionWitness 及 Fact／TypedContradiction／Observation 扩展完整；V 需要实际有效 typed witness；非 aggregate 类型排除 aggregate。
19. 五个冻结 error.rs 全部69种已知错误：resource12／flow11／usage11／recovery13／execution22；含 nominal/source SHA/payload；未知 UnsupportedModel，缺 payload IncompleteFacts。
20. ACE1 grammar 未改；精确 Option、NFC／raw bytes、nominal width、数学数值次序、typed sets/maps、有限 DAG／重复拒绝等已注册；未实现 G01 encoder／normalizer。
21. 正式分配 analysis agentflow.root-export-checker@3；已检查起始 spec/crates 无该已分配身份，不按阶段盲猜版本。
22. Report shape agentflow.g01.report-shape@2；Obligation agentflow.g01.obligation@2；规范模型 agentflow.computation.proof-report.model@2。
23. 完整 compiler tuple 纳入新来源、全部 registry bytes、closed tables、24 selected sources／149 applied rules／support roles，严格区别旧版；标签或 checksum 不替代完整内容。
24. 新增4 Specification、16 RuleId（AF-SEMANTIC-0229–0244）；实际总计50／452，reference-covered451／exception1／missing0。
25. 旧46／436行按整个 JSON record 完整相等；不是缩短字段投影；旧规范源与 Expected 未修改。
26. 实际新增173个稳定独立暂存 CaseId：27+12+69+23+16+10+10+6；不是173个 cargo test function，也不是48语义向量。
27. Edition1 精确旧构建独立全量回归通过：512 CaseIds，0失败／0跳过；Cargo 报告382次测试通过（含真实子进程调用），0 ignored；子进程按需过滤144次不是跳过 manifest Case；所有1732源 blob 前后匹配。
28. Edition2 精确起始构建独立回归通过：原51暂存 CaseIds，0失败／0跳过；335 test functions；所有1811源 blob 前后匹配。
29. 新版 Successor 全量回归已退出0：173个注册／形状／准入 CaseIds 全通过；Cargo 336个测试通过，0失败／忽略／过滤。旧结果不换标签计入新版通过。
30. Native 10／Portable10／Profile6及35个补充直接调用检查拒绝 capability；未完成支持不产生 CheckedRootExportCompatibility。
31. G01 Binding REGISTERED、Capture NOT IMPLEMENTED、完整 Scope BLOCKING、Normalizer NOT IMPLEMENTED；G02 OPEN/P1，G03 OPEN/P2。
32. F01／F02 CLOSED；F03／F04 OPEN/P2；规则 reference coverage 不等于 dependency closure。
33. B01／B02 Full Conformance Closure 及 B03–B07 保持 BLOCKING。
34. 新 Findings 见 findings.md：旧约束数量消歧、实际 nominal/import 物化、非权威 shape 边界、旧 generator 精确输入冻结；首次重叠源码运行失败日志保留。
35. ABI775／AFIDL17／HashV1 domains14，既有 Digest／SemanticIR／ComponentContract／RootExport／Portable／D9 Expected／D10 gaps 全冻结；原1801未授权路径之外文件逐字节不变。
36. 最终 Cargo fmt／clippy／test／spec-check 全部退出0；实际代码 HEAD635616cea0916429161a5a353dc9a4090834589d，见 gate-report.md；不将失败开发轮或部分通过当最终全量通过。
37. 原目录32个 gen-*.py 的登记文档双轮全部退出0；1866文件 baseline=round1=round2，0增删改，190.43s／166.89s。收据提交后最终树另跑完整连续双轮，实际最终结果交付时报告，不拼接两组。
38. 按登记实现／规范消歧／注释／审计文档与最终收据分类本地提交；交付前核对干净工作树；不 push。
39. 已达到 R2 Outcome A — COMPLETE / REGISTERED STAGING；实际全量与原目录双轮收据已通过。最终交付另要求最终提交树双轮与快检通过；不是 Proof Activated。
40. 满足 R2 所有门禁后仅获得 R3 的规范前置准入；T3 Producer Resumption 必须另行授权。本阶段停止，不自动进入 R3／T4–T8／D10 full-field／publication／runtime。
