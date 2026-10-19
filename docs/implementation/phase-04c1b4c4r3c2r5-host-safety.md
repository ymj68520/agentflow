# R5 主机安全与失败归档准备

所有昂贵验证沿用 G3 已验证的 Windows Job Object：单顶层任务、CPU 硬上限
5%、单进程已提交内存 768 MiB、整 Job 1536 MiB、BelowNormal、测试线程 1、
Cargo Build Job 环境 1。用户要求的 `-j 2` 命令仍受整个 Job 的共同限制。
内存负载到 80% 或可用物理内存低于 8 GiB 时终止，不把 OS 中断当正式通过。
没有修改 Windows、驱动、分页、杀毒或持久电源设置，没有放宽限额。

新 Raw 边界仅使用普通深度的原有声明、空 Native Catalog 和三个实际模块缺失
样本；未构造超深输入，没有再调用已知 `type_expr_shape` 的指数分配路径。
旧摘要编码器的独立风险没有在本轮修复，也没有变更 ACE1。

历史第八组准备的第一次尝试失败：验证编排辅助脚本将整个 `git archive` 的
stdout 通过 Python `check_output` 读进内存，触及 Job 的单进程上限，报
`MemoryError`、退出 1。最大已提交进程内存 790,257,664 字节、Job 891,998,208
字节，最少可用物理内存 14,721,630,208 字节。没有历史构建因此被计作通过。
失败日志与封套保留在 `target/phase-04c1b4c4r3c2r4g3-safe-r5-history8-prepare.*`。
这不是正式 Proof ResourceLimit，不是 Native/Oracle 语义反例，也不是主机重启。

该全量缓冲路径已停止使用。修复只涉及忽略目录中的验证编排辅助脚本：Git
直接输出 tar 文件，Python 流式遍历并逐成员验证安全相对路径后解包；Git Blob
和原始 SHA256 以 1 MiB 分块计算，不把整个 Archive 装入 Python 内存。
不覆盖已存在的部分源码树；只在确认为空的本阶段专用目标中首次解包。
随后从头运行完整八组历史门禁，失败准备日志不与成功记录拼接。

流式准备重跑实际退出 0，耗时 54.02 秒；峰值进程已提交内存 136,445,952
字节、Job 163,409,920 字节，最少可用物理内存 13,854,887,936 字节。
成功记录为 `target/phase-04c1b4c4r3c2r4g3-safe-r5-history8-prepare-stream-r2.*`。

第八组首次冷编译也失败：原 `cargo test --all -j 2` 在编译
scope_closure_conformance 时 LLVM 报 out of memory，Cargo 退出 101。进程峰值
798,748,672 字节、Job 1,542,209,536 字节，最少可用物理内存 10,663,260,160
字节；OS 硬限额有效，失败不计通过，日志保留为 `safe-r5-history-8.*`。
重跑仅将第八组构建并发改为 1，并以命令行临时配置关闭 dev/test 调试符号。
不改变优化、debug assertions、features、测试选择或源码，不修改 Cargo.toml，
不改变规范 Compiler Semantics Identity、Registry 或 Expected。全部测试重跑；
最终回执明确列出该组与前七组的构建配置差异。硬限额仍为同一配置。

生产代码、G3 核心、测试输入、Expected、Registry 和已完成代码门禁的源字节
没有因这项编排修复改变；此前完整代码门禁仍绑定其确切代码 HEAD。最终回执
分别记录成功代码门禁、失败准备、修复后完整历史组及生成器双轮，不预填成功。
