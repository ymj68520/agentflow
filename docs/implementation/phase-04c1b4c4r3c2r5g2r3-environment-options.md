# R5G2R3 — 验证环境方案比较（设计裁决）

所有实际环境方案均为 **Not Executed**。本轮不安装、连接或调用其它机器；不运行 Cargo、rustc、生成器。正式身份、历史配置、Expected 和权限边界不变。

| 方案 | 现有证据 / 主要限制 | 裁决 |
| --- | --- | --- |
| A 当前 Windows + 缓存 | Edition-1 两次 reserve79；冻结 G3 Debug2 LLVM OOM101；121 次/600 秒观测未满足稳定14GiB。没有完整缓存来源链，也没有在现行限制下完成全部门禁的证据。 | 未证明稳定窗口，NOT ADMITTED；不再自动探测或重试。 |
| B 隔离 Windows x64 MSVC 验证环境 | 可以设计冻结输入、工具链、环境、资源、执行者及输出的独立证明；尚未选定或认证任何环境。更多物理 RAM 不能消除仍受768MiB限制的单进程分配失败风险。 | **唯一推荐，条件性方案；NOT ADMITTED。** 需独立系统预算及编译进程预算、具体环境与等价条件审批。 |
| C 可信冻结产物复用 | 旧R5日志说明当时命令曾成功，不等于本轮完整门禁已重做。审查的缓存元数据和日志不能认证每个EXE/PDB/依赖的完整 Source+Toolchain+Profile 来源。R2冻结G3编译未完成。 | 当前审查集合中没有已合格的可复用完整产物包；不是对全盘所有缓存的穷尽否定。复用只能按明确批准的等价规则记账。 |
| D 修改历史配置 | Debug2→0、缩小测试集、改Expected、减少CaseIds都改变冻结验收条件。 | 默认拒绝等价验收；本轮不改任何参数。潜在非语义调整也需逐项证明并另行授权。 |

Option C 必须同时具备原始构建成功回执、完整输入/工具链/有效配置哈希、全产物及运行依赖哈希、源和产物不变性、原执行者证明，以及允许复用而非新构建的正式批准。缺任一项即失败关闭。当前HEAD的Debug0产物不能代替历史Debug2，也不能借完整源码相同而忽略工具链/配置不同。

旧构建的“已完成”与本次“需完成”分开：旧R5八组成功记录保留为历史来源；新要求实际历史完成仍0/8。被冻结G3新的完整Cargo、独立Oracle及证据比较尚未完成。缓存文件存在不代表 Cargo 将不编译，亦不代表 doc-test 已覆盖。

原命令、八组 HEAD、Debug配置、CaseIds及独立检查见机器清单和historical-execution-plan.json。新环境尚未证明满足这些条件；不以设计推荐代替可行性认证。

技术依据：[Windows Job limits](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information)将进程/Job提交内存分别约束；[Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)说明调试信息及配置覆盖。因此主机容量、进程限额、有效构建配置必须分别核验。
