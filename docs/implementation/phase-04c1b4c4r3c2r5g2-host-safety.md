# Host caps and failed-run retention

全程沿用tools/run-production-input-graph-safe.py：topjob1、CPU硬5%、process768MiB、
job1536MiB、BelowNormal、RUST_TEST_THREADS1、CARGO_BUILD_JOBS环境1。不提高限额。
所有compiler/Oracle/generator均在受控Job中，日志有readback、peak committed、最低可用RAM。

开发编译r5g2-test-dev-r2在默认debuginfo2触及进程上限，LLVM OOM，退出101，不计通过。
原日志、外壳回执保留。低调试符号重试profile.dev.debug=0/profile.test.debug=0成功；
只去除调试符号，不改优化、debug assertions、features、测试选择或规范内容。
后续完整工程命令保持原参数，环境显式使用同一低调试符号配置，并在运行账本单列。
这不是降低测试门禁、提高内存或把受限中断算通过。

初次开发测试因Cargo显式test列表未添加target而退出101；添加仅新test入口后重试，
旧test targets没有删除/跳过。开发性metadata-cases-dev因重复解析模型手动终止，
校验唯一所属PID并确认worker结束后只移除其单个残留active锁；该中断不计通过。
随后只读模型缓存重试；r2发现新适配器nominal参数签名问题失败，r3修复后成功，原日志保留。

旧摘要编码器深层指数分配风险仍未关闭，不运行危险深度输入。完整历史使用各自源码/
Registry/Compiler/Expected；任何源码归档用流式tar文件，不将整份archive装入内存。
最终没有完整新鲜回执的门禁都应保留未完成，不能由Cargo成功推出外部Oracle或历史成功。
