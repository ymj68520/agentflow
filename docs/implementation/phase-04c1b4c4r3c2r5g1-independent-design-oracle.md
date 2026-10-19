# 独立设计 Oracle 与诚实执行范围

入口 `tools/oracle/native_relation_source_design.py`，不调用 Rust Actual 来造 Expected。
完整 Model@7 由原规范源合成；required source fields 来自六字段 Descriptor；
site/policy denominator 从完整注册模型重建；原始 Raw 声明来自独立旧输入；Native A 与
current lookup/validation Expected 来自冻结 AFIDL/Native 输入的独立 Python 算法。
Rust Actual 只用于七例真实边界的对照，不决定 required roster 或设计推荐。
设计A投影在独立 native_source() 中仅读取声明的Native catalog、family/revision/pin/bytes，
不调用C或C Expected；declared_context()另保留完整当前模块定义和九分区输入。
All-None向量的真实声明上下文中三个Slot都为None、native_source为null，不把Some输入
冒充None。不同Owner配对各保留其真实原始声明；同A/不同C保留两份完整当前Context。

三层输出严格分开：

- current census / current boundary：现有 Model@7 可验证的真实结果，Source 节点仍0。
- proposed future recipe：完整 Native source 字段、名义目标、实际来源坐标、合法根路径、
  source-stage、rank dependency、C-source domain/target 和失败关闭的手算 symbolic Expected。
- production successor graph/rank/registration：Not Executed / 未实施；无真实未来 DagRank。

49个向量，四组12/12/13/12。初始全为 Not Executed；结果保留 production/registration
Not Executed，另列实际执行的 symbolic recipe comparison。旧七例诊断在独立记录中，
不把设计向量改记为49个新生产 CaseIds。38个独立 source-derived omission 检查包含
双方同时漏 source、六个单独字段、nominal、source association、root、reachability、stage、
rank dependency、C target/domain；双方相同缺失也必须 REJECT，不能从双方 view 求分母。

每个设计向量保存实际既有声明/Native input/Owner/Slot、registered Expected、future
Expected、具体 proposed candidate/paired input、executed/unexecuted scope、规则与裁决。
负例在候选 recipe 中实际删除/更换字段；不是仅打一个 reject 标签。重复来源、另一既有
Owner、三 Slot、同 A/不同 C 配对保留；它们是设计重放，不假冒 Rust 已执行新 native association。

Oracle 的 exact comparison 验证本设计契约，不声称对未来完整 DAG 做了 codec/rank 全字段
执行；可达性/最小性/rank 的完整设计证明在独立文档，具体生产必须在正式注册后独立实现。
此阶段并不修复 G02、T4、完整 Raw producer 或旧 schema digest 编码器资源风险。

最小真实诊断复用冻结七例，重跑 cargo 当前测试后，从 Source/AFIDL 重建完整 G Expected，
独立 strict decoder 比较每节点/边/rank/root/字节，再逐字段比较真实 C 与 Port 输出及
原 R5 frozen Actual。C metadata 的老算法 header@7/model6 是 relation-only，不升级为
Analysis@8 Subject capability。Boundary46 mutations 仍仅计 boundary，不计设计生产信用。
