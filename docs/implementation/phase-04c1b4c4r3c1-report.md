# R3-C1 — Outcome C: Formal Binding Gap

本阶段没有完成 T3。发现 **C1-GAP-01 / P1 / BLOCKING** 后停止后续适配。
本报告中的最终运行结果只以同名 `target/...-final-runtime-results.json`
为准；该文件须在实际完整门禁结束后生成，不能以本报告预填成功。
最终 HEAD 由该运行账本和交付消息记录，避免提交自引用。

1. 起始 HEAD `98c7f0a8df1da880da104d0f322aa0727d0bcfb4`，起始工作树干净，
   2000 个非忽略路径。完整代码门禁 HEAD
   `dd1492ed1d790571d2c72b37114d67baaea9c9d7`；之后仅证据提交。
2. F1 的原 Oracle、Expected、Bytes 修复源码均未改。当前完整测试重新产出原
   53 输入及 62 个 F1 专项输入；独立门禁分别检查 53/53、26+36、40 个专项变异，
   并精确比较原 51 个 Actual 全值。最终实际数值见运行账本。
3. Native 七 Case 的既有 Collector 经有限加固，但 **C1-A 未完成**。
   添加输入借用视图、阶段/角色校验、适用实例及依赖检查；没有声称完整字段闭合。
4. Observation 额外保存实际 Raw Declaration；原 canonical declaration/bytes、
   编译器身份、九成员 Context 保持独立。`check_whole_contract` 调用真实 Checker；
   Raw 或 Whole Contract 拒绝不伪造 Checked。借用视图不是完整 Scope 序列化。
5. Source occurrence 包含 binding ordinal、identity、row kind、Forward/Action/
   Usage resolution。`not_queried`、absent、present(0) 不合并；不存在从 reason 反推。
6. Bounds/Overflow 使用实际运行 trace；保持乘加除及 first-failure 顺序。
   新 validator 消费已有结果，不修改正式计算；完整独立 Seed 比较仍待完成。
7. Forward 与 Action 输入结果保留。Action error 的选中分支仍依赖成功 Forward
   输入；Forward error 不把未选中的 Action 诊断冒充成功前提。正式裁决不变。
8. Containment 只有真实执行时才有对应事实；跳过分支不制造比较结果。
9. 原 53 输入覆盖的十二 Branch 继续由未改的 Native Trace Oracle 核对；
   这不是十二 Branch 的完整 Seed/Origin/Root Observation 闭合。
10. 原完整十一字段 Guard Record 继续由旧差分检查；新增 `guard_evaluation`
    Basis 借用真实 record、branch、typed local cause refs，而不是重新解释错误文本。
11. 原十四字段 Countermodel 及 Assignment 三字段仍由 F1/原 Oracle 检查。
    新 Seed Witness 关联校验不等于新增全字段独立 Oracle；结构性 Violation
    不制造 Assignment，Unproven 不制造 Countermodel。
12. 新 `NativeDeclaredStep` 是实际字段/ordinal 路径，非 UUID、摘要或调试文本。
    Origin 的最终注册模型绑定及完整独立比较仍未闭合。
13. 本地事实引用与 Native record ordinal 已分开；Root Observation 边未实施，
    Native 局部 reachability 不能当 Root-global reachability。
14. **C1-B 未完成**。新 Python 仅实现两例 NFC source-admission 独立检查和
    一例 Raw binding gap 审计，不是 Full Native Seed Oracle。
15. Required Field 完整 roster/比较实例闭合、双侧字段删除验证未完成。
    现有 validator 能拒绝部分缺失/错误状态/坏依赖，不能据此宣称 Missing=0。
16. Standalone 27 Case 未适配：按检查点顺序，在 C1-A 停止。
17. 原 Standalone 十二维 Checker 未修改，旧回归继续运行；不计作新 Capture。
18. Declarative Application Origin 未新增，不从调用次数/临时 ID 造源。
19. 六个 RootExport direct producer 未适配；早期拒绝不虚构 preflight/九维执行。
20. Control 65、Leaf 27、Inherited 3 重新词法 census，未做完整 typed adaptation。
21. 11 prerequisite arms/20 literals 重新 census；新 Native 局部 premise 检查
    保留真实支持顺序，全部 Reference/Observation 跨 Producer 闭包尚未完成。
22. U1/U2/U5 全局来源保真未闭合；没有解析历史字符串补造 source。
23. Collector 仍为 non-authoritative LocalTypedFactSeed。源/成员验证失败拒绝，
    不签发 CompleteView、ObligationIdentity 或 RootExport Checked。
    新 NFC literal 源门禁仅覆盖声明中的 guarded String literals，不能泛称整个
    Context 的 NFC 审计完成；没有修改正式 Guard Verdict。
24. 新只读 census 仍为 181 个不同词法锚点，不是 181 个唯一语义 Producer。
    分类有重叠，Unique Semantic Producer Coverage **NOT CLOSED**。
25. 新 certified complete T3 CaseIds=0；新增完整比较实例=未闭合，而非 0 gaps。
    NFC 两例匹配及 Raw binding 一例缺口诊断单列；不混计旧 889/53/62。
26. 新 T3 full-field Comparator Mutations=0。F1 40 个专项变异独立保留，不重复计数。
27. 新完整 T3 Real Input Mutation corpus 未实施。两个 source-admission 输入和
    一个真实 Raw-port-zero 输入是有限诊断回归，不能冒充建议的 20 类闭合。
28. 当前 Cargo 内的原 889 注册案例重新运行，仍不是 Capture CaseIds。
29. 三个独立精确历史构建实际均 exit 0：Edition-1 `abc91d5...` 的512 CaseIds，
    Edition-2 `51239a0...` 的51，Edition-3 `4d3bb59...` 的173。
    分别用自己的 Registry/Expected；全部5445 Git blobs 在运行前后精确一致。
    实际 Cargo log pass records 与历史 CaseIds 在账本分开记录。
30. 完整代码 HEAD 上重跑 fmt、Clippy(all-targets/all-features,-D warnings)、
    cargo test --all -j2、spec-check。初轮 Cargo 390 pass/0 fail/0 ignored/74 groups；
    新鲜最终门禁的退出码与日志摘要只在最终运行账本确认。
31. `frozen-inputs.json` 冻结1997个原始文件；仅3个明确许可旧路径修改。
    54/468/467/1/0、775 ABI、17 AFIDL、14 HashV1、652 types/1941 refs/
    97 imports 保持，90项现行历史类型声明及19组快照精确校验。
    规范、旧 Expected、F1 原 Oracle 与 af-effect 修复原字节保持不变。
32. 最终证据提交后，原目录 `E:/Projects/agent` 实际34个生成器连续两轮。
    验收要求 baseline=round1=round2=最终全部非忽略文件逐路径 raw SHA256；
    任一脚本失败不得拼接。实际退出、时长、路径数由最终运行账本记录。
33. Native analyzer、direct checker、private constructor、alternate entry 以及
    Portable D4 原拒绝测试重新运行；默认 D0 限额不改，Positive RootExport
    保持 NotAuthorized。局部 Proven 不越权。
34. G01 incomplete；G02 OPEN/P1；G03 OPEN/P2。完整 Scope/Normalizer 均未实施。
35. F01/F02 CLOSED；F03/F04 OPEN/P2，未修改已有判定。
36. 原 B01/B02 及 B03–B07 仍 blocking，不与独立的 R3-NG-B01 混淆。
37. `R3-NG-B01 CLOSED — PRODUCTION LOWERING FIX VERIFIED`：严格 base64url
    到 Bytes 不退化成 String；原53/专项62/40重新执行，完整 Seed 闭合仍 pending。
38. 新 **C1-GAP-01**：真实端口0 Raw 输入在 canonical validation 拒绝，产生
    schema-validity Violation。实际九成员 Context 没有 resolver_dependencies；
    现有 contract_validation Subject 要求十成员 Context，另外四个 Subject
    均要求此次调用未提供的输入或阶段。最早无损点是 analyze_contract 调用
    invalid_contract_report 之前。精确输入、规则、首个不可绑定字段及五分支排除
    见 `formal-binding-gap.json`。这不是 canonical Native Guard 七 Case 本身缺口，
    也不能仅以待做 G02 序列化解决。不得补造空 resolver manifest。
39. **Outcome C；未达到 Outcome A。** Native checkpoint 未完成，R3 incomplete。
    非 NFC 探索不构成有效输入域上的 Production/Formal Contradiction：注册
    String 是 nfc_text；拒绝该 capture source，保留正式 kernel 原始诊断。
40. T4 **NOT ELIGIBLE**；不进入 T4–T8/G02/G03/D10 或后续架构阶段。
    下一步需单独授权 direct native raw-validation Subject/Context/Source/Origin
    的规范裁决与注册，不能由本阶段 Rust-only predicate 代替。
41. 分类提交：`acfb5c1` 有限 capture 加固；`73bce46` 两项测试/独立审计；
    `dd1492e` 冻结与 census 审计；其后仅报告证据提交。最终完整 HEAD、干净
    工作树、未推送状态由最终账本及交付核实。过期空 index.lock 移到同目录
    `index.lock.c1-stale-20261016T1444` 可恢复备份；没有重置索引或删除源码。

## Development evidence and limits

`target/phase-04c1b4c4r3c1-source-probe-{1,2,3,4}.log` 保留探索至修正全过程。
早期 prototype 的“差异”不是有效域语义 B 判定；随后测试曾因 canonical
diagnostic presentation 与物理 NFD 源混淆而失败。修正新审计中该 presentation
的预期，不修改旧 Oracle、旧 Expected 或 Guard kernel；物理源单独完整保留。
旧 Actual projector 对 observation 和 witness.source_contract 使用 canonical
presentation，不能据其对无效 NFD 源宣称无损捕获。
`clippy-1.log` 保存 redundant-closure lint 失败，`clippy-2.log` 保存修正。
这些开发日志不替代稳定源码上的最终新鲜门禁。

独立外部重跑驱动初次未启用 UTF-8，Windows GBK stdin 将文件中 UTF-8 的
“甲”误读，出现52/53；磁盘 Actual 原字节及 Cargo 内 PYTHONUTF8=1 的53/53
均正确。失败保存在 `independent-driver-encoding-failure.log`；仅新 ignored
驱动改用 `python -X utf8`，不改原 Oracle、Actual 或 Expected。
