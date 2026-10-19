# R5G1 门禁证据的读取方式

本文件只固定门禁合同，不预填未来成功。实际退出码、HEAD、开始时间、日志 SHA、
资源 readback、计数与最终 Outcome 由本轮运行结束后生成的回执提供：
`target/phase-04c1b4c4r3c2r5g1-final-runtime-results.json`。
缺失/未完成/非零失败回执不可解释为通过；历史成功不替代本轮执行。

完整串行命令入口：

```powershell
python -X utf8 tools/audit-native-relation-source-design.py --quality
python -X utf8 tools/audit-native-relation-source-design.py --independent
python tools/run-production-input-graph-safe.py r5g1-freeze python -X utf8 tools/audit-native-relation-source-design.py --freeze
python -X utf8 tools/audit-native-relation-source-design.py --generators
python tools/run-production-input-graph-safe.py r5g1-final-seal python -X utf8 tools/audit-native-relation-source-design.py --finalize
```

Scheduler 仅等待单个子任务及流式 SHA/写回执；每个 compiler/validator/Oracle/generator
都由原受限 Windows Job runner 执行。CPU5%，process768MiB，job1536MiB，test threads1，
CARGO_BUILD_JOBS=1；显式 cargo `-j2` 保留用户要求，OS限额不放宽。无并行验证任务。
主机物理内存80%或available<8GiB时停止；中断或OOM均不得通过。失败日志原样保留。

quality 保留四条原命令：fmt check、clippy -j2 all-targets/all-features -Dwarnings、
cargo test --all -j2、spec-check check。当前源未改，仍需真实重跑，不引用旧446测试代替。
Rust quality 起始 HEAD 与完整 Python/独立对照代码门禁 HEAD 分开保留。静态审查发现
冻结 boundary Oracle 把七行进度写入stderr，安全日志合并后不能按裸JSON解析；仅修正
新 scheduler 的日志读取，不修改冻结 Oracle。完整 Python 代码在其后独立门禁执行。
实际 Rust/规范源在两 HEAD 之间及最终 HEAD 均必须零差异，不重标早先运行的 HEAD。
新设计Oracle第一次严格落盘回读因内存selector为tuple、JSON为list而退出1；该失败与
日志保留，不计通过。修订只让源枚举器直接产生注册selector格式的JSON数组，不放松
exact comparator，不从Actual修补Expected，不改任何旧Oracle/规范/生产。重跑使用
AF_R5G1_GATE_ATTEMPT独立日志后缀，避免覆盖失败和前轮成功回执。

independent 必须使用 quality 开始后新鲜产生的 Rust Actual：current registration3662、
Native53、F1 lowering26+native36=62及40mutations、G3 154正/26负/59mutations、
R5 boundary7/46mutations。新 Oracle 重建全模型 census、49个设计向量、38个 omission
检查和七例完整边界；没有给未来注册/生产向量执行信用。新旧 Strict Raw Total 必须退出2。
Capability denial 来自真实 current3662/auditor和现有私有构造/直接调用/portable拒绝，
加七例 G/C/port issuer 均不能 RootExport；不通过伪造“完整 Raw Seed”来测试能力。

历史八组：`Not Rerun — Referenced R5 Frozen Evidence`，仅因本轮所有原有源字节冻结。
旧schema摘要编码器的深层指数资源风险仍存在，本轮不解决，也不重复危险fixture。
无深图压力输入，无持久 Windows 设置变化。

原目录生成器：重新枚举实际 `tools/gen-*.py`，逐个受限串行、完整连续两轮。
基线包含全部新增设计材料，所有非忽略路径与原始 SHA256 比较：
Baseline=Round1=Round2=Final，added/removed/changed皆0才通过。
不预填2343为最终分母，不将历史CRLF与GitLF冒充原始字节相等。

最终回执生成时应同时校验 clean worktree、代码门禁后 crates/tools/spec 零差异、
所有起始2343个原文件原字节、当前源与生成器最终树相等。最终HEAD自指只写 ignored
runtime receipt，避免在 tracked 文件里制造“改一次最终SHA又变”的假封存。
