# R5G2R3 — 构建与执行分离的合法性

结论：可以设计分步证据协议，但**现有批准仍要求原完整Cargo Gate**。本轮没有获准以“预编译EXE运行成功”替代新的Cargo编译通过；分离方案和实际步骤均 **Not Executed**。默认后继仍执行完整原命令，除非单独批准精确等价实施。

建议记录五类步骤：冻结输入/原始字节验证；精确profile编译；完整测试执行；独立Python Oracle与否定权限；最终机器封存。每个步骤各有来源、命令、退出、时序和资源回执，上游未完成不能标记下游PASS。整体PASS须满足所有原门禁的合取。

必须从对应冻结manifest确定workspace/package/default target/显式tests、features与自动发现规则，不把当前HEAD的target列表套用历史树。例如当前af-portable设置autotests=false且显式注册测试，说明“扫描tests目录就认为执行全部”不可靠。单元、集成、二进制、相关example、doc-test及零测试组都必须如原合同记录；不能改用--lib、--test选子集或忽略零输出组。

`cargo test --no-run`只意味着不运行测试，不能未经核验便宣称已预编译全部doc-test。[Cargo test文档](https://doc.rust-lang.org/cargo/commands/cargo-test.html)说明rustdoc按文档片段生成并编译测试。因此执行阶段可能仍启动rustdoc/rustc；执行前后构建图和进程回执须识别实际编译。出现未批准的编译类别即停止，不能套测试预算绕过编译限制或自行升额。

build scripts、proc macros、依赖、动态库、PDB、生成sidecar和运行fixtures都属于完整产物/输入闭包。必须携带有效cwd、CARGO_MANIFEST_DIR以及嵌入路径依赖。即使只换执行环境，也须认证OS/ABI/系统库及路径等价、传输前后产物哈希和独立执行者。优先同一隔离环境完成构建与执行，不引入不必要的跨机重定位。

历史1至8组仍各自执行完整Cargo；3、7、8附加其原冻结audit脚本（参见机器计划），其余组的独立验证由原Cargo target内Oracle和原冻结验收定义覆盖，不能借不存在额外Python命令而跳过其Oracle。G3另外要求production_input_graph --all、--mutations和精确冻结auditor的receipt检查，保留154正例、26负例/边界及59比较器变异分母。所有原始Actual、Expected、未比较字段及权限拒绝仍独立核验。

计数按可信CaseId集合并集验完整性，分步重复执行不得重复计数；跨组同名需命名空间。Artifact可复用仅能记“原批准产物复用+新执行”，不能生成“新构建成功”。原Cargo构建要求若无法由复用满足，必须重做，不得仅凭来源链省略。

允许性划分：精确冻结源码和原批准历史对应关系已经存在；分步记录本身不更改语义。取消原命令、跨机执行、路径/非确定性等价、重新分类编译保护预算均需后继单独批准。本轮没有发现强制规则互相矛盾，也没有以资源不足宣称绑定模型缺失。
