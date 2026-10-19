# G3 最终交付报告（终态由最终执行回执绑定）

本报告不提前宣告完成。最终验收状态、最终完整 HEAD、工程命令退出码、历史
复核和原目录生成器双轮由 `target/phase-04c1b4c4r3c2r4g3-final-runtime-results.json`
绑定。该文件在最终提交后生成，不纳入 Git，避免提交包含自身 HEAD 的循环。
机器明细见同前缀的 production-binder-input-manifest、production-audit、
runtime-coverage-ledger 和 g1-vector-conversion；实现说明见 implementation.md。

1. 起始 HEAD：`5ef4e56a3ad5ef9cc3fe050dcbe9b98f678476ea`。
   上一次中断门禁 HEAD：`08f508a2a93006911cd36805d9d199e35a42d10c`，不计全量通过。
   安全修复后完整代码门禁 HEAD：`d082c7498c0a3c97136d2e315457666a891b28fa`。
   最终完整 HEAD 以新执行回执封存；此后没有修改 Rust、tools 或 spec。
2. 冻结身份：Analysis `agentflow.root-export-checker@8`；Model
   `agentflow.computation.proof-report.model@7`；Report `agentflow.g01.report-shape@7`；
   Obligation `agentflow.g01.obligation@7`。没有另分配 Edition。
3. 完整正式清单：317 Structural Sites、66 Reference-use Policies、1,297
   注册 Field/Variants 均逐项收入输入清单；注册数量不作为生产图分母。
   208 Input Types、232 Output Types、205 Rank-free Source Closure / 0 dag_ref。
4. Source Admission 使用完整实际 Package、Owner Anchor、Revision、实际文档
   Slot、内部成员路径和实际借用；不从局部 Candidate 反向制造来源。
   Native 与 Physical 声明、完整文档和 C/K 同一对象约束均保留。
5. Compiler.specifications 是真实 44 项完整内容；FullSupportManifest 另含全部
   70 个规范源、四张闭表、217 条实际规则和十个 Role。Opaque bytes 完整入图，
   不以摘要代替。静态更早字段没有被倒写成原始更早动态故障。
6. 原始动态阻塞字段 compiled_modules 已有真实生产适配：实际 Native 私有 Map
   的 String Key 经 SpecificationId 规范化和 lookup 指针核对，完整 CompiledSchema
   无损转换；空、单元素、多元素、嵌套定义及不同插入顺序分别提供真实输入。
7. 匿名 Record、Inline、Tuple、Option、Sequence、Variant 按正式父声明、Site、
   字段顺序、Tag 和精确 Child Type 生产；名义身份不以 Rust Layout/Debug 代替。
8. Map 使用正式 map Body 和完整 Key/Value References；按类型化 Key 排序，
   重复 Key 在 Interning 前拒绝。合法 Key 映射保持 Grammar、内容和单射性。
9. 15 个 Set Site 单独核对 Member Type、成员唯一性、顺序、Child References
   和名义身份，不套用 Map Entry 语义；真实 Owner/Origin 不因规范化消失。
10. 类型化相等性完整比较 Nominal Type、Body 和递归 Child 内容；数值按数学值、
    字符串/字节按无符号 UTF-8/字节序，容器按正式类型化顺序。
11. Canonical Interning 只把 SHA256 当候选桶索引；桶内仍完整比较。
    哈希不充当 Value Identity，不新增 HashV1 Domain。
12. 真实生产节点由私有 rank-free 来源展开产生；没有手工 DAG Subgraph、
    Placeholder Reference、C Result、Fact 或 Witness 决定 G 的内容。
13. 真实 DagRank 按 AF-SEMANTIC-0217 的 Height、Nominal Owner、冻结 Type Rank、
    Payload 和已分配 Child Ranks 决定；连续 U64，Child 在 Parent 之前。
14. 精确重复、四个 Map 插入顺序镜像独立比较完整规范图；没有任意拓扑序、
    Memory Address、Clock、UUID、Call Counter 或 Source Ordinal 代替 DagRank。
15. 每条实际 Child Edge 都按正式 Descriptor 和 G-local Policy 核对目标类型、
    值、方向与 Rank；缺失、错误类型、悬空、前向及溢出 Token 分别拒绝。
16. Root 只有真实 Node<RawValidationSubject>；完整验证 Reachability、Minimality，
    不留重复相等节点或不可达构造节点。Root 排序/类型也不是调用者选择。
17. 66 条引用策略逐项保留；本入口只允许 g_value/g_local。W、output_record 和
    其它 Profile 不获得 G 生产信用；错误环境、G/W 使用和 Cross-edition 均拒绝。
    适用 G-local 7/7 条有实际边覆盖，共 1,896 个直接引用实例；其余 59 条单独保留。
18. 没有 C/W/Fact/Observation/Report 到 G 的反向构造边。私有 PendingId 只是
    构造句柄，公开结果仅含实际规范 DagRank；T4 不参与回填或重排。
19. 完整 Source Package、Owner 的 domain/owner_key/revision_key、文档 Slot 和
    Member Path 进入来源节点。同内容不同 Owner、同内容不同合法 Slot 有受控对照，
    共享内容 Child 不等于合并完整 Occurrence。
20. Native A 的 family/revision/pin/catalog/canonical bytes 全量保留。
    Native 构造器输出须等于提供的完整规范 A 输入；C-Valid 与 C-Missing 在 G 后
    独立执行，A 来源不随 C 变化。这些旧 C 元数据不是 Model@7 Subject Admission。
21. 默认界限不放宽：Depth96、Nodes1,048,576、Edges262,144、Scalar/View Bytes
    134,217,728、Work4,194,304；原 Source/Compiler 限制仍保留。
    23 个真实来源反例、三项 Native 深度/NFC/Work 边界分别记账。
    不能由公开 API 提交的畸形节点由封闭类型边界和独立图变异检验，不能伪计为 Native 输入。
22. VerifiedInputGraph 私有字段、无公开构造器/Clone/Deserialize，持有实际 Source/
    Context 借用与完整不可变图/字节。RawSubjectInputNode 绑定同一 Carrier 和实际
    Root Rank。两项编译失败测试拒绝伪造；普通输出 Claim 数据不产生能力。
23. 独立 Python Expected 只从声明源、Compiler/Context 数据和冻结规则生成；
    Rust Actual/日志不反喂 Expected。另有完整类型化 Decoder；并比较全部规范字节。
24. Site 运行域：253 个适用 G Site；四个固定 Compiler Source 不活跃分支
    （Profile B 三项 Attribution 与 future_dependency）；60 个 W-only。
    不活跃依据来自实现自有固定 Profile A/Role 源，不是删除可由调用者选择的分母。
25. 独立 Descriptor Worklist 要求 455 个实际 Value Types、538 个 Field/Variants。
    实际全部覆盖；154 个图合计 Node Instances 1,203,505、Body Fields 4,533,662，
    required=compared。完整逐图/逐字段记录在生产审计，不能以 317 或 1,297 替代。
26. 完整 Child Edges、Roots、DagRanks、Source Associations 使用每个实际图的
    required/compared 分母。来源账本逐项记录 Source Path 和实际规范 Node Rank；
    Source Set 的存储序号不会被当成规范图位置。Missing/Unexpected/Uncompared 必须全零。
    本轮实测 Edges1,751,963、Roots154、DagRanks1,203,505、Source Associations23,716，
    required=compared，Missing/Unexpected/Uncompared 均0；数量是跨图合计而非单图大小。
27. 新生产正例 154 个，来源反例 23 个、资源/名义边界三项；合计 180 个稳定输入
    CaseIds。278 个 Native 声明只是完整来源词汇，不另充 278 个生产案例。
28. 新完整图 Comparator Mutations 59 项，本轮全部拒绝。包括双侧遗漏并合法重排、
    双侧缺 Node/Type/Body/Child/Root、错误 Owner/Revision/Slot、缺 Compiler Source/
    Context Module，以及伪造 Verified/正向能力；分母始终从原始完整 Source 重建。
29. 输入敏感性由精确重复、受控同内容不同 Owner/Slot、空/单元素 Module Map、
    Native Context 变化与插入顺序镜像检验；实际输入变化不能由 CaseId 文本替代。
30. G1 原 48 向量保留 Not Executed，转换信用 0；新转换回执逐项明确没有执行
    原向量的完整专门合同。相关 G3 条件不自动把旧设计向量改成已执行。
31. 当前 3,662 Registration CaseIds 另行新鲜执行和独立核对，不计作 G3 生产案例。
    冻结 G2 内 NotImplemented/0 是历史 staging 元数据；本次运行状态由 G3 回执
    单独证明，不借此升级 Normalizer、Raw 或正向证明资格。
32. 七组精确历史：Edition1 `abc91d533b4d62e91226722e1d1fcaa30bd00883`/512；
    Edition2 `51239a0045d7a6fb274bb24cdc8130f2346bf085`/51；
    Edition3 `4d3bb59031795d31ecfccf29e3d9f2d89a7d9ce8`/173；
    Analysis4 `875e6e41f16845c78d4f6d31573826c547d48c07`/889；
    Analysis5 `e623aab705c5f8269807201e112a32e67e6869bf`/262；
    Analysis6 `2ad4359410c087462f8366f1ddb5c38bebfef681`/254；
    Analysis7 `8dc4f508c2f97b431053528ab14d2bd9263b3737`/534。
    各自使用自己的 Registry/Compiler/Expected；前后及最终 Git Blob/Raw Bytes 必须相等。
33. Native 53/53、F1 62 Inputs/40 Mutations 和原 51 个未受 F1 修复影响结果
    本轮均独立比对通过；实际退出码和输入文件 SHA256 由本阶段新鲜回执封存。
34. 冻结仍为 70 Spec / 522 Rules / 521 Covered / 1 Intentional Exception / 0 Missing；
    1,131 Descriptor / 2,559 References / 97 Imports；ABI775 / AFIDL17 /
    HashV1 Domains14 / ACE1v1；旧 774 Type Ranks 与追加 357 均不改变。
    2,290 个起始文件中只授权修改旧 Cargo.toml/lib.rs 以挂接新模块和测试。
35. 工程命令严格为 fmt --check、全目标/特性 Clippy -D warnings、未过滤
    cargo test --all -j2、spec-check；本轮全部 Exit0。Cargo445 Passed / 77 组结果，
    0 Failed / 0 Ignored。完整退出码与代码 HEAD 见 quality 及 final-runtime-results；
    Cargo0 不替代独立生产 Oracle。实际 RUST_TEST_THREADS=1、单任务、CPU5%硬上限、
    单进程768 MiB/整组1,536 MiB；完整 Cargo 峰值进程708,964,352字节、整组
    762,384,384字节，至少16,256,692,224物理字节始终可用。中断旧运行无通过信用。
36. 原目录 E:/Projects/agent 的最终全部 tools/gen-*.py 要连续完整两轮全0，
    不是隔离快照或拼接尝试。最终回执列出实际脚本/执行数及全部非忽略文件原始
    路径/SHA256，要求 Baseline=Round1=Round2=Final，Added/Removed/Changed 全0。
37. Capability Denial 必须在新 Binder、C 边界、Native/F1、当前注册/Portable
    门禁和编译失败测试共同保持；Binder 失败不返回部分图、完整 Raw Fact 或 RootExport。
38. Raw Full-field、Native Full Seed、Standalone、其它/Total T3 均 INCOMPLETE；
    T4 NOT ELIGIBLE；Positive RootExport NotAuthorized。Strict Raw Total 应仍 Exit2，
    本轮实际 Exit2；不能把独立 G 前置条件成功当成 Raw 已完成接入或完整生产证明。
39. 只有最终回执为 Outcome A 且所有要求闭合，才声明 COMPLETE —
    NON-AUTHORITATIVE PRODUCTION INPUT-G BINDER；R4-DAG-STRUCT-01 为正式阶段/
    载体绑定已注册、生产 Binder 在授权输入域已验证。否则仍按 B/C/D 保留失败证据，
    不宣告 Binder Verified。本报告自身不签发任何能力。
40. 分类提交为实现、测试/独立 Oracle、来源与覆盖文档、确定性 LF 修复、测试构造/
    主机保护修复，以及门禁后的仅文档结果封存。完整提交清单、最终完整 HEAD、
    干净工作树和 Push=false 由
    最终回执绑定。本阶段没有删除材料文件；仅本地分类提交，未 Push。

完成后停止。不得自动恢复 Raw Full-field、Native Full Seed、Standalone、其它 T3、
T4–T8、G02/G03、D10、Portable 集成审计、LockGraph、PlanCore、WorkflowVersion、
Publication 或 Runtime。

主机安全说明与已知旧摘要编码器风险见 host-safety.md。其指数分配路径未计作
Binder 反例，也未绕过、假造或放宽正式资源界限；新真实 Native 深度输入先到达
Binder 的拒绝检查。安全限制由 run-production-input-graph-safe.py 提供，非模型权限。
