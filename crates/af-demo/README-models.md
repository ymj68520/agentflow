# Studio 阶段 3：真实 LLM 与本地工具

这是独立演示系统，不是正式 Semantic Runtime、RootExport、Publication 或 Durable Execution。离线 A/B/C 保持可用；D/E 通过服务器配置的 OpenAI-compatible **Chat Completions** 服务调用真实模型。

## 配置与启动（现有 fwq 工作区）

```bash
ssh fwq
cd ~/projects/agent-r5g2-linux-validation/workspace
# 已配置时无需再执行。交互输入密钥，不要把密钥写在命令参数里。
python3 -B tools/configure-demo-model.py
# 仅当 target/studio-model-dev 尚不存在时初始化本次开发会话：
# python3 -B tools/init-model-demo-session.py
python3 -B tools/run-model-demo-gate.py serve
```

服务绑定服务器 `127.0.0.1:8787`。Windows/本地另开终端：

```bash
ssh -N -o ExitOnForwardFailure=yes -L 127.0.0.1:8787:127.0.0.1:8787 fwq
```

打开 http://127.0.0.1:8787 。不要重复启动已有会话；停止命令使用启动时打印的 UUID：

```bash
python3 -B tools/linux-model-demo-dev.py cancel --gate serve --id <UUID>
```

配置脚本默认 Base URL 为 `https://chat.cordapp.org/v1`，默认模型 `deepseek-flash`；可交互改为其它兼容服务。根 URL 自动补 `/v1/chat/completions`，`/v1` 自动补 `/chat/completions`，也接受完整 endpoint。服务器要求 HTTPS；只有明确配置的 loopback 协议 Fixture 可以使用 HTTP。

凭据保存在项目内忽略的 `target/private/model.json`，目录 0700、文件 0600，不进入 Git。服务通过 `AF_DEMO_MODEL_CONFIG` **路径**加载，不把密钥写进执行器环境回执。示例配置结构如下，实际文件由交互脚本生成：

```json
{"base_url":"https://provider.example/v1","default_model":"model-id","api_key":"<交互输入，勿提交>"}
```

轮换时用 `python3 -B tools/configure-demo-model.py --replace`，然后正常停止/重启服务。未提供配置文件时离线仍可运行，Real LLM 明确报未配置，不会偷偷改成离线回答。界面只显示配置状态，不会在打开网页时自动调用/计费。Cargo 的 `--offline` 只禁止联网取依赖，不阻止用户点击运行 Real LLM 后的指定模型请求。

## 如何操作

五种节点不变。在 Agent 节点的“模型模式”中选择 `Offline Deterministic` 或 `Real LLM`。Real 模式可编辑模型名称（空白用服务器默认）、System Prompt、最大模型请求轮数、工具预算、超时与工具白名单。**服务地址和密钥只能在服务器配置，不接受来自工作流的网络地址/凭据。**

拖放、Text 端口连接、节点参数、删除、草稿保存恢复、JSON 文本/文件导入导出和取消均继续保留。`workflow` 与 `layout` 仍分离，旧 v1 工作流省略 `model` 就是离线模式。工作流 JSON / localStorage 只保存公开模型参数，不保存凭据。

| 案例 | 模式 | 结果 |
| --- | --- | --- |
| A | 离线 Tool | `Hello AgentFlow` → `HELLO AGENTFLOW` |
| B | 离线 Condition | 输入包含 `Agent` 时选择 Task A，否则 Task B |
| C | Offline Deterministic | `agent` 经 5 次本地字符变换成为 `AGENT`；不是 LLM |
| D | Real LLM，无工具 | 模型直接回答问题 |
| E | Real LLM + Tools | 模型自主选择工具及参数，根据真实函数结果回答 |

五个可导入文件在 `examples/`，也可在界面直接加载。模型具有随机性，不要求每次回答逐字相同，也不硬编码调用顺序。Trace 区分 Model Request / Model Response / Tool Requested / Tool Started / Tool Completed / Model Final Answer，显示真实经过时间及调用耗时（毫秒取整）。值超过 512 字符时明确标记为预览截断，Output 最多 16 KiB。

## 安全的闭合工具集

| 逻辑名称 | 参数（禁止额外字段） | 函数实际结果 |
| --- | --- | --- |
| `text.word_count` | `{text: string}` | `{word_count, character_count, utf8_bytes}`，单词按 Unicode 空白分隔 |
| `text.transform` | `{text: string, operation: "uppercase" / "lowercase" / "trim"}` | `{operation, text}` |
| `text.find` | `{text: string, query: nonempty string}` | `{found, match_count, positions:[{byte_index,character_index}], positions_truncated}`，大小写敏感、不重叠、零基位置，最多列出 128 个位置 |

工具的描述和 JSON Schema 在 `src/tools.rs::definitions`；本地 Task 和模型调用使用同一组函数。OpenAI 标准函数名称不接受点号，所以协议中的名称是 `text_word_count` / `text_transform` / `text_find`；仅做固定一一映射，不把任意模型文本当函数名执行。[标准工具回传格式](https://developers.openai.com/api/docs/guides/function-calling)使用 assistant 的 `tool_calls` 和带对应 `tool_call_id` 的 tool 消息。

整批调用先验证名称、唯一 ID、参数与预算，成功才串行执行；任何未知/未允许工具、非法参数、重复 ID 或超预算都会明确失败。没有任意 shell、磁盘访问或其它网络工具。模型服务地址仅由可信服务端配置决定。

## 资源、取消与边界

沿用合格项目 cgroup 执行器，所选上限为 AS 8 GiB / 任务树 6 GiB / CPU 在线容量 10% / Swap 0 / PIDs 128，jobs 1、test threads 2、重型任务并发 1、每命令 90 分钟、主机内存保留 8 GiB、可用磁盘保留 20 GiB、本任务累计增长阈值 12 GiB。B2 是采样保护，不是硬磁盘配额。没有重复 OOM/PIDs/Guardian 死亡资格测试。

模型每请求最多 2048 输出 tokens；每 Agent 1–32 个请求、0–32 次工具调用、1–180 秒，总工作流 300 秒。连接超时最多 10 秒、单模型请求最多 60 秒；完整会话/响应均有 128 KiB 上限。多个 Agent 的声明预算合计最多 128。没有自动 API 重试、模型替换或语义降级。

HTTPS 复用安装好的 `/usr/bin/curl`，不引入新 Cargo 依赖。它是固定的内部传输实现，**不是可由工作流执行的 shell 工具**。Bearer 凭据只通过匿名 stdin 管道传入，不在 argv、工作流、浏览器或 Trace 中；不加载 curlrc、不使用环境代理、不跟随重定向、保持 TLS 校验。响应内意外回显的已知凭据先脱敏再进入 Trace/工具。错误不输出 HTTP 原始错误正文或 Authorization 头。

取消会中断并回收本地 HTTP 传输，阻止后续工具调用，不保证第三方服务同步停止推理或计费。系统只保留最近一次运行，不支持生产级认证、多租户、崩溃恢复、分布式执行、MCP/A2A 或 Windows 独立安装包。不要将服务公开到互联网。

## 验证

```bash
python3 -B tools/run-model-demo-gate.py format
python3 -B tools/run-model-demo-gate.py fmt
python3 -B tools/run-model-demo-gate.py clippy
python3 -B tools/run-model-demo-gate.py demo-test
python3 -B tools/run-model-demo-gate.py test
python3 -B tools/run-model-demo-gate.py spec
```

测试使用合成凭据和本地协议 Fixture，不会自动用你的真实密钥发外部请求。Fixture 通过不是“真实 LLM 成功”；真实 D/E 的实际浏览器结果及门禁结果分别记录。原始执行日志在 `target/studio-model-dev`，以前阶段证据保留。
