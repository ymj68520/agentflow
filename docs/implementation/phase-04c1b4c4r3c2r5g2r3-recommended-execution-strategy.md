# R5G2R3 — 唯一推荐策略与可行性裁决

选择 Option B：隔离且经独立认证的Windows x64 MSVC兼容环境，精确冻结源码/配置，各历史组独立工作目录与构建目录，构建、执行及Oracle优先留在同一认证环境。**Not Executed；NOT ADMITTED；未实施完成。**

当前限制为CPU5%、单进程768MiB、Job1536MiB、系统物理reserve8GiB、BelowNormal、Cargo jobs1、测试调度1、单个顶层受控Job。它们是本阶段必须保持的操作约束，不可自行解释成“大机器可忽略”的保护建议。历史Debug2仍为2，只有第8组保留其原批准Debug0。

两个独立预算问题：

1. 系统物理内存：在实际背景负载下，须认证持续安全余量、可用物理/commit/负载指标、采样间隙、暂停/中断阈值及操作者响应。R2的14GiB/600秒只是当时启动准入规则，不是峰值上界证明；不得自动调低。RAM规格表不能证明稳定窗口。
2. 编译分配预算：768MiB附近OOM证明当前方案不能直接认定可行，但不提供真实完整峰值。设计变量 **P_build / J_build** 仅表示待批准的编译进程/Job保护策略；没有赋新数值、没有授权使用。需先由用户批准有限的资格试验策略（包括硬上限、reserve、CPU、时间、输出大小、终止规则），再在该界限内测量。不能先无界运行以“测出”额度。测量仍失败即停止，不自动上调。

在完全维持现行上限时，尚无证据证明推荐环境能够完成；本轮结论为可行性**未证实、不可准入**。这不构成规范矛盾或缺少正式绑定。

后继资格流程（全部Not Executed）：用户确认环境及政策→验证执行者/OS/ABI/工具链/依赖和隔离→认证完整源码与所有必需CaseId/target/artifact分母→按已批准保护策略验证系统窗口及有界编译可行性→逐组完成原Cargo及独立检查→逐字段验证证据与否定权限→独立复核→封存。任一步失败/未知即NOT ADMITTED，不推进后续组、不自动重试。

即使批准编译保护修订，也不改变Rust生产语义、Runtime资源维度或任何RootExport/Portable权限。运行测试仍用原限制；若运行步骤触发rustdoc/rustc等编译，必须停止，或在事先明确批准的编译类别中运行，不得临时升额。

资源失败见resource-failure-analysis.json。完整构建契约与来源哈希见verification-contract-inventory.json。R5G2仍Registration Incomplete，Historical Builds = 0/8。
