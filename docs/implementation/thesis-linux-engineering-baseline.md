# 毕业设计 Linux 工程基线

本轮完整门禁：fmt PASS；Clippy PASS；workspace tests PASS；spec-check PASS。

- 原生测试：446 通过，0 失败，0 忽略，0 过滤。
- 文档测试：2 通过，0 失败，0 忽略，0 过滤。
- 合计：448 通过，0 失败，0 忽略；79 个实际测试目标/文档测试组。
- 最终命令耗时：fmt 2.24s，Clippy 84.07s，workspace测试含冷编译 980.50s，spec-check 27.62s。
- spec-check：74份规范、531条规则（530覆盖、1条明确豁免），缺失覆盖0；17个AFIDL模块，交叉引用错误0。

产品改动只有将 HandleProtected 分支调整为 rustfmt 要求的单行形式，保留语义。
AGENTS.md 改为连续开发模式；新增项目专属开发执行入口与固定命令白名单。
没有删除测试、降低 Clippy 标准，或修改正式规范/Expected/Oracle/Registry。
Cargo测试profile设为opt-level1、调试信息level1，并显式保留debug-assertions和overflow-checks；
这只调整工程编译性能，没有修改正式规范中的执行Profile或产品能力。

实际执行命令：
```
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --offline -j 1 --all-targets --all-features -- -D warnings
cargo test --locked --offline --workspace -j 1
cargo run --locked --offline -j 1 -p af-spec --bin spec-check -- check
```

首次测试尝试因实测串行向量耗时过长，由代理通过受控执行器取消并保留全部日志。
第二次完整尝试使用2个测试线程、整机10% CPU，但仍触及90分钟超时；均未记为通过
或产品断言失败。opt-level1/full-debug构建又触发采样式12GiB磁盘保护，未执行测试，
同样未冒充产品错误或通过。随后保留行级调试信息、调试断言和溢出检查，减少冗余
类型调试符号，在相同上限内重新完整验证。两轮清理约5.8GiB和6.6GiB可再生的
项目构建缓存；源码、锁定依赖、旧快照、已有图像中间结果和执行日志均保留。
最终测试计数仅来自最后完整完成的一轮。fmt/Clippy最初使用5% CPU和1线程。
全部执行保持单进程8GiB地址空间、任务树6GiB、Swap0、PIDs128、并发重型任务1、
每命令90分钟、主机内存保留8GiB、文件系统剩余20GiB和任务累计新增磁盘12GiB阈值。
B2采样不是硬磁盘配额。未重复破坏性资格测试，也未修改共享系统配置。

完整实际argv、退出码、耗时、日志SHA256、资源观测及测试目标清单保留在
`target/thesis-fast-dev/summary.json`和各次执行原始日志内，不再逐次生成大型审计报告。
测试基于起始提交bb2857c加工作树修复执行；提交前验证最终Rust构建输入与通过时一致。
最终提交身份另存`target/thesis-fast-dev/final.json`，不冒称门禁在事后提交HEAD上运行。

十个历史快照使用已有身份清单和干净Git状态确认未变，未反复全量扫描历史路径。
旧Windows历史仍0/8，正式能力边界不变；本轮没有启动Windows构建、RootExport、
Publication、Runtime或图形编辑器功能开发。工程基线通过不等于这些功能已实现。
只做Linux本地提交，不Push。下一步可在此基线上开展最小可操作工作流演示。
