# P1 — Concrete Environment Qualification（NOT AUTHORIZED）

本文件是停止在P0的操作计划，不是执行指令。目标机器、所有者、执行者、批准、运输边界及生产信任根均未提供，不搜索、选择或连接任何主机。

1. 取得EnvironmentApprovalRequest的具体EnvironmentId、用户批准身份/范围、所有者与执行权限、Windows x64/MSVC目标、网络和源码/日志运输边界。缺项即REQUIRED USER AUTHORIZATION。P0候选中null不是localhost，也不是默认云实例。
2. 独立固定本阶段schema、验证器、Oracle、契约、完整要求分母及工具依赖哈希；用户/独立认证者在清单之外固定Evidence Trust Policy、公钥、执行者身份、撤销与重放规则。Manifest自带key、批准字符串或trusted=true不授信。
3. 只有得到接入权限才允许读取指定环境。认证OS/build/ABI、rustc/cargo/rustdoc/LLVM/MSVC/linker/SDK/Python精确字节、Cargo配置层/env/flags、全部锁文件与依赖/离线缓存来源。版本字符串和.rustc_info缓存不能代替完整身份；本地审计Python包身份不能冒充目标环境。
4. P_build/J_build仍null。先批准有限资格试验的硬额度、CPU、系统reserve/commit/物理窗口、时限、磁盘/日志限额、监测与停止规则，才能测量；不允许无界探峰或自动加额。测试P_test/J_test和8GiB reserve保持旧批准基线，除非明确另行裁决。旧主机14GiB/600秒不是新机器普适准则。
5. 认证隔离与权限、只读冻结源、独立输出目录、防缓存交叉污染、依赖预审/网络、任务所有权和保护器实际生效。任何资格试验可能涉及编译时，必须在P1批准范围内单独列明；P0从未授权这种试验。
6. 依据EnvironmentEquivalenceAudit逐条取得“已证明/需资格/需批准/不支持”的证据。只允许既有精确历史逐路径LF/CRLF对应，不做通用归一化。路径/CARGO_MANIFEST_DIR/fixtures/大小写/Unicode/链接/绝对嵌入信息都必须核对；未知差异失败关闭。
7. 资格执行者签名覆盖完整canonical payload、包哈希、nonce/ExecutionId/环境/政策/输入/输出；原始字节、长度与哈希先验。私钥不进入项目或日志；真实信任链必须独立建立，公开RFC向量绝不登记为生产根。

通过P1也只记环境资格，不得记八组历史PASS、冻结G3完成或注册完成。签名有效、来源可信、执行结果可信和正式权限五者分开审定。P2仍须另行明确授权；本轮停止在P0。
