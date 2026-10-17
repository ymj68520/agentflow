# Phase 4C.1B.4C.4R3-C2G2 — final report

目标裁决：Outcome A — Document Occurrence Registered Staging Accepted。
仅在最终运行帐本确认全部必需门禁通过后成立。C2-GAP-01正式绑定已登记，
不等于Raw Producer已实现；本阶段没有正向RootExport激活。

## 按请求顺序的49项交付

1. **HEAD。** 起始e623aab705c5f8269807201e112a32e67e6869bf、初始干净。
   完整代码门禁4ce26eb5ce32d291d638ddb7a6d1c2fd602fa385；最终完整HEAD及退出
   见target/phase-04c1b4c4r3c2g2-final-runtime-results.json。最终文档提交后才运行
   原目录双轮，并核对最终树，运行帐本避免提交哈希自引用。
2. **安全暂存。** 先审查Current-only/ModelRef/D4/Native/direct/private/alternate/
   profile/release gates；Registered与NotImplemented/NotAuthorized独立，无原子激活依赖。
3. **SourcePackage。** 完整domain+owners序列；owner={anchor,container}，三种
   完整实际容器及document。显式Caller输入，不在Collector中反向包装候选。
4. **Anchor。** 完整{domain:{namespace,domain_key},owner_key,revision_key}，
   全部exact非空UTF8 bytes；完整typed比较，不是一个document_id字符串。
5. **域语义。** 每键1..4096字节、无NFC/大小写折叠，scope限定实际包/联合包；
   immutable revision来自实际声明，不声称全球唯一、组织所有权或已有外部认证。
6. **实际Slot。** U64真实declarations/imports/sources序列位置，不是内存分配号、
   声明内部entry、Port ordinal或Report index。不得偷塞旧source_ordinal字段。
7. **Occurrence。** O=(完整Anchor,真实document-entry ordinal)；完整包与完整
   文档仍保留。内容身份与O分离，不以digest/长度/显示值代替。
8. **同内容不同Owner。** 两个合法Owner/slot准入，完整内容相等而Owner不同，
   输入O投影不同；正式Source/Origin闭包可表示该差异，不宣称已生成canonical FactKey。
9. **同Owner不同Slot。** 真正slot0/slot1的相同文档可准入，O不同；本地指针只
   核验关联，不序列化成Portable identity。
10. **重复使用。** 同Anchor/key/content重复准入不创建新来源；API无分配ID，
    同一实际输入的O投影不变。不同完整Scope/Compiler/Context不强行合并Fact。
11. **冲突。** Duplicate Owner行Malformed；同O不同文档BindingConflict；显式
    联合包同Anchor不同完整OwnerRecord为OwnerBindingConflict，逐全包检查。
12. **缺失失败。** 缺包/Owner/revision/slot、越界、错文档/借用/候选、unknown
    variant/domain/字段及不完整内容明确拒绝；可先IncompleteFacts/UnsupportedModel，
    不转成Predicate V/U，不造随机/指针/call-counter/UUID/default Owner。
13. **权威隔离。** DocumentSourceAssurance仅declared_only；可信名称也不认证。
    authenticated/supported额外字段拒绝，DurableHandle未支持，不发外部可信所有权。
14. **Physical。** 完整原始bytes、实际包/Owner/slot、严格UTF8/重复键/结构/宽度/
    候选及真实path；supplied关联必须同O/同文档，不重序列化或NFC修复历史字节。
15. **Native。** 完整13字段Raw metadata序列及原条目，physical=not_supplied。
    使用显式闭合inline companions做non-wire metadata关联；非通用wire Decoder，
    也不是受控Native Raw Producer或历史对象归属认证。
16. **不支持矩阵。** native+supplied、physical+not_supplied明确
    UnsupportedPhysicalAssociation；跨O物理关联拒绝，不伪造转换receipt。
17. **Context。** 原九成员、analysis/canonical_validation真实同对象分区不变。
    同值不同对象拒绝，无resolver_dependencies=[]或Context digest/Role代替Owner。
18. **RawInputOrigin。** 新模型最早改为document_binding:Node<OccurrenceBinding>
    +原始内部path；旧model@4完整源及旧语义不改。
19. **Subject。** 完整Compiler/input/context不省略，引用进入新binding闭包。
    Metadata成功只表示关联准入，不意味着canonical或whole Checked。
20. **Source。** Source.raw_validation仍完整subject引用；其闭包当前是新模型。
    其它Root/Native/Substitution Source不重写成Raw Source，不虚构Root/IR/Resolver。
21. **RawOrigin。** input_origin引用新RawInputOrigin，direction/original ordinal
    保留；外部slot和内部成员路径/排序ordinal分离。
22. **Origin/FactKey。** 原字段及引用传播新O，不末尾拼UUID；Source/Origin/
    Witness/Basis/Evidence/Physical均须同一binding，无新完整身份字符串构造器。
23. **Observation。** 同完整key且非观察payload相等才union合法观察/consumer；
    冲突InvalidFacts。模型字段/身份法则测试不是Validator已产生Fact的证明。
24. **最小DAG。** AF-SEMANTIC-0217不变，相等内容共享、不同真实binding不同值。
    有界已注册scalar/ordinal子图与DAG envelope负例执行，完整Scope/Normalizer未实现。
25. **无环。** 输入Package/Owner/容器/Doc闭包100类型，经机械遍历无output/
    Scope/Fact/Witness/Report/Obligation/binding回边；输入输出DAG分离，无固定点。
26. **Port冻结。** 一个positive_unique_io_ports Case，Input-first、U32稳定排序、
    原/排序ordinal、真实前驱和早停止不变；P/Zero V/Duplicate V不变。未改Validator。
27. **迁移。** 旧content-only Source按精确旧Edition；缺真实额外信息
    UnmigratableMissingOccurrence，不拆旧合并事实、不补Owner/slot、不贴新标签解释旧内容。
28. **新正式身份。** 冲突检查后分配Analysis@6、Model@5、Report@5、Obligation@5。
    完整62注册源/36选定源/188applied规则/全部Registry与closed tables进入身份；
    Unknown/旧标签新内容/新标签旧内容都拒绝，F03不宣称依赖闭包完成。
29. **Registry增量。** 实际新增4 Specifications和14 RuleIds0272–0285，当前
    62规范/493规则/492覆盖/1例外/0missing；DO01–14有完整注册与执行映射。
30. **原记录。** 原58 Specification与479 Rule完整JSON对象逐条相等，全部
    旧规范源原始字节冻结，不只比较id或计数。
31. **Descriptor增量。** 新增21描述符/46引用；当前704/2064。21包括14正式
    关系/状态及7完整inline metadata companions；替换3旧类型仅在新owner下发生。
32. **Case/字段。** 当前254注册CaseIds=170shape+14rule+51admission+8DAG+
    11Portable；88/88新/替换字段与Variant站点有双向实际回执，gaps0仅指注册站点。
33. **负向准入。** 实测缺包/Owner/revision/slot、越界、错content/entry/借用、
    同键冲突、duplicate owner、unknown schema/owner、pointer/counter/UUID-only、
    假wrapper缺来源、坏物理bytes、错physical组合/Context/compiler及额外认证声明。
34. **48向量状态。** 原C2G1 JSON保持Not Executed、字节不改。本阶段另存精确
    转换矩阵，只对有对应完整条件和真实回执的24项登记Executed—Metadata Admission
    Only，其余24仍Not Executed。相似DAG/Source/Fact负控不自动升级原向量状态。
35. **四版历史。** 分别在abc91d5/51239a0/4d3bb59/875e6e4精确构建执行
    512/51/173/889 CaseIds，独立Registry/Compiler/Expected；实际Cargo数与退出见帐本。
36. **Analysis@5。** 起始e623aab完整Git blob源另行构建全套，保留原262-case
    注册/准入/C2诊断等；不使用新全局Registry冒充旧回归。五源9542 blobs前后核对。
37. **Native/F1。** 本次当前完整测试产生的Actual再经原Oracle独立53/53，
    F1 26lowering+36native inputs/40mutations，原51完整Actual相等；不是Full Seed。
38. **G01/G02/G03。** T3 INCOMPLETE；G02 OPEN P1；G03 OPEN P2；完整Raw
    捕获/SourcePackage Scope/Normalizer/Producer Oracle均未实施。
39. **F01–F04。** F01/F02 CLOSED；F03/F04 OPEN/P2，无额外关闭声明。
40. **B01–B07。** 原七项BLOCKING；R3-NG-B01保持生产lowering修复关闭及回归要求。
41. **C1-GAP-01。** FORMAL BINDING REGISTERED，九Context绑定不重开、不削弱。
42. **C2-GAP-01。** 正式模型/准入关联登记后为FORMAL BINDING REGISTERED，
    不是实际Raw Producer Verified或新的完整Scoped Fact身份已生成。
43. **冻结。** 2076原路径中2065 raw字节冻结，11审查过的Registry/路由/生成器
    例外；旧Expected/规范/models/97imports、ABI775/AFIDL17/HashV1 domains14、
    ACE1及历史digest意义不动，无端口/Guard/Substitution/Countermodel语义改动。
44. **完整工程门禁。** 固定代码HEAD新鲜fmt/Clippy/Cargo --all/spec-check；
    只用最终完整退出，不拼接focused结果。实际pass records/组数/哈希见最终帐本。
45. **原目录双轮。** 最终证据提交后真实36生成器连续两轮E:/Projects/agent，
    全部72次exit0、全非忽略路径/raw哈希baseline=round1=round2=final、增删改全0
    才验收。失败保留并重新从头双轮，不用镜像、旧文件图或拼接片段。
46. **能力拒绝。** Registered YES；Raw Producer/Scope/Normalizer NotImplemented；
    Native/direct/private/alternate/profile/D4不能因SourceO新增而issue RootExport。
    默认D0资源限制与显式有界D4分别测试，生产默认limits未放宽。
47. **Outcome。** 仅在所有前述门禁闭合后为A：COMPLETE—REGISTERED STAGING。
    不是Raw Capture Complete。若历史/双轮/当前/独立门禁未闭合，不能宣布A。
48. **恢复准入。** 只获得申请单独恢复R3-C2的规范前置资格；不自动实施Producer。
    Native Full Seed/Total T3 INCOMPLETE，T4 NOT ELIGIBLE，Positive RootExport NotAuthorized。
49. **提交/停止。** 5fd6337规范/Registry/闭合语料；d558ad0非权威实现与测试；
    4ce26eb补齐D0/D1负例阶段断言；最终文档/冻结/回执为分类证据提交。最终完整
    HEAD、clean worktree及未Push由最终帐本与git状态核验。完成后停止，不进入禁止阶段。

## 交付索引

同前缀overview/registration-manifest/source-admission/scope-identity/legacy-isolation/
c2-readiness/gate-report合理合并题目建议文档，全部19主题均覆盖。JSON附件为
frozen-inputs、allocation-receipt、descriptor-manifest、design-conversion-actual及gate-index。
权威完整table及Corpus在spec/与conformance下；旧Expected未改。
动态实际结果统一target/phase-04c1b4c4r3c2g2-前缀，final-runtime-results最后生成，
其最终HEAD与最终提交树一起核验，不制造提交自引用。
