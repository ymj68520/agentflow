# R5G2R3 — 安全与资源政策

本轮只做小规模、流式、只读输入审计，写入仅限新R3材料和ignored target派生回执；重型任务、远程环境、迁移产物均 **Not Executed**。流式采集、设计向量、三项回读和最终源树复核均由原受控执行器包裹，CPU5%、进程768MiB、Job1536MiB、reserve8GiB、BelowNormal、一个顶层Job、Cargo jobs1/测试调度1不变。不得改执行器，亦不得通过子进程breakaway绕过限制。最后closeout仅在Job已退出、锁不存在后读取现成小回执与工作树/PID元数据并写最终账本，不启动受控工作负载或再次扫描源树；该元数据步骤不冒充受控Job内运行。

这些是用户明确冻结的操作条件，不是可自行取消的优化选项。方案B的 **P_build / J_build** 只是无数值的待批准变量；新保护政策须逐项授权且实现/验证执行前失败关闭。编译保护额度与正式契约中的Runtime资源要求完全不同，不能以编译豁免改变运行时语义。没有用户决定就仍使用原限额且不可认定可行。

未来系统预算需独立的物理可用、commit余量、系统负载和时间窗证明；进程/Job预算需明确硬上限、测量采样、CPU/时限/磁盘上限、OOM及中断终止条件。资格试验也必须先批准保护额度，不允许无界探峰。发生中断只终止本任务受控Job、保留全部原始证据，不重试或自动升额。不终止无关进程，不改pagefile、内存压缩、优先级政策或OS内存管理。

原执行器已使用KillOnJobClose、默认子进程入Job且不配置breakaway，安装限制后才释放worker；未接入Job completion port。因此旧峰值只能支持配额一致性判断，不能伪造为收到内存上限通知。[Job Objects官方说明](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)支持该区别。原监督器采样不是逐分配追踪，也不保证物理reserve绝不瞬时穿线；两次实际79仍如实保留。

LLVM日志实际OOM101、监督器cancelled=null、后续派生“资源安全中断”三者各自保留，不能相互覆盖。Windows Job进程/Job限额对应提交虚拟内存，而[GlobalMemoryStatusEx](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/ns-sysinfoapi-memorystatusex)的可用物理数包含可供分配的物理页；不将二者相减归因外部程序。

环境执行者最小权限、冻结源只读、构建输出限目录、凭据与源码隔离、依赖预审/网络默认禁用。证据输入视为不可信数据，拒绝路径遍历、链接、重复JSON键、未知字段、超大内容及自授信。传输认证、签名实现/密钥管理、具体容量、壁钟限制和磁盘额度均留待后继审批，没有暗示现有执行器已实现这些新增能力。

RootExport/Portable正向权威不签发，Native权限不扩展，CompleteView/ObligationIdentity不签发；运行日志与环境签名只能证明执行来源，不能升级权限。任何语义失败需真实规则/输入/Expected/Actual/字段差异，不能隐藏成资源事件；资源不足也不能伪称规范矛盾。
