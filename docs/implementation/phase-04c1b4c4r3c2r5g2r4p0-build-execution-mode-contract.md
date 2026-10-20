# P0 — 两种执行模式合同（均Not Executed）

模式一是原完整Cargo Gate。以historical-gate-contracts.json和frozen-g3-gate-contract.json的完整HEAD、原命令/cwd/env、独立源树/构建目录及原Registry/Compiler/Expected/Oracle为准。历史1..7 Debug2、第8原批准Debug0、冻结G3 Debug2；不得改--all为--lib或筛选测试。

模式二是“受控编译→可信产物→完整测试→独立Oracle→封存”的额外证明设计，**没有替代模式一的批准**。必须另行批准分离规则、路径/环境等价和运输；已有合同若要求完整Cargo，则模式二只能补充。预编译产物运行不产生新的Cargo编译成功记录。

静态TOML census保留每个历史包的显式targets、autotests等发现规则、tracked测试源、lib/doctest声明、features、build scripts/proc macros及依赖。它不是cfg展开后的实际执行名单，也不声称每个bench/example都被Cargo默认命令执行。后继须在批准限额内认证实际target/test/doc-test清单，与独立冻结要求逐项比较；不能用两侧交集缩分母，不能重复计数CaseIds。

`cargo test --no-run`不自动证明doc-test已全部预编译；rustdoc可能在执行阶段编译片段。任何编译必须事先归入已批准的P_build/J_build步骤，否则停止，不能临时升额。所有EXE/PDB、动态依赖、build outputs、fixtures、CARGO_MANIFEST_DIR及路径映射须进入产物闭包。各Edition不得共享未经独立认证的target/缓存。[Cargo test说明](https://doc.rust-lang.org/cargo/commands/cargo-test.html)是该区别的技术依据。

P0没有运行两种模式，也没有连接机器、复制产物或授予Native/Portable/RootExport权限。P1环境合格也不自动启动P2；P2必须另行取得明确门禁执行权限。
