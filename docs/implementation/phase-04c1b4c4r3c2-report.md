# Phase 4C.1B.4C.4R3-C2 — Outcome C / Additional Formal Binding Gap

结论：**C2-GAP-01 / P1 / OPEN / BLOCKING**。用户明确要求“两份来源独立、
整份内容也完全相同的文档必须区分”；当前冻结模型只有文档内容及其内部路径，
没有可表达该要求的文档 occurrence/owner 身份绑定。在 Source Admission 检查点
停止 Producer 适配，没有修改规范、Registry、Edition 或生产 Validator。

这不是已观察到的 false-Proven，也不把普通 ContractError 当作新的 Typed Fact。
C1-GAP-01 仍为 FORMAL BINDING REGISTERED；本次是更强的文档身份要求暴露的新缺口。

## 最小复现与可证范围

新增 Rust 诊断构建两份分别持有存储的真实两条目声明文档。两份完整文档都包含
一个 input Port 0 声明及一个 input Port 1 声明，逐个完整类型值相等；选择各自真实
第0条目，使用同一完整 Compiler 和同一个九成员 Context 对象。两份均通过当前
`admit_native_raw_source`，但实际 canonical validation 在 Port 0 拒绝，不能取得 Checked。
测试不在 Collector 内倒推文档，也不把对象地址序列化为 Origin。

独立 Python 审查从冻结 Descriptor 建立字段穷举证据：

- RawDocument 的 Native Variant 只有 declarations，Physical Variant 只有 bytes。
- RawInputOrigin 只有 document 和实际内部 path；RawValidationSubject 只有
  owner_compiler/input/context。相同 Context 的 roles 不能代替文档身份。
- Source.raw_validation 绑定同一类型的完整 subject；RawValidationOrigin 的
  input_origin/direction/original_ordinal 在此例也全部相同。
- Scope/Origin/FactKey 的其余字段不含真实文档 occurrence 输入；不能把新的文档
  编号塞入 declared_path/source_ordinal、另造外层文档或加入无关 Scope roots。
- AF-SEMANTIC-0217 的规范最小 DAG 必须 intern 相等 typed values；不同任意
  Node rank、重复相等节点或合成 UUID 不能合法地补足该差异。

因此现有类型只能表达“相等内容的同值文档”，不能表达用户要求的“相等内容、不同
来源文档”。不同完整内容或不同真实条目路径本来已可区分，不属于此缺口。
这是一份结构可表达性证明和真实准入诊断，不是假称已执行的 T4 Scope 编码实验。
原生类型不支持 Serialize：证据保留完整反序列化输入，并以实际 Rust 完整类型 Eq
断言绑定两份文档；它不是完整 Raw Actual 字段投影，更不是完整 Seed 差分结果。

独立工具默认退出2，表示缺口仍然开放；`--report` 退出0只是诊断报告模式。
工程测试断言该拒绝结果，不将“成功复现缺口”标成 Raw Capture 通过。

最小输入、实际准入诊断、完整相关 Descriptor、首个无法绑定的要求及论证保存在
`phase-04c1b4c4r3c2-formal-binding-gap.json`；2047个原文件的原始字节冻结清单在
`phase-04c1b4c4r3c2-freeze.json`。可运行：

```powershell
cargo test -p af-computation --test raw_document_identity_c2 -j2
python -X utf8 tools/audit-raw-document-identity-c2.py
```

## 按请求顺序的41项交付

1. **HEAD。** 起始23a7c01ab616a268d600ea8b1be9d721341eee31，开始时工作树
   干净，2048个非忽略路径。完整代码门禁 HEAD 为
   b529f8fe0ee2075f11afe93578a3cf640cd69473。最终完整 HEAD、实际门禁退出码、
   日志哈希及工作树状态在 `target/phase-04c1b4c4r3c2-final-runtime-results.json`。
   该帐本在最终报告提交和生成器双轮之后生成，避免提交哈希自引用。
2. **正式版本。** 冻结 Analysis agentflow.root-export-checker@5、Model
   agentflow.computation.proof-report.model@4、Report agentflow.g01.report-shape@4、
   Obligation agentflow.g01.obligation@4。683 descriptors /2018 references /
   97 historical imports 不变。没有 Rust-only Predicate、Variant 或新 Type Rank。
3. **Source Admission。** 重新审查 raw_admission.rs：校验完整 Compiler/Registered
   Edition、实际同 Context 对象、真实文档及所选条目的完整值。成功只返回 unit，
   不是 Fact/Verified Producer token。内容相等不能满足独立文档 occurrence 身份要求。
4. **Physical/Native。** 原 API 原样保留：Physical 保存完整原始字节，严格拒绝
   重复键、非法 UTF-8、未知字段、错误宽度，不重序列化伪造历史字节；是非 wire
   Metadata，不是通用 Decoder。Native 为真实完整十三字段声明，physical=not_supplied。
   该 Metadata 结构准入也不证明真实 Port Predicate 已执行。
5. **真实文档。** 新复现提交两份分别构建的两条目文档，各自选 entry0，条目内容
   相等但不是同一存储。诊断对象非别名只是测试证据；地址没有编码进模型。
6. **MissingDeclarativeOrigin。** 无实际文档返回该失败；选 entry1 而候选仍是
   entry0 返回 InvalidSourceAssociation。没有补造单条目来源或完整 Origin Fact。
7. **Context。** 模块、Schemas、Effects、Resources、Flow、Usage、Recovery、
   Execution、Relations 九成员用 Rust 穷尽解构检查。两角色和两文档都借用同一
   context；另一个同内容 Context 对象被 InvalidContextBinding 拒绝。不补 Resolver。
8. **canonical_ports Hook。** 源码已定位正数/重复判断至 ContractError 之前。
   因更早 Source 身份检查点失败，Hook NOT IMPLEMENTED；生产源码完全未改。
9. **Input/Output 顺序。** 原 canonicalize_contract 先 identity，再 inputs，再
   outputs。源码检查及冻结旧测试保留；不是新事件流的动态完整覆盖。
10. **稳定数值排序。** 原 sort_by_key(PortRef::number) 是 U32 稳定数值排序，
    未修改，不使用文本排序。新 Capture 的排序证据未实施。
11. **Ordinal。** 实际条目和输入端口均选原始 index0；未实现稳定排序期间保留
    original/sorted ordinal 的 Hook，未从排序后结果反推来源。
12. **三分支。** number>0 且 number!=previous，哨兵0非真实前驱的既定语义保留。
    本复现只执行旧 Port0 拒绝；旧注册回归的其它行为不冒充新 Producer 三分支证明。
13. **Typed Basis。** RawPortCheckBasis 已注册但本阶段没有生产任何此类对象。
14. **Witness。** zero/duplicate 已注册，新的 Typed Witness NOT IMPLEMENTED。
    没有解析 ContractError.detail、code 或 report reason 构造 Witness。
15. **Source/Target/Coordinate。** 已审查已注册绑定；Source 的文档 occurrence
    输入缺失是首个失败点。没有伪装 schema reference 为实际执行的 Schema Query。
16. **RawValidationOrigin。** 文档值、内部路径、direction、original ordinal
    都相等时无法区分独立同内容文档。不滥用其它 ordinal、ProbeId 或临时 UUID。
17. **双路径。** 同一真实 entry0/input0 是反例固定位置；没有实现双路径 Capture
    校验器，不能宣称 RawDocumentMember/DeclaredStep 全量生产一致性已完成。
18. **早期拒绝。** Port0 的旧 canonical validation 拒绝已实测。没有为后续
    output 或未访问端口制造事件；reached/not_reached/prefix Capture 未实施。
19. **Raw/Checked 隔离。** 两份 Raw 均通过来源准入，但 canonicalize 和 whole
    check 均拒绝；旧 Native guard 记录数0。不产生假 Canonical、Checked、CompleteView。
20. **Collector。** 未增加 Raw 接口/开放 Proven constructor。不得把本阶段无
    Capture 解释为“成功获得完整空 Fact Collection”。原信任边界未改。
21. **独立 Expected。** 新 Python 仅独立构造反例输入并穷举冻结身份 Descriptor，
    比较实际准入诊断。Independent Raw Full Seed Oracle NOT IMPLEMENTED，不读 Actual
    反向制造 Expected。旧 Native/F1/C1 Oracle 全部原样保留。
22. **Required Event Coverage。** NOT IMPLEMENTED / NOT ZERO GAPS。无法列出
    全部适用生产事件；不以两份准入诊断或262注册 CaseIds代替事件闭包。
23. **Required Field Coverage。** NOT IMPLEMENTED / NOT ZERO GAPS。没有
    RawProducerRequiredFieldRoster 全字段闭包，也没有双侧删除字段通过保证。
    身份 Descriptor 穷举不是生产 Required Field Coverage。
24. **新 Capture CaseIds。** 0。仅新增1个 source-binding 诊断 CaseId；Cargo
    新二进制共22条 test records，其中21条来自复用旧 fixture，不算22个新 CaseIds。
25. **Comparator Mutations。** 新 Raw 全字段 Comparator mutations=0；不把
    已有 F1 的40个或本次准入负例计入该数字。
26. **真实输入负例。** 缺少文档、选错真实条目、同值但不同 Context 对象均真实
    拒绝；它们只是缺口复现的局部负控，不是32-case Capture 或24-mutation corpus。
27. **当前注册回归。** 262 CaseIds（241 shape+11 rule+8 portable+2 runtime
    portable adversarial）和129/129新 Descriptor 站点由未修改的注册测试/审查重跑。
    注册完成不等于 Producer 完成；实际工程门禁见最终帐本。
28. **历史。** 分别在精确源构建运行 Edition1/2/3/Analysis@4 的512/51/173/889
    CaseIds，冻结 HEAD 分别为 abc91d533b4d62e91226722e1d1fcaa30bd00883、
    51239a0045d7a6fb274bb24cdc8130f2346bf085、
    4d3bb59031795d31ecfccf29e3d9f2d89a7d9ce8、
    875e6e41f16845c78d4f6d31573826c547d48c07。7466个源 blob 前后精确核对。
    Cargo pass records 与 CaseIds 分列，实际完成/退出只以最终帐本为准。
29. **Native/F1。** 当前实际输出重跑原53-case默认差分、F1 26 lowering+
    36 native inputs/40 mutations，并核对原51例完整 Actual 未变。TRACE_ONLY_MATCH
    不等于完整 Seed；实际独立输出/退出码留在 independent-gates.json。
30. **G01/G02/G03。** G01/T3 INCOMPLETE；G02 OPEN/P1；G03 OPEN/P2。完整
    Scope/Normalizer 未实施；等待 G02 不能自行补足未注册的文档身份输入。
31. **F01–F04。** F01/F02 CLOSED；F03/F04 OPEN/P2，不新增关闭声明。
32. **B01–B07。** 原七项仍 BLOCKING，不与新的 C2-GAP-01 或 R3-NG-B01混淆。
33. **R3-NG-B01。** 原严格 Bytes lowering 修复不变，原 Native/F1 独立回归
    重跑后维持 CLOSED — PRODUCTION LOWERING FIX VERIFIED，不修改旧 Oracle。
34. **C1-GAP-01。** FORMAL BINDING REGISTERED，不重开九成员 Raw Subject
    问题；但不能宣布 RAW PRODUCER VERIFIED。新 C2-GAP-01/P1 是独立同内容文档
    occurrence 身份表达缺口，用户已确认以 Outcome C 交付。
35. **冻结。** 58 Specifications/479 Rules/478 covered/1 intentional exception/
    0 missing，ABI775/AFIDL17/HashV1 domains14 不变。2047个原路径 raw SHA256
    全冻结，唯一原文件变更是 Cargo.toml 增加诊断 test 注册。原 Expected、Oracle、
    正式语义内容身份、ACE1、Validator 均未改。
36. **工程门禁。** 完整代码 b529f8f 上运行 fmt --check、Clippy all-targets/
    all-features -D warnings、cargo test --all -j2、spec-check。最终帐本记录真实
    exit/434 Cargo pass records（含21条复用fixture，不是434新用例）及完整日志哈希。
    代码通过不能代替缺失的 Full-field Oracle。
37. **生成器。** 最终证据提交后，在原 E:/Projects/agent 连续运行真实全部35个
    tools/gen-*.py 两轮；要求70次退出0及全非忽略路径/原始字节哈希
    baseline=round1=round2=final tree，增删改全0。仅最终帐本实测确认后成立，
    不用隔离目录或提交前快照替代。完整映射和日志使用本阶段 target 前缀。
38. **Capability Denial。** 保留 Native/direct/private/alternate 和 Portable D4
    的拒绝测试。Registered 不代表 support/activation；默认 ResourceLimit 与显式
    有界16MiB scalar准入分开，默认生产限制不改。新诊断不签发 RootExport。
39. **阶段裁决。** Outcome C — Additional Formal Binding Gap，不是 Outcome A。
    新 Raw Producer/Full Oracle/Required Field Conformance 均未完成，不宣称零缺口。
    没有 observed Production/Formal false-Proven，不能把它归成 Outcome B/P0。
40. **后续准入。** R3/Native Full Seed/Total T3 INCOMPLETE，T4 NOT ELIGIBLE。
    需独立规范裁决真实文档 occurrence/owner 绑定才能讨论恢复；本阶段不设计或
    注册新模型。未进入任何其它 Producer、T4–T8、G02/G03、D10、Publication或Runtime。
41. **提交/停止。** 分类提交为 b529f8f 测试/独立审查，加本报告/最小证据/冻结
    清单提交。最终完整 HEAD、工作树干净及未 Push 以最终运行帐本与 git status
    核验。用户未授权 Push，未推送；交付后停止。

## 开发记录与核验边界

首次诊断编译因原生类型无 Serialize 失败，改为保留实际完整反序列化输入和原生
完整 Eq 断言；不修改生产类型。首次 Clippy 要求显式 raw borrow，修复仅在新诊断
测试中。原始失败日志独立保留，不能拼成最终成功门禁。冻结清单的工具编排有一次
读取长度命令书写错误，修正后才生成文件，不影响任何旧内容。

首次提交遇到旧 .git/index.lock；核验无 Git 进程、锁文件为旧的空文件后，将其
移至 target/phase-04c1b4c4r3c2-stale-index.lock 保留，再正常提交。未重置索引。

运行证据前缀为 target/phase-04c1b4c4r3c2-。提交内的 gate-index.json 是门禁条件
和证据定位合同，不预填尚未运行的成功结果；最终 final-runtime-results.json 才是
最终提交树的实际运行帐本。冻结模型不足的结论不因工程回归成功而改变。
