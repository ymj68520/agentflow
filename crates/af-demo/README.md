# AgentFlow Studio — 可视化工作流演示

本地、离线、非生产原型。没有正式 Runtime、Canonical Semantic IR 转换、RootExport、Publication、Durable Execution 或 Effect Authority。

## 在现有 Linux 工作区启动

```bash
ssh fwq
cd ~/projects/agent-r5g2-linux-validation/workspace
# 本次开发已初始化会话；仅当 target/visual-workflow-demo 不存在时执行：
# python3 -B tools/init-demo-session.py
python3 -B tools/run-demo-gate.py serve
```

另开本地终端建立转发（服务只绑定服务器回环地址）：

```bash
ssh -N -o ExitOnForwardFailure=yes -L 127.0.0.1:8787:127.0.0.1:8787 fwq
```

浏览器打开 **http://127.0.0.1:8787**。启动命令打印本次 UUID；正常停止可在另一 SSH 终端执行：

```bash
python3 -B tools/linux-demo-dev.py cancel --gate serve --id <启动时打印的UUID>
```

`UserCancelled` 是受控服务停止记录，不是 Cargo 产品错误；强制清理时 Cargo 退出码可能未观测到，不伪造为成功。服务最多运行 90 分钟。端口占用则准确报告，不接管其它服务。项目执行器会预检内存、磁盘、工具链、任务锁并回读 cgroup / RLIMIT；不满足则拒绝启动。它不是外部不可信代码沙箱。

## 操作

- 左侧拖放组件（或点击添加），拖动节点标题移动。
- 从输出圆点拖到输入圆点，或依次点击两个端口。所有数据均为 Text；Condition 有 true/false 两个输出。端口占用、自连接、外部环路、非法汇合会被拒绝。
- 点击节点编辑名称、输入、工具参数、条件或循环上限。点击连线后可删除；删除节点也删除相关连线。
- 保存/恢复草稿使用当前浏览器 localStorage；导入/导出 JSON 面板支持复制粘贴与文件选择/下载。不完整草稿可以保存；执行前必须通过服务端完整验证。
- 工作流 JSON 的 `workflow` 只包含语义节点与边，`layout` 独立保存位置，`version` 为 1。
- 加载 A/B/C 后点击运行；运行期间配置冻结。底部显示步骤、选择分支、工具反馈与最终输出；取消按钮终止当前运行。

## 三个案例

| 案例 | 输入 / 设置 | 可验证输出 |
| --- | --- | --- |
| A 顺序工具 | `Hello AgentFlow` → uppercase | `HELLO AGENTFLOW` |
| B 条件 | 包含 `Agent`（区分大小写） | `Matched: Hello AgentFlow`；改为 `offline` 走另一路，得到 `Not matched: offline` |
| C 离线 Agent Loop | `agent`，最大 8 轮 | 每次调用“单字符大写”工具；5 次调用 / 5 次反馈 / 6 次决策后得到 `AGENT` |

C 的策略是确定性算法，**不是 LLM**。将最大轮数改为 1 会在该节点报出上限错误。没有密钥、任意 shell、文件读写工具或网络调用工具。

## 实现与边界

`Visual Authoring → ValidatedWorkflow → Demo Executor → Event trace`。新增 `af-demo` workspace crate，使用既有 `af-core::primitive::parse_uint32_text` 严格校验循环次数，以及已有锁定 serde/serde_json。没有新增第三方包；正式静态契约 crate 保持不变，不能把演示图当作它们的完整合法输入或证明。

后端：std::net loopback HTTP + 单活动 worker + 可取消的有界文本解释器。前端：本地 HTML/CSS/JS + SVG，无 npm、CDN 或前端构建。静态资源编译嵌入二进制，修改后须重启服务并刷新页面。

限制：最多 32 节点、64 条边、单 Input/Output；只支持单路顺序/条件执行，只有 Output 允许分支汇合，没有并行 join。每个循环 1–32 轮，所有循环声明预算合计最多 128 次工具调用。单文本最多 16 KiB UTF-8；日志值显示明确标记的前 512 字符预览，最终输出不作预览截断。HTTP 文档最多 128 KiB。只保留最近一次运行；重启不恢复执行，没有用户认证、多租户、正式资源授权或崩溃恢复能力。仅用于可信本地演示，不应暴露互联网。不是 Windows 发布兼容性验收。

## 验证入口

所有 Rust 命令经项目受控执行器串行运行：

```bash
python3 -B tools/run-demo-gate.py format
python3 -B tools/run-demo-gate.py fmt
python3 -B tools/run-demo-gate.py demo-test
python3 -B tools/run-demo-gate.py clippy
python3 -B tools/run-demo-gate.py test
python3 -B tools/run-demo-gate.py spec
```

选用既有授权内的较低上限：CPU 在线容量 10%，进程 AS 8 GiB、任务树 6 GiB、Swap 0、PIDs 128、jobs 1、test threads 2；主机内存保留 8 GiB、磁盘剩余 20 GiB、本任务累计新增 12 GiB。B2 为采样保护，不是硬配额。原始日志与资源回执在 `target/visual-workflow-demo`；旧基线证据不覆盖。
