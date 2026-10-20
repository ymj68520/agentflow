# R5G2 — Native Source nominal registration staging

起始HEAD `0e94c9f6a2084aad996ffc410d180abbf4be1d38`，起始2368个非忽略文件的
原始SHA256在target同阶段start-raw-map.json，开始时工作树干净。

候选ID经Registry、规则及完整Model7审计无冲突后分配：Analysis@9 / Model@8 /
Report@8 / Obligation@8。新增规范源4、规则9（AF-SEMANTIC-0315..0323），总计
74Specs /531Rules。旧70Spec与522Rule对象逐项完全相同；旧规范源与旧Expected原字节保留。
不新增ABI、AFIDL、HashV1域、ACE1语法、ModelType rank、NodeBody或failure Variant。

只改五个声明：ContractProofContextContents、StructuralValue0030..0033；四个Site的
闭合描述同步变化。三个Some精确指向NativeRelationSnapshotSource，不将原Alias cast。
完整Named Descriptor仍1131、Refs2559、历史Import97、Sites317、Policies66。
Source六字段与Native Eq/pin/bytes/revision/catalog、真实C与port语义不变。

用户明确选择显式Analysis9注册暂存入口，旧Analysis8兼容入口固定完整历史身份。
旧接口名current_*在此兼容实现中不代表新版current；其生产Input-G域仍只属于旧8/7。
新native_source_staging模块只查询/校验规范metadata；不接入input_binder/input_graph，
不能返回VerifiedInputGraph、Native receipt、CompleteView或ObligationIdentity。

规范DAG测试是限额内手写Source完整子树，其Rank是注册样例数字，不是生产DagRank。
新增私有metadata verifier由原structural_graph_metadata机械版本隔离，算法/metadata
限制不变，仅绑定显式Model8上下文；没有复制或修改生产图构造器。

当前源审计、声明读取、Source完整性、1297字段/Variant样例、Source子树正反例及
独立Expected/Actual已验证。完整工程、全部精确历史和原目录生成器门禁另行封存，
没有完成回执前不得宣布COMPLETE — REGISTERED STAGING。

原G1 49向量保持原文件和Not Executed状态，本轮没有逐项转换记账；新的注册CaseIds
与生产capture credit严格分开。新版本生产Binder NotImplemented，生产DagRanks0，
Raw Full-field/Native Full Seed/Standalone/OtherT3/TotalT3 INCOMPLETE，T4 NOT ELIGIBLE，
RootExport/Portable positive proof NotAuthorized。G02/G03及其它既有发现不关闭。
