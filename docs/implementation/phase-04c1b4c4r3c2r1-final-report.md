# C2R1 最终报告 — Outcome D（完整 Conformance 未闭合）

动态实际门禁、最终完整 HEAD、原目录字节图与日志见
E:/Projects/agent/target/phase-04c1b4c4r3c2r1-final-runtime-results.json。
此文档不把局部通过写成完整验收，全部最终执行结论须与该账本共同核对。

1. 起始 HEAD 23a25b69d65166be3ec870ec1177ddd6ea78dc69；完整代码门禁 HEAD
   dbdbba88e22e064b50e63c6b168b248d84ab3e9b；最终 HEAD 在运行账本，避免提交自引用。

2. Analysis agentflow.root-export-checker@6、Model agentflow.computation.proof-report.model@5、
   Report agentflow.g01.report-shape@5、Obligation agentflow.g01.obligation@5 全部冻结。

3. 实际入口 capture_raw_ports / collect_raw_ports，调用者实际提供完整来源请求。
   metadata admission 之后执行封闭适配和真实 canonical_ports，不以准入成功代替事实。

4. Owner 完整四键、Revision、真实 declarations/imports/sources Slot 来自已提交包。
   没有默认 Owner、UUID、调用计数、时间戳或合成历史来源。

5. 相同文档不同 Owner 的两次真实 Capture 保持不同 Subject；不以内容相等合并来源。

6. 同 Owner 不同 Slot 区分；同完整 Occurrence 重复 Capture 复用身份，合法观察可合并。

7. 文档不同拒绝 BindingConflict；显式联合包 Owner 全记录不同拒绝 OwnerBindingConflict。
   同键状态/Witness 等 Payload 损坏拒绝且不改写已有结果，无全局来源账本声明。

8. 来源仅 DeclaredOnly。合法不同声明域名可准入，不代表外部认证或全球所有权。

9. Physical+匹配 supplied、Native+not_supplied；另两种组合失败关闭。
   Physical 原始字节不重序列化，UTF8/重复键/宽度/字段/路径负例先于 Predicate 失败。

10. 原始候选必须完整匹配实际文档内部 Entry；转换所有 Native 文档的十三字段
    实际 Rust 值，不截断文档。测试保留并比较完整 Typed 值，含非空其它维度/NFD 文本。

11. 九成员 Context 完整借用；analysis/validation 必须同一个实际对象，
    同值异对象拒绝。未伪造 resolver；跨进程完整 Scope 和独立内容投影仍缺。

12. Hook 在真实 canonical_ports 条件分支，错误字符串构造前产生 Zero/Duplicate，
    Schema resolve 前产生 PositiveFresh。没有第二份替代正式 Validator。

13. Identity→Input→Output 原顺序保持，两方向各自独立命名空间。

14. 实际 U32 数值稳定排序，覆盖 1/2/10/u32::MAX 和稳定重复，未采用字符串排序。

15. sidecar 保留原始 ordinal；sorted ordinal 及真实前驱由实际排序元素产生，
    不以报告索引/源码行替换 Origin。

16. Fresh positive=P、zero=V、duplicate positive=V。P 无 Witness，V 有对应本地
    typed witness；不制造 Unproven，不把 P 提升为整个 Schema/Contract 通过。

17. Identity 失败无 Port Fact；Schema 在 P 后失败保留真实 P；后续事件不制造。
    主 Capture 独立适用事件 125；四个准入成功但无 Port 事件的输入单列，非未知空集合。

18. RawBasisRef 保留完整 Subject 借用、Occurrence、前驱、condition、selected_error、
    完整规则。它不是已编码的 G01 RawPortCheckBasis 全字段闭包证明。

19. Zero 本地 Witness 保存 0/1 且无虚构前驱；Duplicate 保存前驱原 ordinal/编号。
    正式 Witness.subject 与 TypedContradiction 物化仍缺，列为 Required Field Gap。

20. Source 由实际 Subject 保留；Target 借用实际完整编译类型定义与规则；Coordinate
    使用真实 occurrence。正式完整 FactKey/Source/Target 封闭事实记录尚未物化。

21. Origin 视图携带实际 Owner/Slot/完整文档/内部路径、方向和原 ordinal，
    不把 Context 或观察位置当来源。正式 Origin/FactKey 独立全字段闭包仍未完成。

22. 声明路径与文档内部成员路径分别保留，指向同一原始端口。
    本地类型名标签尚未完成正式 ModelType {owner,name} 投影，未隐瞒此缺口。

23. 密封 Collector 只接收真实 Capture，拒绝私有 Payload 八种损坏、冲突或无效观察。
    本地事实复制合并观察；正式引用/primary/Prerequisite/Consumer/ObservationSite 尚未闭合。

24. 输入包/文档/Context 不引用输出事实/Witness/观察，RawSubjectRef 不含输出。
    未实现完整 SourcePackage DAG/ACE1 Scope encoder，不生成 Canonical ObligationIdentity。

25. 独立 Python 从声明输入与冻结规则产生 Expected，未从 Rust Actual/错误字符串导出。
    仅局部投影比较可以通过；默认严格入口仍 exit 2，不能称 Full Raw Oracle 完成。

26. 独立主 Capture Required Event 清单 125；匹配/缺失/额外计数由实际运行账本确认。
    双侧共同删除事件必须拒绝；早停后未执行成员不当作 evaluated 事实。
    额外身份重跑仅完成关系断言，不声称整个 Raw pipeline 完整事件/字段闭包。

27. 正式 Fact 14 根字段直接取自 model@5；4 类仅有局部映射，10 类未物化。
    至少 1250 必需根字段实例缺口，另有 Compiler/Context/引用链等嵌套缺口，不能报零。

28. 新输入 87 CaseIds：63 来源准入成功，其中 59 有真实 Port 事件；24 来源负例。
    254 注册案例及 88 注册站点完全另计；空输入/更早 Identity 失败不充当 Port Capture 案例。

29. Comparator 35 项变异，含状态/Witness/前驱/排序/规则/源/Schema/假权限，以及
    Expected/Actual 同时删必需字段或事件。它们验证局部门禁，不消除完整字段缺口。

30. 24 个实际来源负例及不同 Owner/Slot/重复来源正例均实际运行。
    开发中 Option 错误和无效域变异保留失败日志，只修本阶段新输入，不改旧 Expected。

31. 当前 Analysis@6 254 注册回归独立按原期望核对：170 shape、14 rule、51 admission、
    8 bounded DAG-subgraph、9 Portable、2 runtime negative；不算生产 Capture Coverage。

32. 五组精确冻结源树分别 cargo test --all，各用其 Registry/Compiler/Expected。
    用户指定 CaseIds 为 512/51/173/889/262；前后 9542 Git blobs 全字节隔离核验。
    实际五组均 exit 0，Cargo 通过记录分别 382/335/336/390/434；不是 CaseId 数。

33. Native 53/53 与 F1 26+36 输入/40 变异独立重核；原 51 完整 Actual 相等检查保留。
    不据此宣称 Native Full Seed 或其它 T3 完成。

34. 62/493/492/1/0、704/2064/97、ABI775/AFIDL17/HashV1 domains14 不变。
    初始 2118 路径仅 5 个授权实现文件可变，2113 原路径 raw SHA 冻结，旧规范/Expected 不变。

35. 在固定代码 HEAD 重新运行完整 fmt/Clippy/Cargo --all/spec-check，实际退出和
    四项最终退出均为 0；Cargo 436 条通过记录、75 组、0 failed/ignored。
    Rust 成功不替代 Python 严格 Full-field 失败；被后续补强取代的 ce35243 旧整轮日志单独保留。

36. 最终文档证据提交树在 E:/Projects/agent 原目录执行实际全部 gen-*.py 连续两轮。
    各脚本 exit、全非忽略路径/raw SHA 图、增删改及最终树相等由账本核验，不用镜像替代。

37. Current-only、Native/direct/private/alternate、默认 D0 ResourceLimit 与显式有界 D4
    拒绝仍在现行回归中；没有新增 Supported/Activated/Released、Checked 或 RootExport 路线。

38. C1-GAP-01/C2-GAP-01 保持 FORMAL BINDING REGISTERED。
    Raw Producer 全字段验证未闭合，不能加 RAW PRODUCER VERIFIED FOR AUTHORIZED SCOPE。

39. G01 注册不重开但 Capture/Normalizer 未闭合；G02 OPEN/P1，G03 OPEN/P2；
    F01/F02 CLOSED，F03/F04 OPEN/P2；B01–B07 BLOCKING；原 F1 修复状态不倒退。

40. Outcome D — Conformance Incomplete。不是 A；没有用缺字段默认值绕过。
    当前缺口是实现/独立验证未闭合，不声称发现新的不可表达模型或生产 false-Proven。

41. Remaining Raw Capture、Native Full Seed、Standalone、剩余 T3/Total R3 均 INCOMPLETE；
    T4 NOT ELIGIBLE，Positive RootExport NotAuthorized。本次不自动恢复其它阶段。

42. 分类提交：72bccc2 真实观察/受检来源/本地 Collector；ce35243 测试/独立局部门禁/
    完整字段失败关闭；dbdbba8 补齐 Collector 原子联合来源冲突检查；文档/机器回执
    另作证据提交。最终完整 HEAD、clean worktree、
    未 Push 以最终账本与 Git 核对。保留全部开发失败日志，交付后停止。
