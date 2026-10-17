# Independent conformance：明确区分局部通过与完整缺口

tools/oracle/raw_port_capture.py 从实际声明输入、冻结 Descriptor/规则、独立
AFIDL 编码器导出 Expected；不调用 Rust 来生成 Expected，不解析最终错误原因。
Rust 只输出 Actual。Full Raw Conformance 入口缺字段即失败，不能由 CaseId 数量
或局部 Expected==Actual 替代。

87 个真实输入含三种 Owner 容器、Physical/Native、数值 U32 排序、稳定重复、
Input/Output 独立命名空间、两个 zero/duplicate 分支、真正初始哨兵、先行 Identity
失败、Schema 在 P 之后失败、同内容不同 Owner/Slot、同来源复用和多合法观察。
九个非空十三字段声明用 zero 的早停保留各类 PrimitiveConstant 原始值，包含
NFD String、原始 Bytes 拼写、i64 极值、Decimal/Timestamp/Duration。
它们只证明 Raw 内容保留，不宣称执行或完成这些其它维度的 Producers。

63 个来源准入成功输入中，59 个包含真实 Port 事件，独立适用检查共 125 个；
该数仅计主 Capture；Owner/Slot/重复来源的额外重跑只完成关系断言，没有独立
序列化全字段闭包，不能把主事件零缺口称为整个 Raw pipeline 的完整覆盖。
正向内部文档 Entry 在本次主输入中均为 0，非零 Entry 的生产正例仍需补齐。
另外四个是三个空端口输入和一个更早 Identity 失败。24 个真实来源负例在
Predicate 前失败。Current 254 注册 CaseIds 和 88 注册站点没有计入这些数。

Required Event 清单独立于 Rust Tape，按完整输入/先行停止原因推导。
实际事件带与密封 Seeds 一一对应；额外、重复和双侧共同删除都拒绝。
Comparator 35 项变异覆盖状态/条件/ordinal/namespace/前驱/Witness/Rule/Schema/
Owner/Slot/文档/候选/角色分区/观察/假 Scope 与 Root 能力，以及双侧字段删除。
Rust 内部另有 8 项 Collector Payload 损坏，不能混称为新增 CaseIds。

## 可执行 Field Contract

正式 Fact 14 个根字段直接提取自 model@5，不从 Actual 字段集反推要求。
局部映射仅 4 类，非完整闭包证明。未物化的 10 类为：

key、prerequisites、references、primary、contradiction、cause_fact_keys、
companion_evidence_status、observations、substitution_basis、native_guard_basis。

125×14=1,750 个必需根字段实例中至少 1,250 个尚未闭合；还存在完整 Compiler、
九成员 Context 独立投影、Witness.subject、ModelType 路径和正式引用链等嵌套缺口。
局部投影字段比较计数由运行账本给出，不能作为正式 Required Field 全零缺口。
本次没有填写“未知字段默认空数组/None”、空 Resolver 或伪 Scope 来消除缺口。

默认严格 Oracle 即使局部 Actual 全匹配也退出 2、Outcome D。
`--report` 退出 0 仅提供局部诊断；Rust 测试同时断言严格模式非成功。
这是失败关闭回归，不是完整 Oracle 已通过。没有发现新的不可表达正式模型字段，
故不是 Outcome C；完整字段仍是当前实现/验证未闭合，不能宣布 Outcome A。

## 开发失败记录

首轮 Option 输入拼写错误在准入阶段拒绝全部案例，未计为 Capture 通过。
另一轮“foreign domain”测试因 Python 输入对象共享 domain 导致同时改了包与
Owner，实际仍是合法匹配域。独立事件清单拒绝这次错误 Expected；保留失败日志，
修正新输入的有效变异，并加入任意合法声明域名的正例。不是修改旧 Expected，
也不是生产 false-Proven；该合法来源下实际 Port 结果本来就是 zero Violation。

原 C2G1 48 向量文件完全冻结，原 Not Executed 标签不改。旧 24 项 metadata-only
对应关系不自动升级；本次没有完整生产 Conformance 关闭声明。
