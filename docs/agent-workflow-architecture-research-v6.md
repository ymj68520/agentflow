# Agent 工作流可组合、可编排工具：抽象设计、架构与 v1 规范化基线（v1 规范冻结版）

> 更新日期：2026-10-07  
> 文档定位：架构设计与调研结论，不包含 MVP 路线、人力排期或分阶段实施计划。  
> 更新说明：本版基于原始 `deep-research-report.md` 与上一版更新稿，继续合并后续逐项讨论中已经确认的设计结论。文档分为“架构调研/设计理念”和“v1 规范化机器基线”两层：前者解释为什么这样设计，后者冻结当前已讨论清楚的公开语义、状态机、协议与不变量。早期表述若与后文 v1 基线冲突，以后文为准。**当前状态：v1 Architecture Discovery 已关闭，Core Architecture、Semantic Model、Durable Runtime、Security/Recovery、Distributed Authority 与 Public Projection Boundary 均已冻结。Multi-region Ownership、Audit/Security Evidence、13 Opcode Normative Sheets、Conformance/Reference Oracle、Specification Packaging 与 SDK/Public Projection 已完成规范级收敛。剩余工作不再属于架构发现，而是正式 AFIDL/registry/table/golden corpus 的制品化、Reference Interpreter 实现与跨语言/故障 Conformance 验证。**

---

## 执行摘要

本报告的核心判断保持不变，但底层抽象已经进一步收敛：**不要把 Agent 作为编排系统的最小原语；Agent 应当是建立在统一工作流语义之上的复合组件。** 原报告用 `Action / Decision / State / Control / Boundary` 五类 primitive 来说明这一方向，这一表达适合产品层和早期讨论，但它混合了计算、资源、控制和边界几个不同维度。经过进一步推导，推荐将真正的 Semantic Kernel 收敛为：

```text
Semantic Kernel
=
Value
+ Task
+ Await
+ Control
+ Resource
```

其中：

- `Value` 是不可变工作流数据；
- `Task` 是有限、主动、可调度的工作单元；
- `Await` 是不占 Worker 的 durable suspension；
- `Control` 是 Sequence / Select / Parallel / Race / Iterate / Retry / Timeout / Scope 等结构化组合语义；
- `Resource` 是显式、可授权、可绑定的有状态能力。

`Effect`、`Identity`、`Policy`、`Budget`、`Information Label` 等是横切语义，而不是新的“节点类型”。Agent、Tool、Human Approval、Memory、Trigger、Checkpoint 都不属于 Kernel primitive：它们分别是 Composite、Task specialization、Resource specialization、Deployment Binding 或 Runtime object。

统一的组合对象不再只看输入、输出和 Effect，而按多个互相正交的 Contract 维度描述：

```text
Computation<Input I, Output O, Failures Φ, Effects ε, Resources R>

+ InformationFlow F
+ Usage U
+ Recovery K
+ ExecutionRequirements X
```

其中 `I/O` 是数据 ABI，`Φ` 是静态 Failure Row，`ε` 是 Effect Signature，`R` 是 Resource Requirement Row；`F/U/K/X` 分别描述信息流、用量/成本、恢复契约和物理执行要求。Agent 仍然只是这些统一 Computation 的高阶 Composite。`Task`、`Await`、`Sequence(A,B)`、`Parallel(A,B)`、`Subflow`、`Agent Composite` 对外都满足 Computation 闭包。这样系统既能运行完全没有 LLM 的确定性工作流，也能运行动态 Agent Loop，而不会形成两套 Runtime。

推荐的总体架构不再是“大而全 Agent Framework”，而是：

```text
Agent Workflow Platform
=
Semantic Computation IR
+ Multi-pass Workflow Compiler
+ Immutable Executable Plan
+ Journal-driven Durable Interpreter
+ Capability / Delegation / Information-flow Security
+ Contract-based Component & Resource System
+ TaskInvocation / Attempt / Worker Protocol
+ Multi-tenant Capability Scheduler
+ Content-addressed Registry & Supply-chain Trust
+ Conformance / Evaluation Framework
+ Visual / DSL / API Projections
```

从参考项目看，仍然没有一个开源项目同时在 Agent Harness、持久化工作流、可视化编辑、跨语言协议、权限、供应链和多租户调度上都给出完整答案。应当按层借鉴，而不是照搬单一项目：OpenCode 适合参考 Harness、Client/Server、Tool/MCP 和权限；Agno 适合参考 Agent/Team/Workflow/Runtime 分层；LangGraph 适合参考 graph/state/checkpoint/HITL；Temporal 适合参考 durable execution；MCP 适合作为 Tool/Capability 边界；A2A 适合作为 Remote Agent 边界；Kubernetes/Ray 适合参考异构资源调度；SPIFFE、OPA、OAuth Token Exchange 适合参考身份与授权；OCI、Sigstore、SLSA、in-toto 适合参考软件供应链。

这套架构的关键不是“功能更多”，而是形成少量稳定不变量：

1. Agent 不是 Kernel primitive。
2. Task 与 Await 分离：主动执行与被动等待是不同语义。
3. Dataflow 与 Control Ownership 分离。
4. State 不再是万能抽象：工作流数据是 immutable Value，共享可变世界是 Resource，执行状态属于 Runtime。
5. Runtime 不依赖重跑任意用户代码恢复；它解释 immutable Plan，并依据 Journal 重建状态。
6. Model Intent 不是 Authorization；Effect 必须经过 Runtime Policy/Grant/Enforcement。
7. External Content 不能自行升级为 Instruction Authority。
8. Worker 只拥有 Attempt，不拥有 Workflow Run。
9. Published Version、Package 和 Plan 都必须 immutable/content-addressed；Live Safety 可以收窄旧 Run 的权限，但不能静默扩大语义能力。
10. Studio、SDK、CLI、AI Builder 都是同一 Semantic IR 的 Projection，没有任何入口可以绕过 Compiler 与 Runtime 语义。

---

## 一、设计原则与整体分层

### 1.1 Harness、Loop 与 Agent 的关系

原报告将 Harness 理解为模型运行环境、Loop 理解为控制结构、Agent 理解为二者组合，这一判断保留：

```text
Harness
=
Context Assembly
+ Model Adapter
+ Tool/Component Access
+ Resource Access
+ Permission / Sandbox
+ Event / Trace
+ Cancellation / Budget
```

```text
Agent
=
Harness
+ Agent Policy
+ Model Task
+ Tool Tasks
+ Resources
+ bounded Control Loop
```

但更新后的设计不再要求 Runtime Kernel 识别 `AgentRuntime` 这一特殊执行类别。更严格地说：

```text
Agent Harness
=
Composite Library
+ Context Resource
+ Model Task Adapter
+ Tool Task Adapters
+ Agent-specific Policy
+ Agent-specific Debug Projection
```

可以为了产品体验和性能提供 Agent Harness Service，但它不能拥有第二套 Durability、Retry、Effect、Security 或 State 语义。

### 1.2 四个宏观平面

整体仍可划分为四个宏观平面：

```text
Design / Control Plane
  Authoring / Compiler / Registry / Version / Deployment

Runtime Plane
  Durable Interpreter / Scheduler / Worker / Timer / Signal

Persistence & Data Plane
  Journal / Checkpoint / Value / Artifact / Memory / Provenance

External Capability Plane
  MCP / A2A / HTTP / DB / Browser / Model Providers / Enterprise Systems
```

Identity/Policy/Trust 与 Observability 横切以上各层。

原报告中的 “State Plane” 建议改名为 `Persistence & Data Plane`，因为 Journal、Checkpoint、Artifact、Memory、External Resource 在语义上完全不同，不应继续以“State Store”概念统一。

### 1.3 产品 Ontology 与 Kernel Ontology 分离

Studio Palette 可以继续使用用户友好的：

```text
Agent
Search
Email
Approval
If
Loop
Parallel
Database
Memory
```

这些不需要与 Kernel 一一对应。产品层的高阶 Block 只要能够无歧义地投影成 Canonical Semantic IR 即可。Scratch/Blockly 值得借鉴的不是彩色积木本身，而是：

- 结构合法性；
- 类型合法性；
- Scope 合法性；
- 通过编辑器帮助用户构造合法程序，而不是画任意图后让 Runtime 猜含义。

---

## 二、Semantic Kernel 与统一 Computation 模型

### 2.1 五个一等语义对象

#### Value

`Value<T>` 是不可变工作流数据。工作流内部默认使用 SSA/functional 风格，而不是共享可变 `state.foo`。

```text
%query      : String
%findings   : List<Finding>
%report     : ArtifactRef<Report>
```

Studio 可以允许用户把变量重新赋值：

```text
report = generate(...)
report = revise(report)
```

Canonical IR 则转为：

```text
%report#1 = generate(...)
%report#2 = revise(%report#1)
```

并行分支之间不共享可变 Workflow State；需要合并时使用显式 Join/Reducer。

#### Task

Task 是 **Workflow Kernel 中最小的有限、主动、可调度工作单元**。它具有显式输入、授权后的 Resource/Capability、一个逻辑 Invocation、零到多个物理 Attempt，以及一个终态 Outcome。

Task 不拥有 Workflow Control，不直接跳转图，不决定自身 Retry，不通过持有线程实现长期等待。

```text
TaskDefinition
    ↓ bind
TaskBinding
    ↓ run
TaskInvocation
    ↓ physical tries
TaskAttempt*
    ↓
TaskOutcome
```

Task terminal outcomes 至少包括：

```text
SUCCEEDED
FAILED
CANCELLED
TIMED_OUT
```

Streaming/Progress 不是 Outcome。

#### Await

Await 是与 Task 平级的 Computation：

```text
Task  = active finite execution
Await = passive durable suspension
```

`Await` 创建 durable `WaitCondition`，不占 Worker，可以等待数天、数月。Human Approval、Webhook Reply、Remote Agent Completion 等长期交互应优先建模为：

```text
Invoke(StartRemoteOperation)
→ Await(RemoteCompletion)
→ Invoke(FetchResult)
```

而不是一个长期占用 Worker 的“异步 Task”。

#### Control

Control 是 Semantic IR 中一等的结构化组合算子，它接收 child Computations，并定义启动、等待、终止、取消与 Outcome 聚合规则。Control 本身不承担业务副作用。

v1 推荐冻结为一组小而封闭的结构化 opcode：

```text
INVOKE@1
AWAIT@1
SEQUENCE@1
SELECT@1
PARALLEL@1
RACE@1
ITERATE@1
RETRY@1
TIMEOUT@1
HANDLE@1
SCOPE@1
COMPENSATION_SCOPE@1
CALL_SUBFLOW@1
```

`CallComposite` 只存在于 authoring/compiler 层并在发布前 lower 掉；Trigger、HumanTask、LLM、HTTP、MCP、A2A 都不增加 Kernel opcode。

其中 `Retry / Timeout / CallSubflow` 理论上可以继续 lower，但在 Semantic IR 中保留有利于 Effect 分析、Diff、Migration、Debug 与用户理解。

#### Resource

Resource 表示 Workflow 外部或跨 Computation 生命周期存在的有状态能力，例如：

```text
Database
ArtifactStore
MemoryStore
Workspace
Filesystem
ObjectStore
SecretStore
Git Repository
Browser Session
Remote Service
```

Task 不获得 raw DB connection、raw filesystem root、raw secret 或万能 Runtime Client，而只获得 scoped、attenuated、opaque `ResourceHandle`。

### 2.2 统一 Computation

统一组合对象按多维 Contract 描述：

```text
Computation<I, O, Φ, ε, R>

+ F  # Information Flow
+ U  # Usage / Cost
+ K  # Recovery
+ X  # Execution Requirements
```

其中：

- `I`：输入类型；
- `O`：输出类型；
- `Φ`：静态 Failure Row；
- `ε`：Effect Signature；
- `R`：Resource Requirement Row；
- `F/U/K/X`：信息流、用量/成本、恢复与执行要求。

`Task`、`Await`、`Control`、`Subflow` 和 Composite 对外都表现为 Computation。

```text
Control:
Computationⁿ
→ Computation
```

这使 Control Algebra 闭合，也使 Agent 可以完全退出 Kernel。

### 2.3 Expression Language

不应把字段读取、比较、简单 Struct 构造都变成 Task。Semantic IR 可包含受限的 pure Expression Language，用于：

```text
binding
projection
boolean predicate
simple arithmetic
struct construction
enum matching
```

Expression 必须：

```text
pure
bounded
deterministic
versioned
side-effect free
```

复杂计算、外部资源、显著执行成本或需要持久 Outcome 的操作仍然是 Task。

---

## 三、Task 执行语义与 Effect System

### 3.1 Invocation 与 Attempt 必须分开

Retry 不应重新创建逻辑 Invocation。

```text
TaskInvocation I
├── Attempt #1
├── Attempt #2
└── Attempt #3
```

不同 Attempt 共享 logical invocation/idempotency context，但拥有独立 `attempt_id`、Lease 与执行证据。

可以将 Invocation identity 稳定派生自：

```text
Run
/ Scope Instance
/ Semantic Op
/ Logical Occurrence
```

Loop 下一 iteration 是新的 Invocation；同一 iteration 的 retry 是新的 Attempt。

### 3.2 Retry、Timeout、Cancellation 与副作用

组件 Contract 需要显式描述：

```text
Replay Semantics:
  RECOMPUTE
  RECORDED
  RECONCILE
  FORBIDDEN

Idempotency:
  NATURAL
  KEYED
  NONE
  RECONCILED
```

Timeout 至少应区分：

```text
ScheduleToStart
StartToClose
Heartbeat
```

Cancellation 是 cooperative，不等于“请求一发出任务就不存在”。终态可能出现：

```text
CANCELLED
COMPLETED_AFTER_CANCEL_REQUEST
CANCEL_UNKNOWN
```

对 effectful Task，失败还必须报告外部副作用状态：

```text
NOT_APPLIED
APPLIED
UNKNOWN
PARTIALLY_APPLIED
```

`UNKNOWN` 不允许 blind retry；必须 reconcile 或进入人工/工作流错误处理。

### 3.3 Effect 的定义

Effect 是 **Task 与纯计算边界之外状态、资源或主体之间的可观察交互描述**。Read 也算 Effect。

Effect 不是权限字符串，而是结构化语义：

```text
Effect
=
Operation
+ Resource Selector
+ Qualifiers
```

例如：

```text
github.issue.create<repo=foo/bar>
network.read<host=docs.example.com>
database.read<table=customer>
payment.charge<merchant=X>
```

同时区分：

```text
Semantic Effect:
github.issue.create

Mechanism Capability:
network.egress
credential.use
fs.write
process.exec
```

### 3.4 Effect 生命周期

推荐固定：

```text
Effect Declaration
    ↓
Effect Intent
    ↓
Policy Decision
    ↓
Effect Grant
    ↓
Execution
    ↓
Effect Receipt
```

核心不变量：

```text
Actual Effects
⊆ Granted Effects
⊆ Declared Effects
```

`PermissionDenied` 是 Policy 正常拒绝；`EffectViolation` 是实际行为超出了 Grant/Declaration，两者不能混。

EffectReceipt 与 TaskOutcome 分开：Output 服务 Dataflow，Receipt 服务 Retry/Reconcile/Compensation/Audit。

### 3.5 Effect Algebra

原子 Effect 可写为：

```text
e = Op<Resource, Qualifiers>
```

Task 类型可写为：

```text
Input ->{ε} Output
```

Effect Signature 至少包含：

```text
Capability Row
Resource Scope
Occurrence Bounds
Guards
Openness
```

推荐支持：

```text
∅        pure
⊑        containment
⊕        accumulation
⊔        exclusive branch join
n⊗ε      repetition scaling
ε | ρ    open row
⊤        unprovable boundary
```

默认 closed row。动态 MCP/Tool discovery 等场景可以产生 bounded open row，但 `unknown` 绝不能解释为 `none`。

不同 Control 的 Effect 传播：

```text
Sequence(A,B)  : εA ⊕ εB
Select(A,B)    : εA ⊔ εB
Parallel(A,B)  : εA ⊕ εB
Race(A,B)      : conservative accumulation
Iterate(n,A)   : n ⊗ εA
```

Race 的 loser 可能在取消前已经产生 Effect，因此不能把 loser Effect 当作未发生。

Retry 必须同时分析：

```text
Logical Effect Cardinality
Physical Attempt Envelope
```

幂等/有 idempotency key 的执行可以保持逻辑 Effect 上界，但物理 Attempt 数仍可能增加；非幂等重试则可能直接放大业务 Effect。

### 3.6 Compensation

Compensation 不是 Task 的隐藏方法。推荐建模为另一个 Task，由 `CompensationScope` 根据已 committed EffectReceipts 决定调用顺序。

```text
Forward Effect
+
Conditional Compensation Effect
```

补偿不会“从历史中减掉”原 Effect；原 Effect 与 Counter-effect 都是事实。

---

## 四、Structured Control、Scope 与事件边界

### 4.1 Structured Control Tree，而不是 arbitrary graph

Canonical Semantic IR 推荐定义为：

```text
Structured Control Tree
+
Scoped Typed Dataflow Graph
+
Contract Tables
```

Control Tree 定义 lifecycle/ownership；Dataflow Graph 只表示 Value dependency。不要让一条 Edge 同时表达：

```text
数据依赖
启动条件
生命周期 ownership
```

### 4.2 Scope 是真正的执行边界

普通 `Scope` 统一承载：

```text
Ownership Boundary
Cancellation Boundary
Deadline Boundary
Budget Boundary
Authority Boundary
Resource Lifetime Boundary
```

补偿不再作为普通 `Scope` 的隐式能力，而由显式 `COMPENSATION_SCOPE@1` 承担。这样普通 Scope 只负责结构化并发和能力收窄，恢复/补偿拥有独立状态机与严格的 Effect closure。

Runtime 形成 `ScopeInstance Tree`：

```text
WorkflowScope
├── AgentScope
│   ├── ModelInvocation
│   └── ToolScope
└── PublishScope
```

核心结构化并发不变量：

```text
Every running child belongs to exactly one Scope.
Scope normally cannot terminate while owned children are active.
```

Detached work 必须显式声明，不能由普通 Task 悄悄“后台执行”。

### 4.3 Sequence / Select / Parallel / Race / Iterate

`Sequence` 只有前一 child 达到规定 Outcome 后才启动下一 child。

`Select` 是显式互斥分支，推荐要求 exhaustive/default；分支输出在外部通过 typed merge 汇合，而不是把 branch-local Value 直接泄露出去。

`Parallel` 需要显式：

```text
Completion Policy
Failure Policy: FAIL_FAST / COLLECT / ...
Aggregation Policy
```

FAIL_FAST 的正确含义是：请求取消 sibling，并等待 owned children 达到 terminal，再完成 parent。

`Race` 与 `Parallel(join=ANY)` 不应混成一种语义。Race 明确选择 winner，并将 winner 记录为 Semantic Decision；loser cancellation 不代表 loser 没有 Effect。

`Iterate` 统一 ForEach / Map / While / Repeat；loop-carried state 仍使用 immutable Value evolution：

```text
%acc#0
→ iteration 0
→ %acc#1
→ iteration 1
→ %acc#2
```

### 4.4 Event / Signal / Trigger / Timer

不要使用一个万能 `Event` 概念。推荐严格区分：

```text
ExternalEvent
Signal
Trigger
Await
Timer
JournalEvent
```

- `Trigger`：External Event → StartWorkflow；属于 Deployment/Entry Binding，不属于 Workflow Body。
- `Signal`：发送给已有 Run/Scope/Interaction 的 typed durable message。
- `Await`：Workflow 内部等待语义。
- `Timer`：Runtime 产生的 durable time event。
- `JournalEvent`：内部执行事实。

Signal 必须包含 stable target/correlation，不能只依赖可复用 business key。推荐每次外部交互生成独立 `CorrelationHandle / interaction_id`。

Webhook ingress 推荐：

```text
HTTP
→ Authenticate
→ Validate
→ Normalize
→ Durable SignalAccepted/Mailbox
→ ACK
```

而不是直接“唤醒某个 Worker”。

Early-signal race 通过 durable Mailbox/Inbox 解决：Signal 可以先到，Await 之后消费。一个被接受的 single-consumer Signal 最多消费一次。

Timer 与 Signal 同时到达时，由 Journal 的 monotonic sequence 决定 canonical winner，而不是 wall clock。

---

## 五、Value、Resource、Artifact、Memory 与执行状态

### 5.1 三类完全不同的“状态”

推荐正式区分：

```text
Kernel Execution State
Workflow Value Evolution
Resource Mutation
```

- Kernel Execution State：Runtime 自己管理。
- Workflow Value：immutable data。
- Resource Mutation：外部/共享可变资源，属于 Effect。

不要提供万能：

```text
StateStore.get/set
```

### 5.2 ResourceType / Binding / Handle

资源至少分三层：

```text
ResourceType / Definition
ResourceBinding
ResourceHandle
```

Workflow 只声明 logical slot：

```text
project_memory: MemoryStore@1
```

Deployment 绑定到逻辑资源身份；Runtime 再向 Task 发放 attenuated `ResourceHandle`。

`ResourceHandle` 回答“你能够接触哪个资源”；`EffectGrant` 回答“这一次允许做什么”。实际 Effect 必须同时落在两者边界内。

### 5.3 Artifact

Artifact 是 special immutable Resource，具有：

```text
ArtifactRef
Content Digest
Media Type / Schema
Provenance
Classification
```

Journal 存 ref/digest，不把大对象直接内嵌。

### 5.4 Memory

Memory 是 Resource，不是 Kernel primitive。`search/get/append/update/delete` 都是 Effect。

Memory Entry 应保留：

```text
provenance
security labels
verification state
sensitivity
```

“进入 Memory”绝不能自动提升 Trust。

### 5.5 Secret / Credential

Secret 不应进入普通 Workflow Value。Credential 不进入普通 Workflow Value 系统。Task 只声明 `CredentialRequirement` / `IdentityBinding`，真正短生命周期 `CredentialLease` 由 Credential Broker 在执行边界附近签发；Task 不获得 raw secret，也不把 Credential 当作可序列化 Value。

```text
credential.use
≠
credential.read/export
```

### 5.6 Workspace

Coding/Browser 等 Agent 需要 Workspace 时，应把它建模为显式 Resource：

```text
root
snapshot
branch/overlay
scope
lifetime
```

并行 Agent 可以 fork workspace，再通过显式 merge 组合，而不是共享一个隐式目录。

---

## 六、Durable Execution、Journal 与 Replay

### 6.1 中心选择：解释 Plan，而不是重跑任意用户代码

推荐的 Durable Execution 模型：

```text
Immutable Executable Plan
+
Durable Journal
→
Deterministic State Machine Interpreter
→
Execution State
```

形式上：

```text
(P, Sₙ, Eₙ)
→
(Sₙ₊₁, D*, C*)
```

其中：

- `P`：Plan；
- `S`：Execution State；
- `E`：已承认 Journal Event；
- `D`：新的 Semantic Decisions；
- `C`：Runtime Commands。

Runtime 不通过重跑 arbitrary Python/TS Workflow code 来恢复。这与 Temporal 的 durable execution 思路相近，但本系统更倾向显式 Semantic IR + Plan Interpreter，而不是以语言函数 replay 作为中心抽象。[R2]

### 6.2 Journal 是执行事实源，不是 Trace

Journal 按 Run 维护 append-only canonical fact stream，并具有 per-run contiguous monotonic `journal_seq`。只写影响恢复和未来执行的 durable canonical facts，例如：

```text
RunStarted
RunPause/Resume/CancelRequested
InvocationPreparationOutcome
TaskAttemptStarted / Outcome / Lost
SignalAccepted / TimerFired
SelectCaseChosen / RaceWinnerChosen
IterationGuardDecision / RetryDecision / AttemptRetryDecision
EffectReceiptRecorded / EffectReconciliationResolved
ObligationResolved
ChildStarted / ChildTerminal
MigrationApplied
ManualRecoveryResolution
```

`EffectGrant`、Heartbeat、Progress、Worker assignment、Command ACK、Queue Position 等不进入 Run Journal。

Token stream、普通 worker log、细粒度 Trace 一般不进入 Journal。

### 6.3 Semantic Decision 与 Command 分开

例如 Race：

```text
RaceWinnerChosen(B)   # workflow truth, Journal
RequestCancel(A)      # physical request, Command/Outbox
```

Command delivery 可以重复；Decision 一旦承认不能因为 Command 发送失败而改变。

### 6.4 Journal + Outbox 原子提交

Runtime Turn 推荐：

```text
load state/checkpoint
consume accepted events
advance interpreter to quiescence
produce semantic decisions + commands
atomically commit:
  Journal append
  Command Outbox append
  Materialized state/version update
```

Dispatcher at-least-once 发送 Command；Command 使用 stable identity 做去重/fencing。

### 6.5 外部 Effect 的“原子性缺口”

不存在通用 exactly-once external effect：

```text
external effect succeeds
↓ crash
receipt/outcome not journaled
```

因此恢复安全依赖：

```text
stable idempotency key
or reconcile external status
or mark EffectState UNKNOWN
```

若三者都不能满足，则不能自动 retry。

可以将 Task recovery safety 归类为：

```text
PURE
IDEMPOTENT
KEYED_IDEMPOTENT
RECONCILABLE
COMPENSATABLE
UNSAFE
```

### 6.6 Checkpoint

Checkpoint 是 Journal 的派生缓存，不是第二份 truth。

```text
Checkpoint {
  run_id
  epoch
  plan_digest
  semantics_profile_digest
  journal_seq
  execution_state
}
```

恢复：

```text
latest checkpoint
+ journal tail
→ current state
```

删除所有 Checkpoint 后，从完整 Journal 重建出的语义状态必须一致。

### 6.7 三种 Replay

推荐最终术语：

```text
Historical Replay
Recovery Replay
Fork / Counterfactual Run
```

- Historical Replay：只读历史导航，不重新执行 Effect。
- Recovery Replay：Runtime 内部 state reconstruction。
- Fork：从历史点创建一个新 Run，重新执行后续步骤并重新通过 live policy/auth/resource checks。

不要把它们都叫“Replay”。

---

## 七、Versioning、Compatibility 与 Migration

### 7.1 Version 与 Compatibility 分开

需要独立版本化的对象至少包括：

```text
Workflow Definition
Semantic IR
Executable Plan
Runtime Semantics
Component Contract
Component Implementation
Signal Type
Effect Type
Resource Type
Journal Event Schema
Checkpoint Schema
Expression Language
Prompt / Agent Template
Model Binding
Policy
Credential
```

Version 表示 identity；Compatibility 表示 substitution 是否安全。

### 7.2 Run 的 immutable semantic baseline

Run 创建时冻结 `RunManifest`：

```text
workflow/version
IR digest
plan digest
runtime semantics profile
expression version
component lock graph
prompt bundle digest
logical resource identities
model binding policy
```

但不要简单冻结 raw `policy_version`、`secret_version`：长期 Run 必须受到 live credential revocation、emergency deny、tenant suspension 等安全变化影响。

推荐三类依赖：

```text
Semantic Pin
Compatible Late Binding
Live Safety Overlay
```

### 7.3 Monotonic Safety

运行时有效能力：

```text
Effective Capability
=
Run Effect Envelope
∩ Delegation Authority
∩ ResourceHandle
∩ Current Safety Policy
∩ Remaining Budget
```

Live Safety 可以收窄旧 Run，但不能静默扩大原 Run 的 Effect Envelope。

### 7.4 Component Compatibility

新实现替换旧实现至少需要满足：

```text
new accepts old input domain
new output fits old expected output
Effects(new) ⊑ Effects(old)
```

Schema/Effect 检查仍不能证明全部 behavior compatibility，因此还需要 publisher declaration、contract tests、eval gates，形成 `Compatibility Attestation`。

Component Contract Version 与 Implementation Digest 分开；可以允许已验证的兼容实现替换，但严格合规模式可 exact-pin implementation。

### 7.5 Runtime Semantics Version

不要为每个旧 Run 永久保留整套旧 Runtime binary。更合理：现代 Runtime 同时包含多个 versioned semantic interpreter/profile，并由 RunManifest 选择。

Expression Language 同样需要 pin；禁止任意 Python predicate 进入 Workflow Control，否则 determinism/version surface 无法控制。

### 7.6 Migration

大多数短 Run 应 drain old version，不要迁移。迁移只适合长期 Run。

Run 只能在 durable safe/quiescent point 迁移，例如：

```text
after Task terminal
waiting Signal/Timer
Scope boundary
iteration boundary
```

不适合：

```text
effectful Task currently executing
unresolved UNKNOWN effect
```

MigrationPlan 至少包含：

```text
from/to plan digest
semantic-id mapping
value schema migrations
wait/signal mapping
resource rebinding
effect diff
pending work
compensation state
```

Migration 本身写入 Journal，并产生新的 `ExecutionEpoch`：

```text
Epoch 0 → Plan A
MigrationApplied
Epoch 1 → Plan B
```

历史 Journal 不重写。

---

## 八、Identity、Delegation、Credential 与 Policy

### 8.1 Identity ≠ Authority ≠ Credential

必须严格区分：

```text
Identity   = 你是谁
Authority  = 你被允许做什么
Credential = 你用什么技术凭据向目标系统证明这件事
```

不要把用户 OAuth Token 直接下放给整个 Agent。推荐链路：

```text
Authenticated Principal
→ DelegationGrant
→ Run Authority
→ Scope Authority
→ Task EffectIntent
→ PolicyDecision
→ EffectGrant
→ CredentialBroker
→ short-lived CredentialLease
```

OAuth 2.0 Token Exchange（RFC 8693）对 delegation 与 impersonation 的区分非常适合作为参考：Delegation 下 actor 保持自己身份，只是代表 subject 行使被委托权力。[R5]

### 8.2 Principal、Actor、Subject 与 Executor

`Principal` 是稳定身份；`Actor / Subject / Delegator / Approver` 是一次授权上下文中的角色。

例如：

```text
Subject:
Alice

Actor:
run_123/scope_research_agent

Executor:
spiffe://prod.example.com/github-worker/42
```

Agent Definition 本身不是 Principal。真正参与授权/审计的是 Agent Scope/Instance。LLM 本身也不是安全 Principal，它只是产生结构化决策的 Task。

### 8.3 Authority 直接复用 Effect System

Authority 不需要再造一套 `READ/WRITE/ADMIN` 抽象，建议直接表示可执行 Effect 的上界：

```text
Authority {
  github.issue.create<repo=foo/bar> count<=3
  github.issue.read<repo=foo/bar>
  network.read<host=docs.example.com>
}
```

Delegation 必须满足：

```text
Authority(child) ⊑ Authority(parent)
```

也即只能 attenuation，不能 amplification。

### 8.4 DelegationGrant

推荐正式引入：

```text
DelegationGrant {
  issuer
  grantee
  authority
  purpose
  run/scope binding
  expiry
  can_redelegate
  max_delegation_depth
  parent_grant
}
```

默认 `can_redelegate=false`。跨 Remote Agent/A2A、Durable Subflow、Trust Domain 时必须显式委托，不传递 ambient parent credential。

### 8.5 Credential Broker

Policy Engine 只做 authorization decision，不直接返回 secret。推荐：

```text
PolicyDecision
→ EffectGrant
→ CredentialBroker
→ CredentialLease
```

CredentialLease 短生命周期、audience/resource/operation-bound；最好由 Adapter/Sidecar 使用 raw credential，Task 本身只看到 opaque capability。

SPIFFE 的 workload identity、trust domain、短生命周期 SVID 很适合作为 Worker/Service identity 的参考，但它解决的是“哪个 workload 在执行”，不是“这个 workflow 代表谁拥有何种业务 Authority”。[R4]

### 8.6 Policy Engine 与 Enforcement 分开

借鉴 PDP/PEP 模型：

```text
PDP: Policy Engine
PEP: Task Runtime / Adapter / Credential Broker / Sandbox / Network Proxy
```

Policy Input 推荐包含：

```text
subject
actor
delegation_chain
workflow/run/scope/task
effect_intent
resource
data_classification
environment
budget
approval evidence
authentication assurance
```

PolicyDecision 不应只有 allow/deny，可以返回：

```text
ALLOW
DENY
REQUIRE_APPROVAL
REQUIRE_REAUTH
REQUIRE_STEP_UP
ALLOW_WITH_CONSTRAINTS
```

OPA 的“决策与执行分离、以结构化 input 返回结构化 decision”是很合适的参考。[R6]

### 8.7 Approval

Approval ≠ Delegation。Approval 只是满足某一次 Policy Obligation，不默认扩大长期 Authority。

Approval 必须绑定：

```text
approver principal
specific EffectIntent digest
purpose
expiry
one-time/replay policy
```

并支持：

```text
separation-of-duties
M-of-N
step-up authentication
```

若要扩大 Run Effect Envelope，应走显式 reauthorization/migration，而不是一个通用“Approve”按钮。

---

## 九、Provenance、Trust、Taint 与 Information Flow

### 9.1 四个概念必须分开

```text
Provenance  = 数据从哪里来、经过什么处理
Trust       = 当前系统对来源/声明给予什么程度信任
Taint       = 某值是否受某类来源影响
Information Flow = 这些标签的数据允许流向哪里
```

不要用一个 `trust_score=0.8` 压平所有维度。

### 9.2 ValueEnvelope 与 Provenance Graph

Workflow 数据仍是 `Value<T>`，Runtime 可以附加：

```text
ValueEnvelope<T> {
  data
  provenance_ref
  security_labels
  taints
}
```

Provenance 应形成 DAG：

```text
Summary
├── produced_by: Summarizer invocation
└── derived_from:
    WebPage A
    WebPage B
    Memory C
```

Policy 常态使用 Taint Summary，需要调查时展开完整 Lineage Graph。

### 9.3 Instruction Authority

Agent 系统需要特别引入 `Instruction Authority` 维度，例如：

```text
PLATFORM
WORKFLOW
USER
DELEGATED_AGENT
NONE
```

外部网页、Email Body、Repository File、MCP Tool Result、Remote Agent Artifact 默认：

```text
instruction_authority = NONE
```

即使内容写着“System message”“Ignore previous instructions”，自然语言本身也不能提升 authority。

关键不变量：

```text
Authentic source ≠ trusted instruction.
Natural-language content cannot self-elevate instruction authority.
```

### 9.4 Taint Propagation

默认保守规则：

```text
Taint(y)
=
Union(Taint(all inputs that may influence y))
```

LLM 被视为强 Taint Mixer：所有 model-visible input 都可能影响所有 ModelOutput，除非存在更强证明。

因此：

```text
external_untrusted web
→ ModelOutput
→ production.deploy EffectIntent
```

EffectIntent 的 influence labels 仍包含 `external_untrusted`。

### 9.5 Data Egress 与 Decision Influence 是两种不同 Flow

`Data Egress Flow`：

```text
confidential
→ external model
```

`Decision Influence Flow`：

```text
untrusted web
→ high-impact mutation decision
```

二者都需要 Policy，但不能混成一个风险标签。

### 9.6 Declassification 与 Endorsement

Taint/Label 不能只增不减，否则最终所有数据都被污染。需要特权操作：

```text
Declassification
  降低 confidentiality restriction

Endorsement
  提升 integrity/trust status
```

普通 Task 不能自行声称“已去掉 PII”或“已验证”。只有受信的 Declassifier/Endorser Contract 才能改变标签，并且该能力本身进入 Registry/Policy。

Sanitization 不等于 Endorsement：

```text
HTML stripped
≠ prompt-injection-free

schema valid
≠ semantically trusted
```

### 9.7 Memory Poisoning

Persistence does not imply trust。外部不可信内容写入 Memory 后，读取时仍保留原 provenance/taint。

建议 Memory namespace 区分：

```text
Working Memory
Observed Facts
Verified Knowledge
User Preference
Agent Learned Heuristic
```

例如 `external_untrusted → trusted_knowledge.write` 应默认 REQUIRE_ENDORSEMENT，而不是直接写入。

---

## 十、Scheduler、Worker 与 Multi-tenant Runtime

### 10.1 Scheduler 不决定 Workflow 逻辑

Kernel 决定：

```text
什么 TaskInvocation READY
```

Scheduler 决定：

```text
READY work 何时、在哪里、由谁获得执行机会
```

Scheduler 不决定 Branch、Retry、Loop、Agent Tool Choice。

Race 是特殊情况：物理 completion order 可以影响 winner，但一旦 `RaceWinnerChosen` 写入 Journal，未来 replay 不再重新竞争。

### 10.2 调度架构分层

不要做一个“超级 Scheduler”。推荐：

```text
Ready Work
  ↓
Admission / QoS
  ↓
Fairness / Priority
  ↓
Placement Matcher
  ↓
Lease / Dispatch
  ↓
Worker Pool
```

旁边：

```text
Capacity Controller / Autoscaler
```

### 10.3 ExecutionRequirements

Task Contract/Binding 可声明：

```text
Hard Constraints:
  browser capability
  GPU >= 1
  memory >= 16GB
  sandbox >= strong
  region/data residency
  trust domain
  component ABI

Soft Preferences:
  artifact locality
  warm model cache
  low load
  cheaper region
```

Capability 与 Resource Quantity 分开；Capability 不应只是 string list，而应支持版本/等级/属性。

Kubernetes 的 filter/score、taint/toleration、affinity，以及 Ray 的 resource requirement / scheduling strategy / placement group 都适合作为底层调度思想参考。[R7][R8]

### 10.4 Worker 是 Attempt Executor，不拥有 Run

```text
Run
→ TaskInvocation
→ TaskAttempt
→ ExecutionLease
→ Worker
```

Worker crash 后 Workflow 仍由 Journal/Runtime 服务拥有。

Worker 建议以 WorkerPool 管理；Pool 描述稳定 capability/trust/region/runtime，个体 Worker 是 ephemeral executor。

### 10.5 Pull + Lease + Fencing

推荐：Worker advertise readiness/slots，Scheduler 发放 Attempt Lease。

```text
Attempt A1
Lease generation=7
→ Worker 42
```

Lease 过期后：

```text
generation=8
→ Worker 51
```

旧 Worker 以 generation=7 提交 terminal Outcome 必须被拒绝。

但 stale worker 上报真实 Effect evidence 不能简单丢弃，应进入 reconciliation path。

### 10.6 三层 Concurrency

必须区分：

```text
Workflow Logical Concurrency
Workflow Execution Concurrency Limit
Worker Physical Slots
```

百万级 Map 不应一次物化百万 READY Invocation；使用 `Windowed Materialization` 与 Backpressure。

### 10.7 Fairness、Priority 与 Quota

三者是不同概念：

```text
Priority = 谁更急
Fairness = 不同租户长期如何分享资源
Quota    = 绝对不能超过多少
```

推荐 hierarchical fairness：

```text
Organization
→ Project
→ Workflow
→ Run
```

Priority 使用有限枚举/受政策控制，不允许所有人随意填最大整数。Priority 不能绕过 tenant quota、安全 placement 或 trust domain。

### 10.8 Spawn Budget

Agent 动态 fan-out 必须受到 Runtime Budget 约束，而不是模型想 spawn 多少就 spawn 多少：

```text
max_child_scopes
max_total_tasks
max_inflight_tasks
max_parallel_agents
```

### 10.9 单机与分布式语义一致

单机模式只是同一协议的退化：

```text
Local Queue / Local Worker / Local LeaseManager
```

分布式：

```text
Durable Queue / Remote Worker / Distributed LeaseManager
```

两者都遵循：

```text
ReadyWork → Lease → Attempt → Outcome
```

---

## 十一、Component Protocol、Worker Protocol 与 Cross-language ABI

### 11.1 三层协议模型

必须拆开：

```text
Component Contract
  描述“这是什么工作”

Invocation ABI
  描述“一次 Invocation/Attempt 怎么跨语言表达”

Worker Transport Protocol
  描述“Attempt 如何租给 Worker、heartbeat/cancel/complete”
```

MCP/A2A/HTTP/stdin 都是 Adapter/Transport，不定义 Kernel 内部语义。

### 11.2 Invocation ABI

跨语言 canonical object 推荐包含：

```text
AttemptEnvelope {
  invocation_id
  attempt_id
  task contract identity/digest
  input ValueEnvelope/ref
  ResourceHandle refs
  deadline/heartbeat policy
  security/effect grant refs
  budget refs
  trace context
  ExecutionLease
}
```

不要传递一个可无限扩展的万能 `ExecutionContext`。

### 11.3 Data ABI 与 Wire ABI 分开

组件 I/O 适合 JSON Schema/Type Schema，方便 Studio、validation、Tool schema。

Runtime Wire Control 更适合稳定 binary IDL（例如 Protobuf/gRPC），用于：

```text
Lease
Heartbeat
Cancel
Worker Registration
Outcome
Version Negotiation
```

Protobuf 对字段编号不可复用、客户端/服务端不能假设同步升级等原则很适合作为内部协议演化参考。[R9]

### 11.4 Worker Protocol 是 lifecycle protocol，不是简单 RPC

```text
Worker Ready
→ Attempt Lease
→ Accepted
→ Started
→ Heartbeat / Progress / Stream / EffectReceipt
→ Terminal Outcome
```

建议区分三条非终态通道：

```text
Control Channel
Progress Channel
Data Stream
```

Progress/Token Stream 默认 best effort，终态 Outcome durable。

### 11.5 AttemptStarted 是关键边界

Lease accepted 不代表业务代码已经开始。Worker 应先完成协议、Lease、Contract、Schema 和 Resource/Grant 验证，然后执行严格的 `Attempt Started Barrier`：

```text
Worker receives AttemptEnvelope
→ submit TaskAttemptStarted candidate
→ canonical Journal accepts Started
→ Worker receives StartPermit
→ only then execute effect-capable component code
```

在 Started 之前 Worker 崩溃，可以 re-place 同一 Attempt；Started 之后 v1 默认禁止透明重跑同一 Attempt，必须先 fence 旧执行并形成 `TaskAttemptLost`，后续是否重试由 Runtime 创建新的 AttemptId。

### 11.6 Outcome 与 Effect Evidence 分开

同一个 Attempt 最终只能有一个 terminal Outcome；重复相同 digest 是 idempotent ACK，不同 digest 是 ProtocolViolation。

但 EffectReceipt 可以在 Outcome 前增量上报；stale Lease 的 Outcome 可拒绝，真实外部 Effect evidence 不可因 Lease 过期而静默丢失。

### 11.7 Local In-process 也必须遵循同一 Invocation Model

本地 Python 函数只是：

```text
Invocation
→ Local Executor Adapter
→ function
```

仍然具有 invocation_id / attempt_id / cancel / outcome / effect semantics。Local Fast Path 是 transport optimization，不是第二套 Runtime。

### 11.8 MCP 与 A2A 的位置

MCP 适合作为外部 Tool/Capability protocol；其 Tasks extension 已提供 durable async task 的 get/update/cancel 等概念，但它不应替代内部 Worker Protocol，因为内部还需要 Lease/Fencing/Worker Registry/Placement/EffectReceipt 等。[R3]

A2A 适合作为 Remote Agent Boundary，提供 Agent Card、Task、Artifact、Streaming 等跨框架 Agent 交互抽象；Remote Agent 属于独立 Trust Domain，不是本平台 Worker。[R10]

---

## 十二、Conformance、Testing 与 Agent Evaluation

### 12.1 三类 Test Oracle

必须严格区分：

```text
Exact Oracle
Invariant / Model Oracle
Statistical Oracle
```

Exact：Type、Effect inference、Signal matching、Compatibility 等确定语义。

Invariant：并发、Crash、Lease、Durability 等不应断言某个唯一执行顺序，而是断言不变量。

Statistical：Agent/LLM 的任务成功率、引用正确率、成本、延迟、不安全动作率等。

### 12.2 Reference Semantic Interpreter

建议维护一个小而严格、低性能、单线程/虚拟时间的 Reference Interpreter。

```text
Specification
→ Reference Interpreter
→ Conformance Vectors
→ Local Runtime / Distributed Runtime / Future Runtime
```

生产 Runtime 必须与 Reference Interpreter 在 canonical event sequence 上语义一致。

### 12.3 Semantic Conformance Suite

Sequence / Select / Parallel / Race / Iterate / Retry / Await 都需要系统边界测试，而不是几个 unit test。

特别需要系统化探索 interleavings，例如：

```text
A complete
B complete
Parent cancel
Timer fire
Signal arrive
```

通过 virtual scheduler / virtual clock / event injection 做 model-based exploration。

### 12.4 Durable Crash Matrix

在关键边界植入 FailPoint：

```text
BEFORE/AFTER_JOURNAL_COMMIT
BEFORE/AFTER_OUTBOX_SEND
AFTER_ATTEMPT_START
AFTER_EXTERNAL_EFFECT
BEFORE/AFTER_RECEIPT_REPORT
BEFORE/AFTER_OUTCOME_COMMIT
```

每个点 crash/restart 后验证：

```text
no lost accepted signal
no stale lease commit
no blind retry of UNKNOWN effect
no Journal ref to nonexistent Artifact
no illegal state transition
```

### 12.5 Worker Protocol Conformance

Python/TS/Rust/Go Worker SDK 全部运行相同 test vectors：

```text
stale lease outcome
idempotent duplicate complete
conflicting duplicate complete
cancel race
heartbeat timeout
worker drain
unsupported contract
large artifact output
```

SDK 只能是 protocol client，不得偷偷实现 Retry/Policy/Workflow Control。

### 12.6 Component Contract Test

测试不只验证 schema，还要验证组件声明：

```text
Effect 上界
idempotency behavior
cancellation contract
failure mapping
receipt completeness
resource access boundary
```

Registry 中应区分：

```text
declared
tested
enforced
attested
```

不要把作者写了 `idempotent=true` 等同于平台已经证明。

### 12.7 Security Test

Authorization 与 Information Flow 分开测试，同时必须测试 PEP 真正执行拒绝。

特别推荐“故意恶意 Fake Model”：让它明确请求越权 Tool、泄露 Secret、无限 Spawn；系统必须证明 Runtime boundary 阻止，而不是测试“模型自己会不会拒绝”。

### 12.8 Agent Eval

Agent Eval 使用：

```text
Eval Dataset
Evaluator
Sampling Configuration
Repeated Runs
Metrics
```

比较 distribution，不用固定文本 snapshot。版本升级使用 paired evaluation：

```text
quality Δ
cost Δ
latency Δ
tool-call Δ
unsafe-action Δ
```

Runtime correctness 与 Model quality 必须是两套发布门禁。

---

## 十三、Registry、Package、Supply Chain 与 Marketplace

### 13.1 Registry 不是下载商店

建议拆开：

```text
Package Store
Registry Index
Resolver
Trust Store
Advisory Store
Marketplace
```

Marketplace 只是用户投影；真正底层是可验证的包与供应链系统。

### 13.2 Package Identity

执行身份：

```text
Logical Name
+ Logical Version
+ Content Digest
```

`name@version` 一旦发布必须唯一映射到 immutable digest。Version 表达逻辑演化，Digest 表达物理不可变身份。

OCI Descriptor 的 mediaType/digest/size 与 content-addressable DAG 是很合适的存储/分发参考，但 Agent Component 的语义模型不应等同于 OCI Image。[R11]

### 13.3 Package immutable，Assertions mutable

Package 本体：

```text
Contract
Implementation
Model-facing Metadata
Tests
SBOM refs
Provenance refs
```

发布后 immutable。

后来发现漏洞、publisher key compromise、compatibility evidence 等，通过独立：

```text
Advisory
Revocation
CompatibilityAttestation
SecurityAssessment
```

表达，不修改 Package bytes。

### 13.4 Contract Identity 与 Implementation Identity 分开

同一个逻辑 Contract 可以有多种实现：

```text
image.resize@3
├── amd64 CPU impl
├── arm64 CPU impl
└── CUDA impl
```

Scheduler 根据 platform/capability/trust policy 选择兼容实现。

### 13.5 Dependency 类型分开

不要把以下都放一个 `dependencies[]`：

```text
Semantic Package Dependency
Implementation/SBOM Dependency
Runtime Capability Requirement
External Resource/Protocol Requirement
```

Workflow Resolver 只解析 semantic dependency，产生 immutable `LockGraph`；Python/npm/system library 属于 implementation SBOM；browser/GPU 属于 Scheduler capability；GitHub/MCP/A2A endpoint 属于 Deployment Binding。

### 13.6 Lock Graph

Lock Graph 不只是平面版本列表，而是完整依赖图，并锁定：

```text
package origin
contract digest
package digest
model-facing metadata digest
effect signature digest
implementation selection policy
```

可以把它理解为一次 Workflow Build 所依赖全部可执行语义的 Merkle closure。

### 13.7 Policy-aware Resolver

Resolver 输入：

```text
Version Constraints
Compatibility Constraints
Trust Policy
Platform Policy
Registry Snapshot
```

不是“最高 SemVer 胜出”。

锁定后执行 Run 绝不能重新 `resolve latest`。

### 13.8 Yank / Revoke / Delete

严格区分：

```text
Yank    = 不再推荐新解析
Revoke  = 不再信任/禁止执行
Delete  = 极少使用的物理清理
```

历史 Run 所引用 immutable artifact 在 retention/audit 期间不能因 Yank/Revoke 被删除。

### 13.9 Supply-chain Evidence

Signature 只证明 signer 与 digest 关系，不证明代码安全。

建议多维证据：

```text
Artifact Integrity
Publisher Identity
Build Provenance
SBOM
Contract Conformance
Security Review
Runtime Enforcement Level
```

Sigstore 适合身份绑定的 artifact signing/transparency log；SLSA 与 in-toto 适合 build provenance/attestation framework。[R12][R13][R14]

### 13.10 Effect Signature 是 Security ABI

Agent Component 相比普通 package manager 最大的不同：升级不仅看 API/schema，还必须看：

```text
Effects
Resource Requirements
Credential Requirements
Information-flow behavior
Isolation
Model-facing Metadata
```

例如：

```text
v1 network.read<docs.example.com>
v2 network.read<*>
```

即使 I/O schema 不变，也属于 Security ABI Breaking Change。

`Installed ≠ Authorized`：获取/缓存 Package 不授予 Runtime Authority。

---

## 十四、Semantic IR 与 Workflow Compiler

### 14.1 三层表示

推荐正式区分：

```text
Authoring Representation
→ Canonical Semantic IR
→ Executable Plan
```

Authoring 层允许：

```text
Agent block
HumanApproval block
Template
Macro
Hole
UnresolvedReference
IncompleteBinding
```

Canonical IR 只保留正式语义：

```text
Value
Invoke
Await
Control
Resource
Expression
Composite/Subflow Ref
```

### 14.2 Canonical IR 结构

```text
Semantic Workflow IR
=
Structured Control Tree
+
Scoped Typed Dataflow Graph
+
Contract Tables
```

Dataflow 遵循 Scope/Dominance 规则：Sibling branch 内产生的 Value 不能在另一 sibling 中直接读取；Select 外部需要 typed merge；Loop-carried Value 显式演化。

### 14.3 Canonicalization

相同语义必须产生稳定 canonical representation/digest。UI 坐标、颜色、折叠、JSON key 顺序等 presentation-only change 不得改变 semantic digest。

### 14.4 编译器不是一个线性巨型 Pass

外部逻辑可以看成：

```text
AuthoringIR
→ CanonicalIR
→ ResolvedIR
→ CheckedIR
→ ExecutablePlan
```

内部建议逐步演化成增量 Semantic Query Engine：Transformation 与 Analysis 分开；Analysis 是显式输入的纯查询并可缓存。

重要 Analysis：

```text
TypeOf
ScopeValidity
EffectsOf
ResourceRequirements
LabelsOf
Cost/BudgetEnvelope
Compatibility
PolicyPreflight
```

Compiler 不允许 Analysis 偷读 `Date.now()`、current user、environment variable、registry.latest 等隐式全局状态。

### 14.5 Resolution Shell 与 Pure Compiler Core

Registry 查询先形成 immutable `ResolutionSnapshot`；后续 Compiler Core 只读取：

```text
CanonicalIR
ResolutionSnapshot
CompilerSemanticsVersion
```

相同输入必须产生相同 build output。

### 14.6 Static Violation 与 Runtime Obligation

Compiler 不可能静态证明所有东西。推荐正式引入：

```text
RuntimeObligation
```

分析结果可三态：

```text
PROVEN_SAFE
PROVEN_INVALID
RUNTIME_OBLIGATION
```

例如动态 repo：

```text
github.issue.create<repo=%input.repo>
```

Compiler 证明它仍在 Workflow Effect Envelope 内，但具体 repo 是否允许，生成 EffectAuthorization Obligation，交给 Runtime 在 concrete input 出现时检查。

### 14.7 Compile / Publish / Deploy / Run Admission 分开

```text
Compile
  程序语义是否成立

Publish
  是否可形成 immutable WorkflowVersion

Deploy
  Environment 是否满足静态要求

Run Admission
  当前 Principal/Quota/Live Safety 是否允许创建 Run

Runtime Enforcement
  每个动态 Obligation 是否在执行前满足
```

### 14.8 Diagnostics

Compiler Diagnostic 推荐包含：

```text
code
severity
primary semantic/source span
secondary spans
message
notes
related contracts
fixes
```

并区分：

```text
ERROR
WARNING
ADVISORY
RUNTIME_OBLIGATION
```

SourceMap 必须贯穿：

```text
Executable Op
→ Canonical Semantic Node
→ Authoring Construct
→ Studio Block / DSL source
```

### 14.9 Composite 与 Subflow

`Composite` 是编译期结构复用，可 inline 或通过 exported contract 做模块化分析，通常不拥有独立 durable run identity。

`Subflow` 是 runtime durable boundary，拥有独立 version/run/checkpoint/history/cancellation relation。

二者即使 UI 都画成“折叠块”，语义也不能混。

---

## 十五、Executable Plan 与 Runtime Interpreter Contract

### 15.1 Executable Plan 的定位

Executable Plan 不是“优化后的 workflow.json”，而是：

> 一个 immutable、content-addressed、versioned durable state-machine program。

逻辑组成：

```text
PlanProgram
Constant/Contract Tables
Static Analysis Summaries
Runtime Obligation Templates
Source Map
```

Plan 中没有 unresolved `latest`。

### 15.2 参数化 State Machine

不要把百万 Map 全部展开成 FSM states。Plan 保留参数化 opcode，与 Canonical IR 的公开核心语义对应：

```text
INVOKE@1
AWAIT@1
SEQUENCE@1
SELECT@1
PARALLEL@1
RACE@1
ITERATE@1
RETRY@1
TIMEOUT@1
HANDLE@1
SCOPE@1
COMPENSATION_SCOPE@1
CALL_SUBFLOW@1
```

Runtime 按 `Plan + State + Journal Event` 实例化状态转换。

### 15.3 Run-to-quiescence

Interpreter 接受 Event 后执行所有纯内部 transition，直到 durable boundary：

```text
需要 Task
需要 Timer
需要 Signal
需要 Policy/Approval
需要 Subflow
Workflow terminal
```

Sequence cursor 等机械 state mutation 不需要每次写 Journal。

### 15.4 三类 Transition

推荐区分：

```text
Mechanical Transition
Semantic Decision
External Command
```

例如：

```text
Sequence index++        # mechanical
SelectCaseChosen(A)     # semantic decision, Journal
ScheduleAttempt(...)    # command, Outbox
```

### 15.5 Runtime State

只保存最小充分的 Journal-derived Semantic State：

```text
run / ExecutionEpoch / Plan identity
ScopeInstance Tree
ComputationInstance states + CompletionLatch
Value bindings / immutable refs
Invocation / Attempt states
Wait / Timer / pending Signal mailbox
Child execution / Obligation outcomes
Effect observation summary
Compensation / recovery state
```

Budget/Usage/Quota、Worker Lease、Command Handler、Credential 等可以拥有独立 durable authoritative ledger，Semantic State只保留必要 refs。

不保存 Worker object、DB connection、OAuth token、Python closure、HTTP client、raw secret。

### 15.6 Opcode Semantics 版本化

推荐有 Runtime Semantics Profile：

```text
profile 3:
  SEQUENCE@1
  PARALLEL@2
  RACE@1
  ITERATE@2
  RETRY@2
```

Plan format version 与 semantic opcode version 分开，避免“序列化格式升级”被误认为“执行语义升级”。

### 15.7 Runtime Obligation Instance

Plan 中是模板：

```text
EffectAuthorizationTemplate
InformationFlowTemplate
CredentialRequirement
ResourceBindingCheck
```

Run 中产生 concrete `ObligationInstance`：

```text
PENDING
SATISFIED
DENIED
EXPIRED
```

Runtime 不重新遍历整个 Workflow 做 Compiler Analysis，只处理显式 Obligation。

### 15.8 Command Contract

Command 只表达 Interpreter 对外围执行系统的 durable logical request。v1 Core Command 收敛为：

```text
ScheduleTaskAttempt
CancelTaskAttempt
ScheduleTimer
CancelTimer
ArmWait
CancelWait
StartChildExecution
CancelChildExecution
EvaluateRuntimeObligation
CancelRuntimeObligation
```

Command ACK只表示 Handler 已 durable 接管请求，绝不代表 Task Started、Timer Fired、Child Started 或 Obligation Satisfied。真正影响 Workflow 的结果必须重新成为 Journal Event。

Command 具有 stable identity，可 at-least-once dispatch。

### 15.9 Local/Distributed/Reference Runtime 共用语义

```text
                    Plan Interpreter
                         │
       ┌─────────────────┼─────────────────┐
       ▼                 ▼                 ▼
Reference Ports      Local Ports       Distributed Ports
```

替换的是 Command Handler/Event Source，不是 Interpreter semantics。

---

## 十六、Observability、Debugger 与 Replay UX

### 16.1 六类观察数据

严格分开：

```text
Journal
Trace
Log
Metrics
Security Audit
Provenance / Lineage
```

物理上可以共享基础设施，但语义 API 不能统一成一个万能 Event Store。

OpenTelemetry 很适合承载 vendor-neutral trace/metric/log telemetry，但 Domain Model 不应反向由 OTel Span attribute 定义。[R15]

### 16.2 Journal 是 Timeline 主骨架

Debugger 的 semantic timeline 按 per-run Journal sequence 排序；timestamp 用于耗时/物理时间，不决定 Workflow control ordering。

```text
Journal seq = logical time
Timestamp   = physical time
```

### 16.3 Scope/Invocation/Attempt 分层 Trace

推荐：

```text
WorkflowRun
├── ScopeInstance
│   ├── TaskInvocation
│   │   └── TaskAttempt*
│   └── ScopeInstance
└── ...
```

Scope Tree 表示 ownership；Causality Graph 表示 Signal→Wait、Failure→Retry、Parent→Child Run 等因果链接，不强迫全部塞进单一 parent span tree。

### 16.4 多种 Debug View

至少应有：

```text
Workflow View
Execution View
Security View
Agent View
Data/Lineage View
```

普通用户默认看 Semantic Timeline，而不是原始 Journal Event Stream。

### 16.5 Step 语义

`Step` 不表示执行一行代码，而表示推进到下一个用户可观察 Semantic Boundary。

`Step Into / Over / Out` 基于 Computation/Scope，不需要 Agent 特例：Agent、Human Approval、Subflow 都可以用同一种调试语义。

同时可提供 Semantic Granularity 与 Attempt Granularity 两档。

### 16.6 Pause

Pause 不承诺物理世界瞬时冻结。推荐状态：

```text
PAUSING
→ DRAIN / CANCEL_ACTIVE / PAUSE_AT_SAFE_POINT
→ PAUSED
```

默认更安全的是 DRAIN：不启动新 work，等待 active owned work terminal 后进入 PAUSED。

### 16.7 Breakpoint

Breakpoint 可绑定：

```text
Before/After Computation
On Failure
On Retry
On Effect
On PolicyDecision
On Signal
On Scope Enter/Exit
```

但 Breakpoint 是 developer control，不是 security control，不能绕过 Policy/Approval。

### 16.8 Historical Replay 与 Fork

Historical Replay 只读，不调用 Model/Tool/DB/Effect；可以 seek 到任意 Journal seq，通过 Checkpoint + tail materialize 历史 State。

Fork 创建新 Run，可以修改 input/prompt/model/plan，但必须重新：

```text
Policy
Approval
Credential
Resource Binding
Effect Authorization
```

Fork 可以默认进入安全 sandbox/dry-run，降低调试生产 Effect 风险。

### 16.9 Agent Debugger

Agent 调试重点不是展示一大段 raw prompt，而是展示 Context Assembly：

```text
System Instructions
Workflow Instructions
User Goal
Memory
Retrieved Documents
Tool Definitions
Previous Turns
```

每部分显示：

```text
source
provenance
security label
token contribution
included/excluded reason
```

Tool Debugger 同时展示：

```text
Decision Chain:
Context → ModelOutput → ToolProposal → EffectIntent

Authorization Chain:
Subject → Delegation → Policy → Approval → Grant → Credential → Receipt
```

### 16.10 Debugger 不是 Security bypass

查看 Prompt、Value、Artifact、Tool args 本身也受 Authorization/Information Flow Policy。Secret 不因“Debug Mode”自动可见；Redaction 应在存储/出口前完成并保留 `[REDACTED: PII]` 之类结构化标记。

---

## 十七、Control Plane Resource Model 与 Workflow Lifecycle

### 17.1 不要把所有东西都叫 Workflow

推荐区分：

```text
WorkflowDefinition
WorkflowDraft
WorkflowVersion
Deployment
DeploymentRevision
WorkflowRun
```

关系：

```text
WorkflowDefinition
→ WorkflowDraft
→ Publish
→ WorkflowVersion
→ DeploymentRevision
→ Deployment
→ StartRun
→ WorkflowRun
```

### 17.2 WorkflowDefinition / Draft / Version

`WorkflowDefinition` 是稳定逻辑身份与 metadata container。

`WorkflowDraft` mutable、可不完整、支持协作编辑与 diagnostics。

`WorkflowVersion` 通过 Compiler/Publish 产生，绝对 immutable，包含/引用：

```text
Semantic IR digest
Executable Plan digest
LockGraph digest
WorkflowContract digest
Compiler/Semantics profile
```

Draft 不是“status=DRAFT 的 WorkflowVersion”；二者最好是不同资源类型。

### 17.3 Publish ≠ Deploy

Publish：产生可信、不可变的 WorkflowVersion。

Deploy：将某个已发布 Version 与 Environment/Bindings 组合，形成可运行部署。

升级 Deployment 不自动迁移既有 Run。

### 17.4 Deployment 与 DeploymentRevision

`Deployment` 是稳定运行槽位，例如：

```text
research-report-prod
```

`DeploymentRevision` 是不可变快照：

```text
workflow_version
Environment
ResourceBindings
IdentityBinding
PolicyBinding
ModelBindings
```

Rollback 也建议产生新的 DeploymentRevision，而不是无审计地把 current pointer 指回旧 row。

### 17.5 Environment

Environment 是正式 security/infrastructure boundary，而不是字符串：

```text
trust domain
region/data residency
runtime profile
policy binding
allowed worker pools
network zones
```

Environment 不承载隐藏业务 Control。

### 17.6 Configuration / Parameter / Secret 阶段分离

早期可先用下列阶段理解配置边界：

```text
Compile / Publish Semantic Inputs
Deploy-time Binding
Run-start Input / Semantic Parameters
Live Safety / Operational State
Security Material
```

但 v1 不再提供一个统一 `config` 概念。最终模型按 **Binding Time + Semantic Influence + Dataflow Visibility + Authority** 四个维度拆分：Workflow Input、Publish-time Semantic Parameter、RunSemanticParameter、Deployment Binding、Control Plane Environment、Operational Config、Feature Flag classes 与 Secret/Credential 各自拥有不同生命周期和兼容性规则。

核心规则是：**任何可能改变相同 canonical facts 下 Semantic Trace 的值，必须在 Published Semantic Baseline、RunManifest 或后续 Canonical Runtime Fact 中显式出现；不存在 ambient semantic configuration。** 因此 Workflow/Component 不允许通过 `env()`、`ctx.config.get()`、`feature_enabled()` 等未声明接口读取隐藏语义输入。完整规范见 22.20。

### 17.7 Resource / Identity / Policy / Model Binding

WorkflowVersion 声明逻辑 requirement；DeploymentRevision 绑定实现策略；Run/Invocation 记录真实解析结果。

ResourceBinding 绑定 logical resource identity，而不是简单 connection string；Credential 仍由 Broker 实时兑现。

IdentityBinding 定义 service/user-delegated/explicit-role 等执行身份策略。

PolicyBinding 区分 semantic baseline 与 Live Safety Overlay。

ModelBinding 绑定 provider/model selection policy，不把某个模型 provider 写死进 Kernel。

### 17.8 TriggerBinding

Trigger 应绑定 Deployment，而不是某个 WorkflowVersion：

```text
Webhook/Cron/API
→ TriggerBinding
→ active DeploymentRevision
→ StartRun
```

事件接受时立即解析具体 DeploymentRevision 并持久化，避免 trigger 与 deploy 同时发生时产生时序歧义。

Trigger 不进入 Workflow Body。

### 17.9 CRUD 与 Execution Commands 分开

Control Plane mutable resources 可以使用 desired-state/reconciliation 模型，例如 Deployment。

Run 状态不能通过通用 PATCH 修改。用户只能发送：

```text
StartRun
PauseRun
ResumeRun
CancelRun
ForkRun
MigrateRun
SendSignal
```

Run 状态来自 Journal + Interpreter。

### 17.10 Run Lifecycle

顶层 phase 建议保持粗粒度：

```text
PENDING
RUNNING
PAUSING
PAUSED
CANCELLING
SUCCEEDED
FAILED
CANCELLED
```

同时另有 conditions：

```text
WAITING_SIGNAL
WAITING_TIMER
WAITING_POLICY
WAITING_CAPACITY
WAITING_APPROVAL
```

`RUNNING` 不代表此刻一定有 Worker 占用。

### 17.11 Deploy、Disable、Migrate 的边界

```text
Disable Deployment
= 阻止新的 Run
≠ 自动取消已有 Run
```

```text
Deploy v18
= 新 Run 使用 v18
≠ existing Run 自动从 v17 迁移
```

```text
MigrateRun
= same run_id, new ExecutionEpoch
```

```text
ForkRun
= new run_id
```

### 17.12 外部一级资源与内部实体分开

用户/Control Plane 顶层资源建议保持有限：

```text
Tenant / Organization
Project
WorkflowDefinition
WorkflowDraft
WorkflowVersion
Environment
Deployment
DeploymentRevision
TriggerBinding
ResourceBinding
IdentityBinding
PolicyBinding
ModelBinding
Run
```

Runtime 内部一等实体：

```text
ExecutionEpoch
ScopeInstance
TaskInvocation
TaskAttempt
WaitCondition
EffectIntent
EffectReceipt
```

不需要机械地把每个内部对象暴露成顶层 CRUD API。

---

## 十八、可视化编排、Studio 与开发者体验

### 18.1 Studio 是 Semantic Program 的 Projection Editor

Studio 不直接编辑 Runtime 状态机，也不把 Canvas JSON 当执行真源。推荐：

```text
Authoring / Presentation
→ Compiler
→ Canonical Semantic IR
```

Presentation 与 Semantic 分离：

```text
workflow.semantic.json
workflow.presentation.json
```

同一 Semantic IR 可以有 Developer View、Executive View、Security View 等多个 presentation。

### 18.2 结构化 Region 优于任意网格 Edge

Sequence、Parallel、Race、Iterate、Scope 等应表现成结构化 region。数据线与 Control ownership 在视觉上也应不同。

UI 连线建议至少区分：

```text
Value/Dataflow
Signal/Event
Resource Binding
```

Control 大部分由 region nesting 表达，而不是所有东西都画 `next` edge。

### 18.3 Agent Block 支持 Progressive Disclosure

顶层：

```text
Research Agent
  turns=20
  tools=38
  cost=$...
```

进入内部后：

```text
Context Build
Model Invocation
Tool Proposals
Tool Invocations
Stop Condition
```

再进入 Attempt/Effect/Security 细节。

### 18.4 AI Builder 不能绕过 Compiler

AI 生成的是 AuthoringIR Patch：

```text
Generate
→ Compile
→ Diagnostics
→ Repair
→ Compile
```

AI 不直接生成可执行 Plan，也不自己成为正确性的最终权威。

### 18.5 SDK 与 Studio 使用同一种 Semantic IR

Python/TS DSL 是 Authoring Projection，不应拥有另一套控制语义。推荐：

```text
Studio
Python DSL
TS DSL
API
AI Builder
→ same Compiler
→ same Semantic IR
```

### 18.6 CLI

原报告中 `validate / inspect / replay` 比单纯 `run` 更重要的判断保留。随着本版对象模型收敛，CLI 最终应围绕：

```text
validate
diff
publish
deploy
run
inspect
watch
history/replay
fork
migrate
registry/component inspect
```

形成统一控制面体验。

> 注：API/CLI/SDK 的精确 Surface 仍属于后续可继续单独细化的开放章节；本版只固定它们必须投影同一 Control Plane Resource Model 和 Semantic IR，不能各自发明语义。

---

## 十九、开源项目与标准：应该分别参考什么

### OpenCode

最值得借鉴：

```text
Harness
Client/Server
Tool + MCP
Permission model
Agent/Sub-agent UX
Event/remote client architecture
```

尤其其权限系统把 Tool execution 放在模型之外进行 allow/ask/deny 控制，符合本报告“Model Intent ≠ Authorization”的原则。[R1]

不建议直接复制：把 coding-agent 产品对象变成通用 Workflow Kernel primitive。

### Agno（原 Phidata / phi）

最值得借鉴：

```text
Agent / Team / Workflow / Runtime 分层
Agent capability composition
```

适合作为高级 Composite/Harness API 参考，不作为 Durable Kernel 规范来源。

### LangGraph

最值得借鉴：

```text
graph/state/checkpoint
HITL
deterministic + agentic hybrid workflow
```

但本设计进一步要求 Structured Control Tree 与 Dataflow Graph 分离，避免长期停留在 generic graph topology inference。

### Temporal

最值得借鉴：

```text
durable execution
history/event driven recovery
worker/task queue separation
retry/cancellation/long-running workflow reliability
```

本设计与之不同的核心选择：不以“重放任意用户 Workflow code”作为规范中心，而以 explicit Semantic IR / Executable Plan Interpreter 为中心。[R2]

### MCP

适合作为：

```text
Tool / external capability boundary
```

Tool schema、metadata、异步 Task 扩展都值得借鉴；但不承担内部 Worker Lease/Fencing/Scheduler 协议。[R3]

### A2A

适合作为：

```text
Remote Agent boundary
```

Agent Card、Task、Artifact、Streaming/Push Update 支持不同 Agent 系统之间互操作；Remote Agent 必须视为独立 Trust Domain。[R10]

### SPIFFE / OAuth Token Exchange / OPA

分别适合参考：

```text
Workload Identity / Trust Domain
Delegation vs Impersonation
Policy Decision vs Enforcement
```

但不要把其中任意单一标准误认为完整 Agent Authority 模型。[R4][R5][R6]

### Kubernetes / Ray

适合参考：

```text
Hard filter + soft score
resource/capability placement
node affinity / taints
placement group / gang scheduling
```

Workflow Scheduler 负责“哪个业务 Work 获得执行机会”，Kubernetes 等更底层 Infrastructure Scheduler 负责 Worker Pod/VM 的节点放置，两层不要合并。[R7][R8]

### OCI / Sigstore / SLSA / in-toto

适合作为：

```text
content-addressed package distribution
artifact signing / transparency
build provenance
attestation framework
```

Agent Component Registry 还需在这些基础上增加 Effect Signature、Model-facing Metadata、Compatibility、Runtime Contract 等领域语义。[R11][R12][R13][R14]

---

## 二十、典型场景验证

### 20.1 完全确定性的审批工作流

```text
Trigger
→ Task[LoadOrder]
→ Select[amount > threshold]
→ HumanApproval Composite
→ Task[UpdateOrder]
```

没有 LLM，仍然完整使用同一 Kernel。HumanApproval 展开为 Task + Await + Validate。

### 20.2 Research Agent

```text
Scope ResearchAgent
└── Iterate(max_turns)
    ├── Invoke ContextBuilder
    ├── Invoke ModelCall
    └── Select
        ├── Final
        └── ParallelMap(ToolTask)
```

Agent 完全由通用 Computation 组合。

### 20.3 Coding Agent with Workspace

```text
Workspace Resource
→ Agent Scope
→ Tool Tasks[read/edit/test]
→ Workspace Branch/Overlay
→ Explicit Merge
```

并行 coding subagents 不共享隐式 mutable filesystem state。

### 20.4 Remote A2A Agent

```text
Invoke(StartRemoteAgent)
→ Await(RemoteCompletionSignal)
→ Invoke(FetchArtifact)
```

Remote Agent 获得显式 DelegationGrant，不获得用户原始 Token；其 Effect assurance 可能是 `ATTESTED` 而不是本地 `ENFORCED`。

### 20.5 Long-running Human Workflow

```text
Task[CreateRequest]
→ Await[Decision]
→ Timer/Race
→ Select[approved/rejected/timeout]
```

等待数周不占 Worker；Signal 可提前到达并在 Mailbox durable 存储。

### 20.6 Million-item Map

Logical：

```text
Iterate 1,000,000 items
```

Physical：

```text
materialization window = 1000
max inflight = 100
worker slots = dynamic
```

通过 Backpressure 限制，不创建百万 READY Task。

### 20.7 Effectful Retry Crash

```text
payment.charge succeeds
→ worker crashes before outcome journal
```

Runtime 不 blind retry。根据 idempotency/reconcile API 获取 external status；无法确认则 `EffectState=UNKNOWN`。

### 20.8 Prompt Injection / Memory Poisoning

```text
External Web(external_untrusted)
→ ModelOutput(inherits taint)
→ Memory.write(trusted_knowledge)
```

Information Flow Policy 要求 Endorsement，Persistence 不自动提升 Trust。

### 20.9 Workflow Version Upgrade

```text
prod Deployment v17 → v18
```

新 Run 使用 v18；旧长期 Run 保持 v17 Plan，除非在 safe point 经过显式 MigrationPlan 进入新 ExecutionEpoch。

---

## 二十一、最终架构边界与正式不变量

经过更新后的调研，推荐把正式规范拆成以下相互关联但可独立演进的文档：

```text
1. Semantic IR Specification
2. Runtime Semantics Specification
3. Effect & Resource Contract Specification
4. Durable Execution / Journal Specification
5. Versioning & Migration Specification
6. Identity / Delegation / Policy Specification
7. Information Trust / Provenance Specification
8. Scheduler / Worker Runtime Specification
9. Component / Invocation / Worker Protocol Specification
10. Registry / Supply-chain Specification
11. Compiler Specification
12. Executable Plan / Interpreter Specification
13. Conformance & Evaluation Specification
14. Observability & Debugging Specification
15. Control Plane Resource Model Specification
```

总体架构公式：

```text
Agent Workflow Platform
=
Semantic Computation IR
    (Value + Task + Await + Control + Resource)

+ Multi-pass / Incremental Workflow Compiler
    (Type + Scope + Effect + Flow + Policy + Compatibility)

+ Immutable Executable Plan

+ Journal-driven Durable Interpreter

+ Capability / Delegation / Information-flow Security

+ Contract-based Component & Resource System

+ TaskInvocation / Attempt / Lease / Worker Protocol

+ Multi-tenant Capability Scheduler

+ Content-addressed Registry & Supply-chain Trust

+ Conformance / Evaluation Framework

+ Observability / Debugger / Replay Projections

+ Control Plane Lifecycle

+ Visual / DSL / API Projections
```

### 最核心的设计不变量

1. `Agent` 不是 Kernel primitive，而是 Composite。
2. `Task` 是有限主动计算；`Await` 是 durable 被动等待。
3. v1 Kernel opcode 是封闭的结构化集合，不支持 GOTO、共享可变 workflow variable 或隐藏控制流。
4. `Workflow IR = Structured Control Tree + Scoped Typed Dataflow Graph + Contract Tables`，不是 arbitrary DAG。
5. `Value` immutable；`Resource` 表示共享可变世界；Runtime Control State 由 Kernel 管理。
6. `Effect` 是 Security/Replay/Compatibility ABI；始终满足 `Actual ⊆ Granted ⊆ Declared`。
7. Effect执行必须同时落在 Delegation、EffectGrant、ResourceHandle、Current Safety Policy 和 Attempt Fence 的交集内。
8. `Model output` 只能产生请求/意图，不能产生 Authority；External Data 不能自行提升 Instruction Authority。
9. Delegation、Scope、ResourceHandle 都只能 attenuation，不能 authority amplification。
10. Runtime recovery 解释 pinned Plan + canonical Journal，不重新执行历史外部 Effect。
11. Journal 是 Workflow Control Truth；Checkpoint 是缓存；Budget/Usage/Quota 等可以是独立 durable ledgers。
12. 每个 Run 在 v1 只有一个 Semantic Owner；canonical fact acceptance 通过 per-Run 单一顺序和 fencing 序列化。
13. Runtime Command是 requested action，Journal Event是 accepted fact；ACK绝不直接推进 Workflow。
14. Worker 只拥有 Attempt Execution Lease，不拥有 Run；`TaskAttemptStarted` 是 effect-capable execution 的 point-of-no-return barrier。
15. Started 前可以 re-place 同一 Attempt；Started 后 v1 默认通过 Lost + 新 Attempt恢复，而不是透明重跑同一 Attempt。
16. Task Outcome authority 与 Effect/Usage Evidence authority分离；stale Outcome可拒绝，真实 late Receipt不可静默丢失。
17. Retry、Reconciliation、Compensation三者严格分开；`UNKNOWN` Effect必须先 Reconcile，Compensation不是 rollback。
18. Published Package、WorkflowVersion、Plan、LockGraph、Canonical Value/Artifact 都 immutable/content-addressed。
19. Stable Logical ID、Revision ID、Content Digest、Runtime Occurrence ID、Authority/Lease ID不得混用。
20. Workflow Failure、Control Termination、Recovery Block、Operational Condition、Runtime Incident、Platform Degradation是不同错误平面。
21. Deploy ≠ Publish ≠ Run Migration；升级 Deployment 不重写已有 Run。
22. Studio、SDK、CLI、AI Builder 都受同一 Compiler、Canonical IR 和 Runtime Semantics约束。

---



---

## 二十二、v1 规范化机器基线（讨论冻结版）

本节不是新的产品层架构，而是把前文已经形成的设计结论收敛成一个可供 Reference Interpreter、Compiler、Worker SDK、Control Plane 和 Conformance Suite 共同使用的 v1 基线。这里的目标是确定**哪些语义必须一致**，而不是规定数据库、编程语言、消息队列或部署拓扑。

### 22.1 Semantic Kernel、Opcode 与 Computation Contract

v1 Kernel 固定为：

```text
Value + Task + Await + Control + Resource
```

`Effect / Identity / Policy / Budget / InformationFlow / Provenance / Usage / Recovery / ExecutionRequirements` 是横切 Contract，不增加任意节点类型。

公开核心 opcode 冻结为：

```text
INVOKE@1
AWAIT@1
SEQUENCE@1
SELECT@1
PARALLEL@1
RACE@1
ITERATE@1
RETRY@1
TIMEOUT@1
HANDLE@1
SCOPE@1
COMPENSATION_SCOPE@1
CALL_SUBFLOW@1
```

`CallComposite` 只存在于 authoring/compiler 层并在发布前 lower；Trigger 是 Deployment Binding；HumanTask 是 Interaction + Await 等高阶组合；MCP/A2A/HTTP/LLM 都通过 Task/Resource/Subflow adapter进入，不增加 Kernel opcode。

Computation Contract最终按如下维度理解：

```text
Computation<I, O, Φ, ε, R>
+ InformationFlow F
+ Usage U
+ Recovery K
+ ExecutionRequirements X
```

`Φ` 使用 nominal Failure Row；`ε` 使用 Effect Algebra；`R` 是资源需求上界。Component substitution 必须同时满足输入逆变、输出协变、Failure 不扩张、Effect 不扩张、Resource 不增强以及 Recovery/InfoFlow/Assurance 的兼容性约束。

### 22.2 Canonical Semantic IR v1

Canonical IR 是长期 semantic ABI，而不是 Studio persistence、SDK AST 或 Runtime State。顶层包括：

```text
format/version
workflow semantic identity
interface
references / resources
structured body
semantic extensions
```

核心约束：

- 结构为 `Structured Control Tree + Scoped Typed Dataflow Graph + Contract Tables`；
- Workflow data 使用 SSA-like immutable ValueRef；
- Expression 只能是 pure、bounded、deterministic、versioned AST；
- 未知 semantic field/opcode fail closed；
- semantic defaults在发布时 materialize，不能依赖未来默认值；
- `semantic_id` 在 WorkflowDefinition lineage 内稳定，用于 diff/migration/debug；
- 编译生成 semantic_id 必须由 origin + expansion path + expansion semantics version 确定；
- Canonical IR 禁止运行时 ID、Worker/Region、Credential secret、presentation geometry、mutable editor state。

v1明确不支持：arbitrary graph/GOTO、共享可变 workflow state、运行时代码注入、未知 semantic extension、ambient credential、隐式 Resource、无界内部 loop/self-modifying workflow。

### 22.3 Identity Namespace、Digest 与 Canonical Serialization v1

对象身份必须分为五类：

```text
Stable Logical Identity
Immutable Revision Identity
Content Digest
Runtime Occurrence Identity
Authority / Lease Identity
```

并进一步把对象分为：

```text
Entity      # mutable logical identity
Revision    # immutable version + digest
Artifact    # immutable content-addressed object
Occurrence  # runtime instance
Authority   # grant/lease/permit lifecycle object
```

禁止使用一个 hash ID同时承担上述职责。`Name / Stable ID / Content Digest` 是三个不同字段；`(name, version)` 必须解析到 immutable digest。

Digest必须 domain-separated，例如：

```text
H("agentflow.plan.v1\0" + canonical_bytes)
H("agentflow.schema.v1\0" + canonical_bytes)
```

Canonical Object Encoding v1至少保证：UTF-8、唯一 object key、确定性 key ordering、明确 integer/decimal/timestamp/duration表示、无 NaN/Infinity、数组有序、missing 与 null 区分。Digest针对 canonical semantic representation，不针对任意 authoring JSON/YAML bytes。

运行时 subordinate object（Timer、Wait、Command、Recovery invocation等）优先使用 deterministic derived ID；`command_id` 明确排除 Region、OwnershipEpoch、wall clock和delivery attempt。

### 22.4 Runtime Semantics Profile v1

Runtime被定义为确定性 durable transducer：

```text
step(Plan, Profile, State, Event?)
→ State' + Decisions* + Commands*
```

同一 Plan/Profile/Canonical Event history必须产生 observationally equivalent Semantic Trace。这里承诺的是**确定性 replay/materialization**，不是每次重新运行都会获得相同现实世界结果。

所有 Computation instance采用：

```text
ACTIVE → COMPLETING → TERMINAL
```

`CompletionLatch` 与 `TerminalOutcome` 分开：Race、Timeout、Parallel fail-fast、Cancellation可以先锁定 primary cause，再等待 owned children / cleanup / compensation收束。

核心语义包括：

- Sequence：前一 child Success 才进入下一 child；
- Select：唯一 case，`SelectCaseChosen` 是 semantic decision；
- Parallel FAIL_FAST：第一个 canonical non-success锁定 primary cause，取消 sibling并等待 terminal；
- Parallel COLLECT：全部终止后按 branch canonical order聚合；
- Race：记录 `RaceWinnerChosen`，loser effect不会因为取消而消失；
- Iterate：Foreach固定 source snapshot，While记录 Guard Decision，所有 loop都有显式 bound；
- Retry：Computation Retry与Task Attempt Retry分离；0 backoff无需虚假 Timer；
- Timeout：Timer/Child通过 Journal order竞争，Timer赢后取消 child但必须等其收束；
- Handle：处理 Failure和显式 TimedOut，不捕获 parent Cancelled；
- Scope：只做 ownership/authority/resource/budget attenuation；
- CompensationScope：显式 FORWARD → STOPPING_FORWARD → RECONCILING → COMPENSATING → RECOVERY_BLOCKED/TERMINAL；
- CallSubflow：owned durable child boundary，v1没有 detached child。

### 22.5 Journal Event Schema v1 与 Canonical Acceptance Gate

Journal Event是“已经被平台接受、足以影响未来恢复的 canonical fact”。统一 Envelope：

```text
JournalEventEnvelope {
  event_id
  event_key
  run_id
  execution_epoch
  journal_seq
  event_type
  payload_schema_version
  logical_time
  source
  target
  causation
  payload
  payload_digest
  dedupe_digest
  committed_at
}
```

`journal_seq` 是唯一 per-Run canonical semantic order；`event_id` 是 record identity；`event_key` 是 logical dedupe identity。相同 key + same dedupe digest是重复；相同 key + incompatible digest必须 conflict/fail closed。

v1核心事件族：

```text
RunStarted
RunPauseRequested / RunResumeRequested / RunCancelRequested
MigrationApplied
InvocationPreparationOutcome
TaskAttemptStarted / TaskAttemptOutcome / TaskAttemptLost
EffectReceiptRecorded / EffectReconciliationResolved
SignalAccepted / TimerFired
ObligationResolved
ChildStarted / ChildTerminal
SelectCaseChosen
RaceWinnerChosen
IterationGuardDecision
RetryDecision
AttemptRetryDecision
ManualRecoveryResolution
```

Heartbeat、Progress、Worker assignment、TimerScheduled、WaitArmed、SequenceAdvanced、ValueBound、Command ACK等不进入 Canonical Journal。

v1引入 per-Run `Canonical Acceptance Gate`：一个 Run 同时最多有一个尚未被 Interpreter处理到 quiescence 的 external canonical fact。流程为：

```text
append external fact
→ gate closes
→ interpreter run-to-quiescence
→ internal decisions + state + outbox atomic commit
→ gate opens
```

Ingress可以 active-active durable queueing，但 canonical Run Journal的事实顺序只由该 Gate/owner决定。

### 22.6 Runtime Command Schema v1 与 Outbox

Command只表示 requested action，不表示事实。统一 Envelope：

```text
RuntimeCommandEnvelope {
  command_id
  run_id
  execution_epoch
  command_type
  payload_schema_version
  issuer
  target
  causation
  payload
  payload_digest
}
```

Core Command：

```text
ScheduleTaskAttempt / CancelTaskAttempt
ScheduleTimer / CancelTimer
ArmWait / CancelWait
StartChildExecution / CancelChildExecution
EvaluateRuntimeObligation / CancelRuntimeObligation
```

`command_id` 是 logical action identity；物理 delivery拥有独立 `delivery_id` 和 `dispatch_fence`。Owner failover后使用新的 fence重发同一个 command_id。

Handler必须 durable accept before ACK。ACK只表示命令被接管，不表示 Task Started、Timer Fired、Child Started或Obligation完成。所有 semantic result仍需作为 Candidate Event重新进入 Journal。

所有 Start/Cancel对必须支持 cancel-before-start tombstone。Outbox transport retry使用相同 command_id/payload，不消耗 Workflow Retry，也不生成新的 Attempt。

### 22.7 Execution State v1 与 Reducer Invariants

Runtime materialization分为：

```text
SemanticExecutionState
DurableExecutionRefs
MaterializationCache
```

只有第一项是 `Plan + Journal` 唯一决定的规范 Workflow Control State。Budget/Usage/Quota、Worker Lease、Credential、Command Handler等可以拥有独立 durable authority。

Canonical state至少包含：

```text
active ExecutionEpoch / Plan
Run control state
ComputationInstance states
Scope tree
Invocation / Attempt logical states
Wait / Timer / pending Signal mailbox
Child execution outcomes
accepted Obligation outcomes
Effect observation summary
Compensation / recovery state
immutable Value bindings
```

不包含：Worker object、Region、Queue position、Quota余额、Credential secret、Command ACK、HTTP client、thread/future/closure。

每个 Computation occurrence通过 deterministic `ComputationInstanceId`区分 PlanOp和运行时 occurrence；Control Tree与Scope Tree分别由 `parent_instance_id` / `owning_scope_id` 表达。

`StateInvariantChecker` 必须检查：Scope无环、Terminal immutable、parent terminal前owned child已收束、Attempt ordinal连续且最多一个业务 non-terminal Attempt、Signal最多消费一次、Race winner唯一、Foreach max_inflight、Compensation entry只对应已知 APPLIED effect等。

### 22.8 Reducer / Interpreter Algorithm v1

Runtime正式拆为：

```text
Pure Semantic Reducer
+
Live Execution Coordinator
```

Reducer只能读取 immutable Plan/Profile/State/Journal facts/Value refs，禁止读取 wall clock、random、Policy Server、Scheduler、Worker、网络、Credential Store等。

Live Turn：

```text
apply accepted external fact
→ deterministic internal microsteps
→ compute/apply semantic decisions
→ advance to quiescence
→ derive execution requirements / command obligations
→ append_turn atomically
```

Replay与Live必须共用同一 Decision函数：Live模式 `compute → record`；Replay模式 `compute → verify recorded`。若当前 Runtime算出的 Decision与 Journal记录不同，必须 `JournalSemanticMismatch`，不能相信任一方继续。

Quiescence定义为：没有新的 accepted external fact/live requirement resolution时，不存在 enabled semantic microtransition。Command ACK不会打破 quiescence。

Control expansion顺序固定为 Sequence lexical、Parallel/Race canonical branch order、Foreach ascending index、Retry increasing ordinal、Compensation reverse effect-commit journal_seq，但这些只是逻辑创建顺序，不规定物理 Worker完成顺序。

### 22.9 Task / Worker Protocol v1

严格区分：

```text
TaskInvocation
TaskAttempt
AttemptExecutionLease
WorkerExecution
```

Task Retry创建新 Attempt而非新 Invocation；pre-start lease reassignment可以复用同一 Attempt，Started之后 v1默认不得透明重跑同一 Attempt。

`AttemptEnvelope`至少绑定 Run/Epoch/Invocation/Attempt、Task Contract、Implementation ref、Input ref、ExecutionRequirements、Authority refs、ResourceHandle refs、Credential requirements、Usage/Recovery contract和Execution Lease。

最关键的是 `Attempt Started Barrier`：

```text
receive AttemptEnvelope
→ submit Started candidate
→ Started enters canonical Journal
→ receive StartPermit
→ only then run effect-capable component code
```

Started之前Worker丢失可安全 re-place；Started之后Worker失联必须 fence并形成 `TaskAttemptLost`，后续是否Retry由Runtime决定并创建新Attempt。

Heartbeat只属于Lease Service；Progress/Streaming默认属于Telemetry；Terminal Outcome必须durable input/output ref先落盘后提交。

Task Outcome authority与Effect/Usage Evidence authority分离：stale generation的Terminal Outcome拒绝，但可信 late EffectReceipt/UsageReceipt仍可记录。

### 22.10 Effect / Resource Adapter Protocol v1

正规的 side-effect路径冻结为：

```text
EffectDeclaration
→ EffectIntent
→ EffectGrant
→ Resource Adapter / PEP
→ External Resource
→ EffectReceipt
```

`EffectIntent`必须绑定 Run/Epoch/Scope/Invocation/Attempt/Generation、Operation、Logical Resource Identity、argument digest + authorization projection、Subject/Actor/Delegation、Purpose、Provenance/Labels/Taints、Idempotency与Recovery Contract。

`EffectGrant`是 exact-intent、默认 single-intent capability。它不扩大 Delegation/Scope/ResourceHandle ceiling。Adapter在真实 provider call前再次验证 Attempt fence、Grant、ResourceHandle、Current Safety Overlay、CredentialLease和provider idempotency contract。

Receipt独立于Task Outcome，状态固定为：

```text
NOT_APPLIED
APPLIED
PARTIALLY_APPLIED
UNKNOWN
```

UNKNOWN不是Failure别名，必须先 Reconcile。Reconciliation本身是受控 Effect，返回 APPLIED / NOT_APPLIED / PARTIALLY_APPLIED / STILL_UNKNOWN。Compensation也是新的 `EffectIntent(purpose=COMPENSATION)`，不能修改旧 Receipt，也不能注册成同一 forward ledger的新补偿项。

高 assurance Adapter应尽量做到 provider effect → durable receipt → return success，但架构明确承认外部系统与Receipt Store通常无法通用原子提交，因此不能宣称 universal exactly-once。

### 22.11 Value / Artifact / Provenance v1

Workflow中跨 durable boundary流动的是 immutable typed refs，而不是进程对象。正式区分：

```text
Value Store       # typed workflow data
Artifact Store    # immutable large/object-like data
Provenance Store  # immutable lineage/trust DAG
Memory Resource   # mutable/queryable external resource
```

数据identity至少分为：

```text
ContentDigest
TypedContentDigest
ValueSemanticDigest
```

因此 same bytes + different Type/Provenance/Labels/Taints 不一定是同一个 Semantic Value。Ref不是 bearer capability；知道 ValueRef/ArtifactRef不代表有读取权限。

统一创建路径：

```text
validate type
→ canonical serialize
→ compute provenance/labels/taints
→ durable persist
→ immutable ref
```

`durable-before-reference`成为全局不变量：Journal、Command、Task Outcome、Signal、Loop snapshot、RecoveryCapsule在引用数据前，被引用对象必须先达到Run要求的durability level。

Provenance是不可变DAG，默认Taint传播保守；schema validation/sanitization不自动提升Trust。Declassification/Endorsement必须是显式、受Authority控制的 transformation，并产生新的 Value，即使 payload bytes相同。

### 22.12 Failure / Incident / Operational Condition v1

禁止一个万能 `Error` 覆盖所有情况。必须区分：

```text
Semantic Failure
Control Termination
Recovery State
Operational Condition
Runtime / Protocol Incident
Platform Degradation
```

`FailureValue`进入静态 Failure Row，可由Retry/Handle处理；TimedOut与Cancelled是独立Control Outcome，Parent Cancelled不可被Handle吞掉。

`RECOVERY_BLOCKED`是durable non-terminal Recovery State，不可被业务Handle绕过。`WAITING_QUOTA / WAITING_CAPACITY / NO_MATCHING_WORKER / WAITING_SIGNAL` 等只是Operational Condition，不进入Journal、不消耗业务Retry；如果希望等待转成语义失败，必须显式使用Timeout/Timer。

`RuntimeInvariantViolation / JournalSemanticMismatch / ProtocolConflict / CommandConflict / ArtifactIntegrityViolation / DurabilityInvariantViolation`属于Runtime Incident，默认阻塞semantic progression而不是伪装成Workflow Failure。Platform service outage属于Degradation，通常等待服务恢复继续。

API/Debugger必须至少分别展示：

```text
Outcome
Conditions
Incidents
Recovery Status / Cause
```

### 22.13 Type、Failure、Resource、Usage Contract补充

Data Type采用 nominal + structural混合模型。公开 ABI/security object优先 nominal；Record内部局部结构可structural。`missing != null`，Variant默认closed，Map只允许string key，Set与任意Any不进入v1核心。

Task substitution约束：

```text
I_old <: I_new
O_new <: O_old
Φ_new ⊆ Φ_old
ε_new ⊑ ε_old
Resource requirement不增强
InfoFlow不放宽
Recovery assurance不降低
```

Resource链路固定为：

```text
ResourceType
→ ResourceSlot
→ ResourceBinding
→ Logical Resource Identity
→ ResourceHandle
→ ResourceAdapter
→ Physical Resource
```

Usage/Cost/Quota严格分开：Semantic Hard Budget使用Reserve → Execute → Settle；UsageReceipt是immutable measured fact；Quota/Admission/Fairness是平台治理，不等于Workflow Budget。UNKNOWN Effect默认继续占用Effect Budget直到证明NOT_APPLIED。

### 22.14 Identity、Policy、Obligation 与 Information Flow补充

Identity、Authority、Credential三者独立。Agent Definition不是Principal，LLM不是Security Principal。默认使用Delegation而非Impersonation，且：

```text
A_task ⊑ A_agent ⊑ A_run ⊑ A_delegator
```

Policy结果抽象为：

```text
Decision + Constraints + Obligations + Evidence
```

Obligation不是Workflow Task。Security Approval与Business Human Task严格分开；Approval绑定 exact intent_digest，并不能扩大原有 Authority ceiling。

`EffectiveGrant`由Declared envelope、Run delegation、Scope attenuation、ResourceHandle capability、Current Safety Overlay、Effect Budget等取交集。Live Safety只能收窄旧Run，不能静默扩大能力。

External content默认 `instruction_authority=NONE/EXTERNAL_DATA`；自然语言不能自我提升。EffectIntent必须携带provenance/labels/taints，以同时执行Authority check、Effect envelope check、InfoFlow check和Obligation check。

### 22.15 Interaction、Trigger、Remote Interop补充

Trigger负责 External Event → new Run；Signal负责已存在Run的durable message；Await负责Workflow内部等待；Interaction是durable外部协作对象，但不是Kernel opcode。

HumanTask是Interaction + Await/Timeout/Select等高阶组合。Claim/assignment/comment/reminder属于Interaction Service；最终typed submission才投影为Signal或Security Obligation evidence。

Remote protocol是integration ABI而不是Workflow language。MCP Tool默认投影为RemoteTask/Resource operation；A2A durable Task可投影为RemoteChildExecution。远程能力发现必须先形成immutable RemoteContractSnapshot并进入LockGraph，运行时不能发现“latest”然后改变已发布Run语义。

未知远程Effect必须保守视为 `⊤`，不能当作纯操作。Remote auth只证明远端身份，不自动等于本地授权；Credential Broker签发remote-specific credentials，禁止转发Control Plane credential。

### 22.16 Multi-tenant Scheduler 与 Multi-region Ownership补充

Scheduler只决定 ready work何时/在哪里运行，不决定Workflow业务路径。治理pipeline：

```text
READY
→ Governance Eligibility
→ Hierarchical Fair Queue
→ Priority
→ Placement
→ Atomic Admission Reservation
→ Dispatch
```

Quota、Fairness、Priority、Capacity、Placement各自独立。`NO_MATCHING_WORKER`与`WAITING_CAPACITY`是Operational Condition；Scheduler不能自行产生业务Retry或Failure。

Multi-region原则是：平台可以active-active，但单个Run同一时刻只有一个Semantic Owner。`RunOwnership`拥有独立 OwnershipEpoch/fence；OwnershipEpoch不进入Semantic identity或command_id。旧Owner写Journal/首次dispatch新Command必须被fence。

自动跨Region failover只有在不会丢失已确认execution facts的Durability Profile下才安全；RPO>0副本不得自动继续旧Run。ResourceHandle/CredentialLease不迁移，failover后从pinned logical resource identity和current policy重新materialize。

### 22.17 Conformance、Formal Boundary 与 Reference Interpreter

规范优先级：

```text
Normative Spec
> Formal Model
> Reference Interpreter
> Conformance Vectors
> Production Implementation
```

Reference Interpreter是小、严格、单线程/虚拟化的 executable oracle，不是规范本身。Differential conformance比较的是 Semantic Trace：Decisions、logical Commands、canonical Outcomes、observable Values和Scope lifecycle，不比较WorkerId、DB布局、Region、network retry等operational细节。

Conformance profiles至少分为：

```text
CORE_SEMANTICS
COMPILER
DURABLE_RUNTIME
DISTRIBUTED_WORKER
EFFECT_SECURITY
RECOVERY
MULTI_TENANT
MULTI_REGION
REMOTE_INTEROP
```

正式性质优先表达Safety和条件化Liveness；外部系统exactly-once、Agent quality、远程系统correctness都不能被Runtime语义证明替代。Agent Eval继续使用统计Oracle，与Runtime exact conformance分开。

### 22.18 Control Plane、API、SDK 与生命周期补充

Control Plane Resource与Runtime Facts分开：

```text
WorkflowDefinition / Draft / Version
Environment
Deployment / DeploymentRevision
Run
```

Published WorkflowVersion、Plan、LockGraph immutable；DeploymentRevision是环境/binding snapshot。TriggerBinding指向Deployment，启动时原子解析具体Revision并写入RunManifest。

API必须区分Resource CRUD与Execution Commands。Run状态不可PATCH，只允许Start/Pause/Resume/Cancel/Fork/Migrate/Signal等显式命令。副作用控制命令使用Idempotency-Key + request digest。

Authoring SDK、Control Plane SDK、Component/Worker SDK建议分包；Studio/Python/TS/AI Builder均输出AuthoringIR并经过同一个Compiler。宿主语言if/for不能在Runtime ValueRef上偷偷执行控制流。

### 22.19 关键后续修订对照

为了避免实现团队按早期讨论字面实现，以下后续结论明确覆盖前期简化：

| 早期表达 | v1 最终结论 |
|---|---|
| `Computation<I,O,ε>` | 扩展为 `I/O/Φ/ε/R + F/U/K/X` 多维 Contract |
| 普通 Scope 含补偿 | 补偿改为显式 `COMPENSATION_SCOPE@1` |
| Control无Handle | 加入 `HANDLE@1` |
| CredentialHandle作为Value | Credential不进入普通Workflow Value，使用Broker Lease |
| Journal记录EffectGrant | Grant进入Security/Effect Ledger；Journal记录真正影响恢复的Receipt/Resolution |
| Signal必须等Await | `SignalAccepted`可以早于Await，使用pending mailbox |
| Timer schedule写Journal | 只写`TimerFired`，ScheduleTimer是Command |
| Retry都等价 | Transport Retry、Attempt recovery、Attempt Retry、Computation Retry严格分层 |
| Worker lease迁移同Attempt | 仅Started前可透明re-place；Started后默认Lost + 新Attempt |
| Budget/Usage都在Run state | 使用独立durable ledgers，Journal只保存控制边界事实 |
| 远程协议成为新节点 | MCP/A2A始终为Adapter/Remote Contract，不新增Kernel opcode |
| Active Region属于语义 | Region/OwnershipEpoch是operational fencing timeline，不进入Semantic State |
| Error是统一状态 | Failure/Termination/Recovery/Condition/Incident/Degradation严格分层 |
| Value只按内容hash | Semantic Value identity还绑定Type/Provenance/Labels/Taints |
| `config: Map<String,Any>` 可统一承载运行配置 | ambient semantic config 禁止；语义值必须在 Publish / RunStart / Canonical Fact 三条路径之一固定 |
| Feature Flag可在执行中读取 current value | Semantic Flag必须在Publish或RunStart resolve并pin；Routing/Operational/Safety Flag分层处理 |
| Secret/环境变量可直接注入Task | raw Secret不进入Workflow Value/Plan/Journal/RunManifest；通过CredentialBinding/Broker Lease兑现 |
| 所有规范可共用一个“v1版本号” | Semantic IR、Runtime Semantics、Journal、Worker Protocol、Encoding、Policy等独立版本化 |


### 22.20 Configuration / Parameter / Secret / Environment Model v1

v1 明确禁止把不同生命周期和安全性质的值重新混成一个 `config: Map<String,Any>`。配置判断统一使用四个维度：

```text
Binding Time
+ Semantic Influence
+ Dataflow Visibility
+ Authority
```

Binding Time 至少区分：

```text
AUTHORING
COMPILE
PUBLISH
DEPLOY
RUN_START
LIVE_EXECUTION
```

同时按语义作用区分：

```text
SEMANTIC_BUILD
SEMANTIC_RUN
DEPLOYMENT_BINDING
LIVE_SAFETY
OPERATIONAL
SECURITY_MATERIAL
```

最核心不变量：

```text
Any value that may affect Semantic Trace
MUST be either:

1. pinned into the published semantic baseline, or
2. pinned into RunManifest at Run creation, or
3. introduced later as an accepted canonical runtime fact.

There is no ambient semantic configuration path.
```

#### Workflow Input

Workflow Input 是 per-Run typed 业务数据。Input Schema 属于 Workflow Semantic Contract；具体输入值不改变 Plan Digest。StartRun 时必须经过 type validation / canonical serialization / provenance attach / durable PutValue，最终 RunManifest 只引用 immutable `workflow_input_ref`。Run创建后 Input immutable；后续新数据通过 Signal、Interaction、Task Result等 canonical facts进入，而不是 PATCH 原始输入。

#### Compile / Publish Semantic Inputs

Compiler内部优化、线程数、cache路径等只要保持 observational equivalence，属于 Implementation Config，不进入 Semantic Identity。会改变合法 Semantic Trace 的 compiler feature/profile/macro semantics则必须版本化并进入 Build Identity。

Publish-time Semantic Parameter允许 Draft阶段存在 hole/parameter，但 Publish 时必须全部 resolve并物化成 Canonical IR/Plan普通常量，例如 `max_turns`、Retry/Timeout policy、Prompt template、Tool Contract set。Publish 后不保留 `${param}` 这种可再次解析的语义占位符；参数变化产生新的 WorkflowVersion/Plan Digest。Semantic defaults也必须在发布制品中显式物化，不能让旧 Artifact依赖未来默认值。

#### Deployment Binding

Deployment Binding负责把已发布的 logical contract映射到某个 Environment中的具体 Resource、Identity、Model、Remote Endpoint、Component Implementation。它不改变 Plan Digest，但每次变化形成 immutable `DeploymentRevision`。StartRun必须原子解析 active DeploymentRevision并写入 RunManifest；已有Run不跟随 Deployment mutable pointer漂移。

允许 compatible physical late binding，例如 Worker实例、CredentialLease、资源Replica或兼容Adapter binary，但只能在 RunManifest已固定的 logical identity / binding contract / compatibility envelope内选择，不能通过物理绑定扩大Effect、Authority或Resource requirement。

#### Environment 与 OS Environment Variable

Control Plane `Environment` 是正式 Security / Residency / Durability / Governance boundary，包含 trust profile、residency policy、network policy、allowed worker/runtime、durability/governance profile等。

OS `process.env` / `getenv()`则默认禁止成为 Workflow Semantic dependency。它只允许用于进程bootstrap、基础设施wiring和其它已证明non-semantic的 implementation config。Canonical Expression Language与Component Runtime高 assurance profile都不暴露 `env()` / `currentConfig()` / `systemProperty()` 之类隐藏语义接口。

#### Secret / Credential

Secret不是Workflow Value，Credential不是普通Config。正式对象分为：

```text
SecretRef
CredentialBinding
CredentialLease
```

raw secret material绝不进入 Plan、Journal、Task Input、RunManifest、Artifact或普通 Value。Credential Broker在Effect/Resource执行边界签发short-lived、audience/resource/operation-bound Lease；最好由Adapter/Sidecar消费，Component不看到raw token。

Secret bytes rotation如果logical Principal/Authority不变，不改变Semantic Baseline；如果Principal/Delegation发生变化，则属于 IdentityBinding变化，不能伪装成普通secret rotation。

#### RunSemanticParameter

v1允许Plan显式声明一组 `RunSemanticParameter`：它们在 StartRun时才resolve，但属于本次Run不可变的semantic constants，例如 `max_agent_turns`、workflow deadline、bounded search limit、声明过的model behavioral parameter。

它们与Workflow Input分离：Input是业务Dataflow并带Provenance/Taint；RunSemanticParameter属于Runtime semantic configuration，只能被Plan中预先声明的slots读取。StartRun必须形成完整immutable Parameter Snapshot并写入RunManifest；执行中不能PATCH。Parameter只能在Compiler已知的bounded domain中选择或收窄，不能动态增加未知Tool/Package、注入arbitrary code或扩大Effect Envelope。

#### Feature Flag

“Feature Flag”不进入Kernel，而拆成四种机制：

```text
Semantic Feature Flag
Release Routing Flag
Operational Implementation Flag
Safety Kill Switch
```

Semantic Flag必须在Publish或RunStart resolve并pin；执行过程中不得读取current value。Release Routing只在Run Admission时选择具体DeploymentRevision，随后Run固定。Operational Flag只有在保证observational equivalence时可live update。Safety Kill Switch属于Live Safety Overlay，只能立即收窄Authority，不能扩大已有Run能力。

因此 v1 没有 `feature_enabled("x")` 这种 Workflow builtin。真正业务实验变量应作为 Workflow Input 或显式 RunSemanticParameter。

#### RunManifest 配置基线

RunManifest的配置相关内容至少固定：

```text
WorkflowVersion / PlanCoreDigest
RuntimeSemanticsProfile
DeploymentRevision / BindingSnapshot
WorkflowInputRef
RunSemanticParameterSnapshot
Resource / Identity / Model binding baseline
Delegation baseline
Environment / Environment policy baseline
Durability profile
Required runtime profiles
```

RunManifest immutable。改变Semantic Baseline只能通过新Run或正式Migration/new ExecutionEpoch；compatible physical late binding不修改Manifest。

这一模型的最终判定表是：

```text
Business data
→ Workflow Input

Semantic constants
→ Publish Parameter / RunSemanticParameter

Physical implementation choice
→ Deployment Binding

Security narrowing
→ Live Safety Overlay

Execution authority material
→ Credential Broker

Infrastructure tuning
→ Operational Config

Mutable application state
→ Resource
```

如果某个值从“Operational Config”开始被业务代码读取并改变Workflow-visible behavior，它必须升级成显式Semantic Parameter/Input；不允许继续以“只是一个配置项”为理由留在ambient runtime。

### 22.21 Normative Specification Packaging / v1 Freeze Matrix

到本节为止，v1架构不再继续增加新的大模块，而正式转入规范封板。所有设计项分成四个成熟度等级：

```text
FROZEN_V1
OPTIONAL_PROFILE_V1
INTERNAL_IMPLEMENTATION_CHOICE
EXPERIMENTAL_OR_DEFERRED
```

`FROZEN_V1` 表示：一旦Artifact/Run/Contract声明使用 `@1`，不同实现必须满足相同 observable semantics；若合法旧输入与历史在新规则下产生不同 Semantic Trace，通常必须引入新的版本线或 RuntimeSemanticsProfile。

`OPTIONAL_PROFILE_V1` 表示规范本身已冻结，但实现可以不声明支持；一旦声明Profile就必须满足对应Conformance。

`INTERNAL_IMPLEMENTATION_CHOICE` 表示规范刻意不规定具体算法、存储或拓扑，只通过不变量约束实现。

`EXPERIMENTAL_OR_DEFERRED` 表示不属于v1兼容性承诺，不允许Production实现通过“隐藏扩展”把它变成事实标准。

#### 独立版本线

v1不是一个全局单版本号，而是一组独立version lines：

```text
Semantic IR Format
Canonical Encoding
Type / Failure System
Effect Algebra
Resource Contract
Runtime Semantics Profile
Journal Event Schema
Runtime Command Schema
Component Contract
Worker Protocol
Policy Evaluation Contract
Interaction Contract
Remote Interop Contract
Canonical Object Model
Control Plane API
Conformance Profiles / Suite Revision
```

例如 Worker Protocol升级不自动意味着 Semantic IR升级；Journal schema增加non-semantic字段也不等于Race semantics变化。RunManifest只pin真正属于该Run semantic baseline的版本与digest。

#### FROZEN V1

以下架构和语义正式封板：

```text
Semantic Kernel ontology
Canonical opcode set
Canonical Semantic IR semantics
Type / Failure model
Effect algebra
Structured concurrency semantics
Await / Signal / Timer semantics
Resource Type / Ref / Handle / Binding model
Identity / Delegation / Authority model
Policy Evaluation / Runtime Obligation semantics
Information Flow / Provenance semantics
Budget / Usage core semantics
Durable Journal semantics
Runtime Command semantics
Execution State semantic projection
Reducer / run-to-quiescence semantics
TaskInvocation / Attempt model
Worker Started Barrier / Fencing semantics
Effect / Resource Adapter / Receipt / Reconciliation semantics
Compensation / Recovery semantics
Value / Artifact / Provenance semantics
Failure / Incident / Operational Condition model
Configuration / Parameter / Secret / Environment model
Canonical object identity / typed ID / digest model
Formal conformance boundary
Control Plane resource lifecycle semantics
```

Canonical Runtime opcode set最终固定为：

```text
INVOKE@1
AWAIT@1
SEQUENCE@1
SELECT@1
PARALLEL@1
RACE@1
ITERATE@1
RETRY@1
TIMEOUT@1
HANDLE@1
SCOPE@1
COMPENSATION_SCOPE@1
CALL_SUBFLOW@1
```

`CallComposite`只存在于Authoring/Macro/Module层并compile away；LLM、HTTP、MCP、A2A、HumanTask等不得变成Production-only hidden semantic opcode。

#### OPTIONAL PROFILE V1

以下能力规范稳定，但不是所有实现必须支持：

```text
MULTI_TENANT@1
INTERACTION@1
REMOTE_INTEROP@1
MULTI_REGION@1
advanced Cost/Billing profiles
high-assurance Data Governance profiles
```

Core Runtime即使不实现Human inbox、MCP/A2A、多Region或多租户Scheduler，也仍可完全conform to Core。

Security Approval语义本身仍属于Core Policy Contract；Interaction Service只是可选UX/infrastructure实现。Remote protocol具体wire版本也独立于平台RemoteInterop Profile。

#### INTERNAL IMPLEMENTATION CHOICE

以下内容刻意不进入长期ABI：

```text
Journal / Outbox / Registry具体存储引擎
Checkpoint物理格式与频率
Scheduler具体Fairness算法
Worker queue topology / pull-vs-push implementation
Worker RPC具体框架
Artifact physical CAS布局
Trace backend
Region placement算法
Reference Interpreter语言
Production Runtime语言
Formal verification工具
UI framework / Studio persistence schema
ID实际分配算法（满足opaque/non-reuse/collision requirements即可）
```

实现自由必须满足规范不变量。例如Scheduler算法可以不同，但不能绕过Quota、Placement、Lease/Fencing；Journal存储可以不同，但必须满足per-Run total order、canonical acceptance和durability要求。

#### EXPERIMENTAL / DEFERRED

v1明确不承诺下列能力：

```text
arbitrary GOTO / unstructured graph / gateway zoo
shared mutable workflow variables / universal StateStore
runtime arbitrary code eval
unknown semantic extensions
open-ended public Any / Set / general user-defined generics
detached child / detached subflow / detached interaction
transparent migration or replay of Started Attempt
checkpointable/resumable physical Attempt
force preemption after AttemptStarted
cross-resource distributed transactions / general transaction scope
multi-writer same Run
automatic failover from acknowledged-history-losing RPO>0 replica
dynamic Effect-envelope widening
runtime addition of unknown Tool/Package
production “latest” remote capability discovery
automatic arbitrary remote provider substitution
arbitrary debugger state mutation
self-modifying workflow topology
universal exactly-once external effect guarantee
```

这些能力未来可以通过新的Profile或语义版本引入，但v1实现不得以私有隐式语义实现后声称仍是相同Core semantics。

#### 规范与实现的优先级

正式优先级：

```text
Normative Specification
> Formal Model
> Reference Interpreter
> Conformance Vectors
> Production Implementation
```

Reference Interpreter是executable oracle但不是规范本身。规范正文应明确标注 `NORMATIVE / RECOMMENDED / INFORMATIVE`，避免把“建议使用gRPC、Worker Pull”等工程偏好误写成语义ABI。

#### Semantic Breaking Change判定

下列变化默认属于semantic breaking：

```text
old legal Plan becomes invalid
same old Plan/history produces different semantic decision/outcome
Effect envelope can widen
Failure surface changes incompatibly
Resource requirements strengthen
Information Flow relaxes
Recovery guarantees weaken
old canonical bytes are reinterpreted differently
```

而更快Scheduler、新Checkpoint压缩、Debugger UI、Trace细化等在保持observational equivalence时不属于Semantic Breaking Change。

#### 当前真正尚未完成的“硬门槛”

架构层已足够冻结。Public v1 Artifact之前仍需完成的工作不再是新架构发现，而是把现有语义写成精确机器规范：

```text
Canonical Encoding v1 exact byte grammar
Canonical Semantic IR exact IDL
Executable Plan exact IDL
Journal Event exact IDL
Runtime Command exact IDL
Component Contract schema
Resource Contract schema
Worker Protocol wire IDL
Failure / Incident / Condition reserved registries
ID / digest domain-separation registry
13 opcode normative sheets
Core executable conformance vectors + cross-language golden vectors
```

因此从本节之后，项目正式由“架构设计阶段”切换到“规范编码阶段”。

---

## 二十三、规范冻结后的制品化阶段：状态、边界与工作方式

v1 Architecture Discovery 已经关闭。此后的默认工作不再是继续增加“大架构层”，而是把已经确认的语义编码成**唯一、可哈希、可重放、可跨语言验证、可发布的规范制品**。这一阶段的产物不是聊天记录，也不是某一种语言的生产实现，而是 AFIDL、Canonical Encoding、Normative Rule Registry、Closed Tables、Golden Vectors、Reference Interpreter 与 Conformance Corpus。

### 23.1 成熟度术语

本报告后续统一使用以下成熟度：

```text
FROZEN
  = v1 语义边界已经确定；破坏性变化默认进入 vNext。

EXACT_MODEL_COMPLETE
  = 字段、状态、identity、durability、fencing 与 invariants 已经机器级定义。

SEMANTICS_COMPLETE
  = 状态转换、authority、failure/cancellation/recovery 规则已经闭合。

CONFORMANCE_SPEC_COMPLETE
  = 已明确 Exact / Invariant / Negative / Fault 测试边界。

SOURCE_ARTIFACT_PENDING
  = 规范内容已冻结，但正式 AFIDL/table/golden 文件尚需物化。

REFERENCE_ORACLE_REQUIRED
  = v1 RC 前必须由独立 Reference Interpreter 覆盖。

OPTIONAL_PROFILE
  = 实现可以不支持；一旦声明支持就必须满足完整 profile。

DEFERRED_VNEXT
  = 明确不属于 v1，不允许作为“高级 v1 模式”偷渡。
```

“设计冻结”与“制品冻结”必须分开：当前核心语义已经 Design Freeze；真正的 Artifact Freeze 只有在 AFIDL、Rule ID、Closed Tables、Golden Corpus、Compatibility Manifest 与 Release Manifest 全部生成并通过一致性检查后才成立。

### 23.2 当前完成度

截至本版，核心模块均已达到 v1 规范冻结所要求的语义成熟度：

| 规范层 | 当前状态 | 核心冻结点 |
| --- | --- | --- |
| Canonical Encoding / AFIDL | `FROZEN` | ACE1、closed IDL、domain-separated digest、canonical scalar/record/variant encoding |
| Semantic IR / Executable Plan / Resolver | `FROZEN` | 结构化控制树、typed dataflow、13 opcode、closed Plan、deterministic LockGraph |
| WorkflowVersion / Deployment / StartRun | `FROZEN` | Publish≠Deploy≠StartRun，Run pin immutable baseline |
| Journal / Canonical Acceptance / AppendTurn | `FROZEN` | per-Run total order、one external stimulus per turn、atomic decision/state/outbox commit |
| Reducer / SemanticExecutionState | `FROZEN` | Journal→Reducer 是唯一 semantic mutation path，stable occurrence identity |
| Runtime Command / Handler | `FROZEN` | 11 commands、deterministic CommandId、durable ACK、cancel tombstone |
| Worker / Effect / Resource | `FROZEN` | Started Barrier、Intent→Grant→Dispatch→Receipt、UNKNOWN/reconcile、no blind retry |
| Value / Artifact / Provenance / InfoFlow | `FROZEN` | immutable semantic identity、lineage、labels/taints、instruction authority |
| Failure / Recovery / Incident | `FROZEN` | Failure、Control Termination、Recovery、Operational Condition、Incident 分层 |
| Identity / Delegation / Policy / Budget | `FROZEN` | no ambient authority，Approval≠Delegation，Budget/Usage/Quota 分离 |
| Scheduler / Multi-region Ownership | `FROZEN` | Scheduler 只决定 WHEN/WHERE；per-Run single fenced canonical writer |
| Checkpoint / Archive | `FROZEN` | checkpoint 是派生缓存；v1 禁止 destructive semantic squashing |
| Audit / Security Evidence | `FROZEN` | Primary Security Evidence + tamper-evident Audit Projection，Audit 不驱动 Reducer |
| 13 Opcode Normative Semantics | `FROZEN` | 每个 opcode 的 state/entry/microstep/cancel/recovery/replay 已统一 |
| Conformance / Reference Oracle | `FROZEN` | Exact / Invariant / Statistical 三类 oracle，Reference Interpreter 不是规范本身 |
| Spec Packaging / Versioning | `FROZEN` | immutable content-addressed release、compatibility/upcaster/errata 模型 |
| SDK / Visual / Public Projection | `FROZEN` | public mutation 必须映射既有 Core operation；Projection 不创造新语义 |

### 23.3 现在允许继续做什么

剩余工作按以下顺序推进：

```text
1. 物化正式 AFIDL / closed registry / normative tables
2. 分配稳定 SpecificationId / RuleId / field_no / case_no
3. 生成 ACE1 / digest / ID / state / command Golden Vectors
4. 实现独立 Reference Interpreter
5. 生成 crash / race / fencing / recovery / security Conformance Corpus
6. 做至少两种独立语言的 canonical encoding 交叉验证
7. 生成 Compatibility Manifest / Upcaster Registry / Spec Release Manifest
8. 进入 release-candidate validation
```

只有发现已经冻结的不变量互相矛盾，才允许重新打开 Architecture RFC。普通字段落盘、文件拆分、IDL生成、Rule ID补齐、测试向量增加、Projection完善，都不再属于“继续设计架构”。

### 23.4 Change Classification

从现在开始，任何新需求必须先分类：

```text
Presentation Change
Projection Change
Optional Profile
Compatible Schema Addition
Normative Clarification
vNext Semantic Change
```

凡是会改变 `Journal history / StateDigest / Command identity / Opcode transition / Effect reality handling / Run authority / Recovery outcome / Canonical race order` 的提案，默认属于 `vNext Semantic Change`，不能直接修改 v1。

---

## 二十四、v1 规范编码：已确认机器基线（Actual Normative Source 汇总）

本章只整理已经在讨论中明确确认的机器语义。它是第 22 章“架构冻结基线”的编码化补充：第 22 章解释系统必须满足什么；本章说明这些结论在机器规范中应如何被唯一表示。若早期概念性表述与本章冲突，以本章和后续独立 IDL/registry 为准。

### 24.1 Canonical Encoding / AFIDL v1

机器规范采用 `ACE1` 作为 deterministic schema-directed UTF-8 JSON subset，采用 `AFIDL1` 作为 schema source of truth。核心约束：

```text
JSON Number forbidden for machine identity fields.
Machine numeric primitives encode canonically as strings.
Strings normalize to NFC.
Map/object keys sort by unsigned UTF-8 bytes.
Duplicate keys and NFC-equivalent duplicate keys are invalid.
Missing and null are distinct; Option/Variant are explicit.
```

机器 primitive 集包括 `unit/null/bool/int32/uint32/int64/uint64/decimal/float64/string/bytes/timestamp/duration`；其中 Workflow CORE primitive 更窄，v1 不把 `float64/uint*` 作为普通 Workflow semantic primitive。`float64` 若用于机器对象，以 IEEE-754 binary64 big-endian 16-char lowercase hex 表示，禁止 NaN/Infinity，`-0` canonicalize 为 `+0`；timestamp/duration 使用 int64 microseconds。

所有摘要都使用 domain separation：

```text
HashV1(domain, payload)
= SHA256("AF\0" || domain || "\0" || payload)
```

典型 digest domain 包括 `semantic-ir`、`plan-core`、`run-manifest`、`lock-graph`、`journal-event-payload`、`runtime-command-payload`、`semantic-execution-state`、component/resource contract 与 conformance object。Opaque ID prefix 与 derived-ID domain 必须集中登记，不能由实现自行拼接字符串形成事实 ABI。

### 24.2 Canonical Semantic IR / Executable Plan / Resolver

Canonical Semantic IR 采用：

```text
Structured Control Tree
+ Scoped Typed Dataflow Graph
+ Closed Contract Tables
```

顶层包含 workflow interface、type definitions/imports、resource slots、run parameters、body 与 semantic version refs。Authoring `CallComposite` 可存在于编辑层，但发布前必须 lower；Runtime 只识别 13 个核心 opcode：

```text
INVOKE@1
AWAIT@1
SEQUENCE@1
SELECT@1
PARALLEL@1
RACE@1
ITERATE@1
RETRY@1
TIMEOUT@1
HANDLE@1
SCOPE@1
COMPENSATION_SCOPE@1
CALL_SUBFLOW@1
```

`ExecutablePlan` 由 `CanonicalIR + ResolutionSnapshot/LockGraph + CompilerSemanticsVersion` 确定。`PlanCore` 是 immutable/content-addressed durable state-machine program；SourceMap、build report 等非语义材料不改变 `plan_core_digest`。Plan 关闭所有 registry resolution，包含静态 contract/resource/signal/requirement/profile tables、closed expression language、RuntimeObligation templates 和参数化 Program，但不包含 Worker、Region、CredentialLease 或物理 endpoint。

Resolver 语义固定为 `agentflow.resolver-semantics@1`。v1 不要求全局 single-version SAT；依赖按 edge 解析最高 eligible version，允许多个 package version 共存。`ResolutionSnapshot` 与 `LockGraph` 都是 content-addressed；fresh resolution 与 locked rebuild 分开，locked rebuild 不允许静默 substitution。Yanked 与 Revoked 的资格规则分开；semantic dependency graph v1 要求 acyclic。

### 24.3 WorkflowVersion、DeploymentRevision 与 Run baseline

控制面正式区分：

```text
WorkflowDefinition  = mutable logical authoring root
WorkflowDraft       = mutable authoring state
WorkflowVersion     = immutable published semantic build
Deployment          = mutable routing pointer for future Runs
DeploymentRevision  = immutable deployment snapshot
RunManifest         = immutable per-Run admission root
ExecutionEpochManifest = immutable per-epoch execution baseline
```

`WorkflowVersion` pin `SemanticIRDigest + LockGraphDigest + PlanCoreDigest + exported contract + compiler/runtime semantics`。`EnvironmentContract` 只描述 execution/security/trust/residency/durability/protocol 能力，不包含 current capacity。`BindingSnapshot` 是 closed/minimal reachable binding closure，包含 component/resource/model/identity/remote/subflow bindings；不允许 raw secret、WorkerId、AttemptLease、current endpoint instance 或任意 `Map<String,Any>`。

`DeploymentRevision` 引用 exact `WorkflowVersion`、`PlanCore`、`BindingSnapshot`、`EnvironmentContract`、RunParameter constraints、budget/durability/governance profiles。Mutable `Deployment.active_revision_ref` 只影响 future StartRun；existing Run 永远不跟随该 pointer。Rollback 通过创建新 Revision 指向旧 WorkflowVersion 完成，而不是修改旧 Revision。

### 24.4 StartRun / RunManifest / ExecutionEpochManifest

StartRun 先把 transport request canonicalize 为 `CanonicalStartRunRequest`，并以 `StartRunIdempotencyKey + StartRequestDigest` 提供 durable idempotency。相同 key+digest 返回同一 RunId；相同 key 不同 digest 必须冲突。

Admission 只解析一次 concrete `DeploymentRevisionRef`，随后所有 input/parameter/security/budget/epoch materialization 都围绕这个 revision 完成，禁止 admission 中途重新读取 mutable active pointer。完整流程：

```text
normalize request
→ idempotency lookup
→ resolve exact DeploymentRevision
→ validate Start eligibility
→ validate input
→ materialize full RunParameterSnapshot
→ validate identity / subject / delegation
→ derive RunAuthorityEnvelope
→ admission policy
→ RunSecurityBaseline
→ RunBudgetBaseline
→ admission quota permit
→ ExecutionEpochManifest(epoch=0)
→ RunManifest
→ RunAdmissionRecord
→ RunStarted(seq=1)
```

RunManifest 实际根对象只保留 `run_id/project_id/workflow_input_ref/initial_epoch_manifest_ref/security_baseline_ref/budget_baseline_ref/durability_profile_ref/durable_semantics_ref`。RunManifest digest包含 RunId，因此相同输入和配置的两个 Run 仍然具有不同 manifest identity。

Migration 不修改 RunManifest，而是同一 Run 新建 `ExecutionEpochManifest(epoch+1)`，并通过 `MigrationApplied` Journal Event 激活新 epoch。existing Run 更换 Plan/WorkflowVersion/Semantic Binding 的唯一合法路径是 Migration；compatible physical late binding 和 secret material rotation不属于 migration，但必须保持 logical identity/contract/authority/trust/residency semantics 不变。

### 24.5 Journal Event Protocol：Canonical Truth

Journal 是 per-Run append-only canonical semantic fact log。每个 Run 只有自己的 contiguous `JournalSeq = 1,2,3...`，不存在 global JournalSeq。核心身份严格分离：

```text
JournalEventId  = committed record identity
JournalEventKey = semantic uniqueness/conflict slot
JournalSeq      = canonical order
```

同一 `EventKey + same payload` 是 duplicate；同一 EventKey 不同 payload 是 conflict，禁止 last-write-wins。

`RunStarted` 固定为 seq1。除它以外，Canonical Writer 必须持有有效 JournalWriterFence。外部服务只提交 `ExternalSemanticCandidate`，Candidate 不得自带 EventId、JournalSeq、canonical LogicalTime 或 committed_at。

Journal v1 核心 body 固定为 22 类：

```text
RunStarted
PauseRequested / ResumeRequested / CancelRequested
InvocationPreparationResolved
TaskAttemptStarted / TaskAttemptOutcome / TaskAttemptLost
EffectReceiptRecorded / EffectReconciliationResolved
SignalAccepted / TimerFired
ObligationResolved
ChildExecutionStarted / ChildExecutionTerminal
SelectCaseChosen / RaceWinnerChosen
IterationGuardDecision
ComputationRetryDecision / AttemptRetryDecision
MigrationApplied
ManualRecoveryResolution
```

明确没有 `WaitMatched`、`HandleCaseChosen`、`TimerScheduled`、`RunSucceeded/Failed/Cancelled` Journal Event；这些由 state + deterministic reducer 推导，避免第二份 truth。

### 24.6 Canonical Acceptance Gate 与 Reducer Turn

Run 只在：

```text
journal_head_seq == processed_journal_seq
```

时接受一个新的 external semantic stimulus。External Event append 后：

```text
head = processed + 1
```

Gate 立即关闭，Run Owner 必须先处理这个 exact stimulus 到 quiescence；因此合法运行时满足 `head - processed ∈ {0,1}`，除同一 Turn 内由 `AppendTurn` 原子追加的 internal decision events。

Internal Decision 只有五类：

```text
SelectCaseChosen
RaceWinnerChosen
IterationGuardDecision
ComputationRetryDecision
AttemptRetryDecision
```

Live 模式中 Reducer independently derives decision 并生成 draft；Replay 模式中 Reducer仍先独立重算，再与 recorded decision 做 exact compare。缺失、多余或不同都 fail closed，形成 `REDUCER_DECISION_*` incident；历史 Decision 不是“告诉新 Reducer 该怎么做”的指令。

`AppendTurn` 是核心 durable transaction，必须原子提交：

```text
internal decision journal events
+ new materialized semantic state pointer
+ processed_journal_seq
+ runtime-command outbox entries
+ TurnCommitDigest
```

同 `TurnId + same TurnCommitDigest` 是 idempotent replay；同 TurnId 不同 digest 是 determinism violation。Reducer本身必须纯：不读 network、registry、policy、budget service、wall clock、random、environment variable、secret service、scheduler 或 worker。

### 24.7 SemanticExecutionState / Structured Control Machine

`SemanticExecutionState` 只保存“未来 Workflow 语义所需、且可由 Journal deterministic replay 重建”的 continuation state，不是 runtime database row dump。必须明确排除：

```text
RunOwnershipLease / JournalWriterFence
AttemptLease / WorkerSession / WorkerHeartbeat
QuotaPermit / ConcurrencyPermit
CredentialLease / raw credentials
Scheduler queue/capacity
Trace / Metrics / Logs / Audit index
Checkpoint publication metadata
```

Computation identity 固定为：

```text
ComputationInstanceId
= derive(RunId, ExecutionEpoch, PlanOpId, OccurrencePath)
```

OccurrencePath 由 structured control topology 和 deterministic ordinal 构成，覆盖 sequence/select/parallel/race/foreach/while/retry/handle/scope/compensation/subflow；不能使用 UUID、timestamp 或 worker identity。

Computation lifecycle 为 `ACTIVE → COMPLETING → TERMINAL`。`COMPLETING` 表示 forward outcome 已经确定，但 owned child/cancellation/recovery 尚未闭合。Structured Concurrency 的硬不变量是：

```text
terminal owner
⇒ no active owned semantic child
```

v1 没有 detach。

### 24.8 13 个 Control Opcode 的 Durable State

`ComputationOpState` 与 Plan Opcode 一一对应：`invoke/await/sequence/select/parallel/race/iterate/retry/timeout/handle/scope/compensation_scope/call_subflow`。关键状态规则：

- `SELECT`、`RACE`、While guard、Retry decision 用 `JournalEventKey` 记录决定性 latch；不用 future EventId。
- `PARALLEL fail-fast` 只停止新工作/请求取消 sibling，parent在 sibling全部 terminal 前保持 `COMPLETING`。
- `RACE` winner 由 Journal order 固定；loser late terminal 不能改 winner。
- `FOREACH` source materialize 一次；fail-fast 后不再启动新 iteration。
- `WHILE` guard 是 pure expression，不保存 durable “evaluating” phase；每轮 guard decision进入Journal。
- `RETRY` 只有 previous occurrence terminal、recovery closed、decision=retry、backoff timer满足后才创建下一 occurrence。
- `TIMEOUT` 的 child/timer winner由 Journal order决定；timer win 后 child cleanup完成前 parent可处于 `COMPLETING`。
- `HANDLE` matcher必须静态closed/disjoint，正常不需要 HandleCaseChosen event；Parent cancellation不作为普通Failure处理。
- `COMPENSATION_SCOPE` 永久保留 forward outcome；compensation success不会把原Failure/Cancelled/TimedOut改为Success。
- `CALL_SUBFLOW` 是 durable child-execution boundary。

### 24.9 Invocation、Attempt、Wait、Timer、Signal 与 Child

`TaskInvocationPhase` 固定为：

```text
preparing
waiting_obligations
ready
attempting
retry_wait
recovery_blocked
terminal
```

Attempt ordinals必须 contiguous，同一 Invocation 最多一个 nonterminal business Attempt。`TaskAttemptState` durable semantic phase 为 `schedule_requested/started/cancel_requested/terminal`，只记录 canonical accepted `started_execution_generation`；pre-start scheduler generation、AttemptLease、Worker/Placement都属于外部 side state。

`WaitState` 在 durable state 中出现即表示已经 ARMED，不再保留无意义 `CREATED` durable phase。Early Signal可先进入按 JournalSeq 排序的 mailbox；AWAIT进入时如果已有匹配Signal，可以直接消费，不创建Wait/ArmWait command。`CancelWait` 不删除Signal。

Timer state为 `active/fired/closed`。Timer在Reducer emit `ScheduleTimer` 时即进入 active；Command ACK不是Semantic Fact。晚到 `TimerFired` 是否有语义由 Canonical Acceptance Gate + current state 决定。

ChildExecution为 `start_requested/started/cancel_requested/terminal`；StartChild ACK只表示外部Handler已接受，只有 `ChildExecutionStarted` Journal fact 才进入 started。

### 24.10 Runtime Command / External Handler Protocol

Reducer只能产生11种Kernel Runtime Command：

```text
ResolveInvocationPreparation
ScheduleTaskAttempt
CancelTaskAttempt
ScheduleTimer
CancelTimer
ArmWait
CancelWait
StartChildExecution
CancelChildExecution
EvaluateRuntimeObligation
CancelRuntimeObligation
```

不存在 `RunTask/RunAgent/CallModel/CallTool/CallHTTP/CreateInteraction/ReconcileEffect/WriteJournal/MutateState` 之类逃生口。

Command identity：

```text
CommandId
= derive(RunId, ExecutionEpoch, CommandSlotKey)
```

故意不包含 TurnId、JournalSeq、Region、Dispatcher、DeliveryId 或 wall clock。Same CommandId+same payload 是 duplicate；same CommandId+different payload 是 `RUNTIME_COMMAND_CONFLICT`。

Command object 必须 durable-before-AppendTurn；Dispatcher只能消费 durable Outbox，不能watch SemanticState后自行发明Command。Delivery是纯transport：one CommandId → many DeliveryIds。Physical destination、sent_at、dispatch attempt、DispatchFence只存在于 Delivery Envelope，不进入Command identity。

### 24.11 Handler durable acceptance 与 cancel tombstone

Handler成功ACK只有三种：

```text
accepted
duplicate
already_satisfied
```

没有 `completed/success/failed`，因为 ACK 只证明 Handler 已经把 Command 与处理责任持久化并可在自身 crash 后恢复。Handler必须 durable accept before ACK；短TTL内存dedupe不能作为正确性机制。

首次 acceptance 必须验证当前 `CommandDispatchFence`。如果同一Command之前已经durably accepted，之后stale dispatcher送来的 duplicate delivery仍可返回 duplicate ACK，因为不会创建新工作。

所有 Start/Cancel command pair 使用外部 Handler target FSM：

```text
ABSENT
ACTIVE
CANCEL_TOMBSTONE
TERMINAL
```

Cancel-before-start 必须建立 durable tombstone；late Start after tombstone不能复活 target。Pairs为：Attempt、Timer、Wait、Child、RuntimeObligation。`ResolveInvocationPreparation`没有cancel pair，旧generation通过PreparationGeneration与state validation自然stale。

Handler永远不能直接写Journal或修改SemanticExecutionState；结果必须：

```text
immutable evidence
→ durable ExternalSemanticCandidate
→ PendingSemanticFact Store
→ Run Owner canonical acceptance
```

### 24.12 Worker / Component Started Barrier

Worker是Attempt executor，不拥有Run。ScheduleTaskAttempt handler只创建/recover `SchedulingTicket`；Scheduler选择placement，AttemptController独占 pre-start `ExecutionGeneration++` 与 AttemptLease issuance。

Effect-capable component code的严格边界：

```text
Worker receives envelope
→ validates contract / lease / resources
→ emits TaskAttemptStarted candidate
→ canonical Journal accepts Started
→ StartPermit
→ only then execute effect-capable component code
```

Started之前Worker丢失，可以同一Attempt新generation重新placement；Started之后不能透明重跑同一Attempt，必须形成 `TaskAttemptLost`，由Reducer在recovery closed之后决定是否创建新AttemptId。Heartbeat/Progress/Stream是operational/telemetry，不是Workflow terminal fact。

### 24.13 Effect Intent / Resource Adapter Protocol

Worker不再接收预计算的通用 `effect_grant_set`。每一个具体资源操作都必须形成exact `EffectIntent`：

```text
Component
→ EffectCallRequest
→ durable EffectIntent
→ Authority / Policy / InfoFlow / Obligation / Budget checks
→ exact EffectGrant
→ Resource Adapter
→ Provider
→ durable EffectReceipt
→ ExternalSemanticCandidate
→ Journal
```

ResourceHandle只包含 logical resource identity、resource type、adapter contract、capability ceiling、infoflow/residency与attenuation chain，不包含raw credential。Grant必须bind完整Intent digest，防止参数TOCTOU；Credential由Broker/Adapter在dispatch附近取得。

Provider调用前必须先有durable `ProviderDispatchRecord`。Crash gap只能依赖 provider idempotency、reconcile 或 UNKNOWN；禁止 blind retry。EffectReceipt immutable，记录 intent/resource/adapter/provider idempotency key/observed state/assurance/fencing/evidence/recovery capsule/result。`NOT_APPLIED/APPLIED/PARTIAL/UNKNOWN`是单条receipt的现实观察；`CONFLICTING_EVIDENCE`只在Reducer aggregate中出现。

### 24.14 Value / Artifact / Provenance

Workflow semantic Value immutable。核心引用包含：

```text
value_id
type_digest
content_digest
typed_content_digest
value_semantic_digest
```

`content_digest`只对canonical typed content bytes做hash；`typed_content_digest`加入Type；`value_semantic_digest`再加入provenance/security labels/taints。相同bytes/type但不同lineage得到不同semantic Value identity。

Artifact与“大Value”分开：Artifact是immutable logical bytes对象，内容digest基于重构后的logical bytes，chunking是物理实现；semantic digest加入artifact type/media/provenance/labels/taints。Workspace/Memory仍是mutable Resource，snapshot才是Artifact。

Provenance是content-addressed DAG，覆盖 workflow input、task output、signal payload、subflow output、expression derivation、resource read、artifact import、fork import、declassification、endorsement。Instruction authority按 `none < external_data < delegated_agent < user < workflow < platform` 管理；authentic source不等于instruction authority。Model output默认继承所有可影响输入taint并增加 `model_generated`。

### 24.15 Failure / Incident / Operational Condition

v1将运行异常拆成六个plane：

```text
Semantic Failure
Control Termination
Recovery State
Operational Condition
Runtime Incident
Platform Degradation
```

禁止一个万能 `ErrorState`。`FailureValue`属于Workflow typed ABI；Cancelled/TimedOut是control outcome，不伪装为Failure；`TaskAttemptLost`是基础设施事实，不自动成为 `Failure<WorkerLost>`；`RecoveryBlocked`不是Workflow Failure，也不能被普通 HANDLE 捕获。

Runtime Incident用于Journal mismatch、durability violation、protocol violation、security/compatibility/storage/invariant fault，可阻塞Run但不伪造业务Failure。`WAITING_QUOTA/WAITING_CAPACITY/NO_MATCHING_WORKER/...`属于OperationalCondition，不进入Journal。API/admission错误同样与Workflow Failure分开。

### 24.16 Configuration / Identity / Policy

所有会影响 SemanticTurnTrace 的信息只能走三条路径：

```text
Published semantic baseline
Run/Epoch immutable baseline
Canonical Journal fact
```

不存在第四种ambient config路径。Run Parameter必须在StartRun完整materialize并immutable；operational config、OS env、queue config不能被Workflow expression读取。SecretRef/CredentialBinding属于security plane，不进入普通Workflow Value；同principal/authority的credential material rotation通常非语义变化，principal/authority变化则必须改变binding baseline。

Identity、Authority、Credential、Approval继续严格分离。Principal可以是human/service/workload/external_system/platform_service，但LLM、model、Agent Definition不是Principal。RunSecurityBaseline pin principal/subject/delegation chain/RunAuthorityEnvelope/policy baseline/environment；Scope只能attenuate。Runtime Effect dispatch必须同时满足：

```text
Authority
AND Effect Declaration
AND Resource Capability
AND Policy
AND InfoFlow
AND Obligation
AND Budget
AND Execution Fence
```

Approval只满足某个ProtectedIntent上的Obligation，不自动扩大delegation或RunAuthorityEnvelope。

### 24.17 Budget / Usage / Quota 与 Scheduler

Budget是Run-local semantic consumption envelope；Usage是immutable actual consumption evidence；Quota是live cross-run governance。三者拥有独立authoritative ledger，不进入SemanticStateDigest。Forward与Recovery budget分开；UNKNOWN effect occurrence reservation持续held，直到reconcile证明NOT_APPLIED或确认实际发生。Compensation不退款forward occurrence。

Scheduler只决定WHEN/WHERE，不决定Workflow branch/retry/loop。`ScheduleTaskAttempt` handler创建idempotent SchedulingTicket；Scheduler执行Governance→Fairness→Placement→Capacity/Permits→Lease；AttemptController做最终CAS和generation fencing。Priority不能绕过quota/residency/trust；fairness与quota分开。pre-start worker loss可同Attempt新generation redispatch；post-start worker loss必须形成TaskAttemptLost。

### 24.18 Multi-region Ownership / Journal Writer Authority

多Region可以跨Run active-active，但单个Run v1明确禁止active-active canonical multiwriter。每个Run在任一commit instant最多一个logical canonical writer。必须区分：

```text
ExecutionEpoch              # semantic migration
RunOwnerGeneration          # canonical interpreter ownership transfer
AttemptExecutionGeneration  # pre-start physical dispatch generation
CommandDispatchGeneration   # outbox dispatcher ownership generation
```

`RunOwnershipRecord` 是 correctness-critical coordination state，保存 `last_owner_generation` 与当前 holder；Acquire 必须线性化，新的 owner generation 单调递增且永不复用。`RunOwnershipLease` 只是临时协调authority，Canonical Storage 的最终写Authority是 `JournalWriterFence`。Ownership transition 与 Journal/AppendTurn write 必须形成单一线性顺序：旧write要么在takeover前commit并被新owner看到，要么在takeover后以 `STALE_OWNER` 被拒绝，不允许第三种split-brain结果。

Run Owner只拥有 candidate canonical acceptance、Reducer、internal decision、RuntimeCommand creation 与 `AppendTurn` 权限；Worker/Scheduler/Effect Gateway只产生candidate/evidence。正式取消额外 `CanonicalAcceptanceLease` 概念，Canonical Gate仅由 `valid JournalWriterFence + head==processed + source/state/EventKey validation` 构成。

`RunStarted(seq=1)` 是唯一pre-owner canonical event；seq>=2以及 `processed_seq/state pointer/outbox` 的AppendTurn更新都必须受同一writer fence保护。Owner local memory永远不是truth：failover后新Owner严格从 `RunManifest + EpochManifest + Plan + Checkpoint + Canonical Journal + Outbox` 恢复。Dormant/waiting/terminal Run不需要永久占有owner；Candidate应先durable ingress，再best-effort routing/wakeup。Owner failover不是Journal Event、不改变ExecutionEpoch，也不改变Attempt/Command generation。

### 24.19 Checkpoint / Recovery Acceleration

权威优先级固定为：

```text
RunManifest / Epoch baseline
+ PlanCore
+ Canonical Journal
+ Normative Reducer
>
Checkpoint
```

Checkpoint只在committed turn boundary创建，可以异步发布。`CheckpointStateSnapshot`包含Run/Epoch/processed seq/state schema/SemanticExecutionState/semantic digest/durable refs；不包含RunOwnerLease、WriterFence、AttemptLease、Capacity/Quota/Credential/Worker session/raw credential。

Checkpoint必须绑定Journal anchor、Plan/Runtime profile与deterministic strong object closure。Restore总是先以Canonical Journal为authority，选择最高可验证K≤N checkpoint，失败则回退旧checkpoint或RunStarted。Unknown checkpoint version可以cache miss/fallback；unknown Journal event必须fail closed。

v1禁止ordinary checkpoint授权 destructive Journal prefix deletion，也禁止“checkpoint becomes new semantic genesis”。Physical compaction/archive可以做，但archive仍然是canonical history。Future若要journal squashing，必须设计独立Certified Replay Snapshot/Cut协议，不能复用普通Checkpoint。

### 24.20 Audit / Security Evidence / Compliance

Audit 被正式定义为独立的 **Security Evidence Plane**，而不是第二份 Workflow Journal：

```text
Canonical Journal     = Workflow semantic truth
Primary Security Evidence = authorization / execution-security truth
Audit                 = security/compliance history
Trace                 = observability
Incident              = runtime/platform anomaly truth
Provenance            = data lineage
Usage Ledger          = accounting truth
```

Audit 的核心模型采用 `Primary Security Evidence + Tamper-evident Audit Projection`。EffectIntent、PolicyDecision、DelegationGrant、ObligationEvidence、EffectGrant、ProviderDispatchRecord、EffectReceipt、RunAdmissionRecord 等已经是对应动作的Primary Evidence，Audit只引用它们形成可查询的Security Narrative，不能替代这些对象成为第二份authority。

Audit Record按security boundary/category/retention class进入独立 `AuditStream`，只有per-stream `AuditSeq`，没有global audit total order；`AuditSeq`与`JournalSeq`完全不同。Record使用 `event core + envelope + previous_record_digest` 构成hash chain，并可进一步封装为sealed segment：连续seq range、Merkle root、previous segment、signature/optional external anchor。Tamper-evident不等于tamper-proof，但任意未授权修改、删除、重排都必须可检测。

必须区分两类写路径：已有Primary Security Record的动作可采用 `primary_evidence_sufficient`，Audit projection暂时失败时允许后续repair/backfill；没有其它等价Primary Evidence的高权限人工动作（break-glass、protected debugger/recovery capsule访问、privileged export等）可要求 `pre_action_durable`，Audit Recorder不可用时必须fail closed operationally。`UNAVAILABLE != DENIED` 继续成立。

Audit append-only；纠正通过新 `AuditCorrectionRecord`，Legal Hold通过独立authority object，不修改历史Record。Audit search index只是projection，可重建。最重要的硬边界：**Audit永远不能成为Reducer的业务输入，也不能被用来替代PolicyDecision、Approval Evidence、DelegationGrant或EffectGrant。**

### 24.21 13 Opcode Normative Semantics Sheets

13个Kernel Opcode现在使用统一Normative Sheet：`Static Inputs / Durable State Shape / Entry Rule / Deterministic Microsteps / Journal Dependencies / Internal Decisions / Runtime Commands / Cancellation / Failure-Timeout / Recovery / Terminal / Replay / Invariants / Conformance Vectors`。Control Opcode自身不执行外部I/O；它只变换Semantic State并产生Runtime Command。

Internal Decision集合继续严格只有五种：

```text
INVOKE task retry  → AttemptRetryDecision
SELECT             → SelectCaseChosen
RACE               → RaceWinnerChosen
WHILE               → IterationGuardDecision
RETRY               → ComputationRetryDecision
```

TIMEOUT、PARALLEL fail-fast、HANDLE matcher、Scope closure等都必须从Plan/State/Canonical Journal order直接重建，不再增加隐藏Decision Event。

各Opcode的关键冻结语义包括：`AWAIT`先查early-signal mailbox、没有隐式timeout；`SEQUENCE`任意时刻至多一个active child；`PARALLEL/ALL`用child ordinal做deterministic aggregate，`FAIL_FAST`由首个canonical non-success latch并取消剩余；`RACE/FIRST_TERMINAL` winner由canonical terminal order决定、loser cancellation不抹除effects；`FOREACH` source只materialize一次、result按ordinal；`WHILE` guard pure并记录decision；`RETRY`在上一occurrence terminal且mandatory recovery closed后才能产生下一occurrence；`TIMEOUT` child-vs-timer winner不可变但parent可能保持COMPLETING；`HANDLE` matcher静态closed/disjoint，Cancelled默认不可捕获；`SCOPE` enforcing structured ownership，无detach；`COMPENSATION_SCOPE`按APPLIED effect canonical commit order逆序恢复，UNKNOWN/PARTIAL先reconcile，最终保留原forward outcome；`CALL_SUBFLOW`是durable child execution boundary，ACK不等于Started/Terminal。

所有Control Computation统一遵守：`forward outcome latched != lifecycle terminal`。只有forward outcome已确定且owned children、cancellation、mandatory recovery全部closed，才能Terminal。

### 24.22 Conformance Corpus / Reference Interpreter Oracle

Conformance正式分三类：

```text
EXACT       = canonical bytes / digest / IDs / Journal / Reducer / State / Command / Replay
INVARIANT   = Scheduler / placement / ownership / distributed safety 等多轨迹领域
STATISTICAL = Agent/LLM质量与评测
```

Kernel correctness不允许“近似正确”。Mandatory Golden Cases必须冻结输入、expected ACE1 bytes、digest、decision、command和state digest；CI不能实时调用Reference Interpreter来生成expected，否则Oracle bug会静默改写“规范”。Reference Interpreter的Authority低于Normative Spec与Frozen Corpus：出现冲突时修Reference Interpreter，而不是迁就实现。

Reference Interpreter应故意保持小：single-process、single-threaded、virtual clock、fake task/effect/timer/signal/child runtime、no network；核心semantic transition代码不应直接复用production reducer。Conformance Corpus必须覆盖 ACE1/AFIDL、IR/Plan/Resolver、Run Admission、Journal acceptance、AppendTurn、SemanticExecutionState、13 Opcode、Runtime Command、Started Barrier、Effect crash gap/UNKNOWN/reconcile、Authority/InfoFlow、Budget/Scheduler、Ownership fencing、Checkpoint replay equivalence、Audit integrity、Cross-language serialization 与 Local-vs-Distributed equivalence。

Durable协议必须系统做Crash Matrix：before write / during write / after write before ACK / after ACK，以及request lost/response lost/duplicate/reordered/start-cancel/stale fence/partition。v1真正的Backward Runtime Compatibility定义为：新binary若要恢复pin在旧semantic profile上的Run，必须继续通过该profile的Mandatory Corpus。

### 24.23 Specification Packaging / Versioning / Release Artifact Layout

Published Spec Release本身必须immutable/content-addressed。最终规范包区分：

```text
normative_source
normative_conformance
derived_machine_projection
reference_implementation
informative
```

AFIDL管machine shape，Canonical Encoding管canonical bytes/hash/ID，Semantic Sheets管state transition与authority/recovery，Closed Tables管有限registry，Frozen Corpus管具体machine-checkable expectations；若这些Normative artifacts真正矛盾，应视为 `SPECIFICATION_RELEASE_INCONSISTENCY`，不能用“某个文件优先”掩盖。

版本正式拆成四条线：

```text
Specification Major      # semantic protocol family，例如 agentflow.journal-event@1
Schema Version            # machine representation revision
Specification Release     # 整个协议包 SemVer，例如 1.3.2
Conformance Corpus Version
```

Schema变化产生新的TypeSchemaDigest；兼容通过显式Compatibility Edge与pure deterministic Upcaster表达，而不是假装新旧Schema identity相同。Upcaster不得查询network/clock/registry/current config。未知Journal/command等durable语义必须fail closed；未知Checkpoint可fallback replay。

`SpecificationReleaseManifest` pin全部Normative artifacts、Corpus、Compatibility Manifest与Reference Oracle binding并形成 `SpecificationReleaseDigest`。Final Release发布后不能覆盖同版本内容；同version不同digest直接冲突。Errata分 `editorial / normative_clarification / normative_defect`；只要所谓clarification会改变Golden output、StateDigest、Journal decision或Runtime behavior，就不再是clarification。

### 24.24 SDK / Code Generation / Public Projection Contract

Python/TypeScript/Go/Java/Rust SDK、REST/OpenAPI、gRPC/Protobuf、CLI、Visual Editor、Debugger都只是Projection，不能成为新的Runtime Authority。机器身份仍必须统一走：

```text
AFIDL object
→ ACE1
→ HashV1 / Semantic ID
```

JSON、Protobuf、host-language object bytes都不能直接作为Plan/Value/Manifest等Semantic digest material。语言映射必须保留Int64/Decimal/Timestamp精度、Opaque ID nominal type、Digest domain和closed Variant；unknown durable variant必须fail closed，presentation-only metadata才允许profile定义forward-compatible忽略。

Public mutation必须满足 `Semantics(Public Operation) = Semantics(existing Core Operation)`。因此没有 `patch workflow state / force success / append journal / retry current worker / detach task` 之类便利接口。`CancelRun accepted != Run Cancelled`，`Signal ingress accepted != processed`，`HTTP timeout != Workflow TimedOut`，`Workflow Failure != API error`。

Authoring SDK、Visual Editor、AI Builder只能产生/修改AuthoringIR，再经过同一个Compiler；不能直接构造Executable Plan。Visual position/color/comment等presentation metadata不进入CanonicalIR/Plan digest。Marketplace visual block可以lower为现有primitive/composite，但不能自己引入新Runtime Opcode。Component SDK同样只能暴露窄的typed facets（EffectClient、CancellationView、ProgressSink、ArtifactWriter、UsageReporter等），禁止万能ExecutionContext、Journal/State/raw credential访问。Local Runtime即使单进程也必须逻辑经过 `Journal→Reducer→State→Command→Handler→Candidate→Journal`，不能形成第二套开发模式语义。

### 24.25 Runtime Core 的最终闭环

截至本版，v1完整规范闭环可以写成：

```text
Visual / SDK / CLI / API
        ↓
Authoring / Public Projection
        ↓
Compiler / Resolver
        ↓
Canonical IR / Plan
        ↓
WorkflowVersion / DeploymentRevision
        ↓
StartRun / RunManifest / ExecutionEpoch
        ↓
Canonical Journal
        ↓
Pure Reducer + 13 Opcode Semantics
        ↓
SemanticExecutionState
        ↓
Runtime Commands
        ↓
External Handlers / Scheduler / Worker / Effect Gateway
        ↓
Durable ExternalSemanticCandidate
        ↓
Run Ownership + Canonical Acceptance Gate
        ↓
Canonical Journal
```

最核心机器不变量：

```text
1. Only Canonical Journal facts may change SemanticExecutionState.
2. Only the Reducer interprets Journal facts into Workflow control state.
3. Only the Reducer emits next-step Runtime Commands.
4. External services consume Commands and produce Candidates; they do not mutate Workflow state or Journal directly.
5. Same Plan + pinned baselines + Journal + RuntimeSemantics => same SemanticExecutionStateDigest.
6. Worker/Lease/Quota/Credential/Owner/Region do not define Workflow semantic identity.
7. UNKNOWN/PARTIAL/CONFLICTING external reality cannot disappear without explicit evidence/recovery.
8. Existing Runs never follow mutable Deployment/latest-registry/current-config pointers.
9. Audit/Trace/Metrics are never hidden semantic inputs.
10. Public/visual/SDK projections cannot invent new Core mutations or authorities.
```

---

## 二十五、v1 Final Normative Freeze Matrix / Implementation Readiness Boundary

这一章是v1的最终边界声明。它不增加任何新机制，而是确定：哪些内容已经可以被实现团队稳定依赖，哪些属于Optional Profile，哪些明确推迟到vNext，以及进入Release Candidate之前还必须物化哪些机器制品。

### 25.1 最终冻结判断

当前可以正式标记为：

```text
Architecture Discovery          CLOSED
Core Architecture               FROZEN
Semantic Model                  FROZEN
Durable Runtime Semantics       FROZEN
Security / Recovery Semantics   FROZEN
Distributed Authority Model     FROZEN
Public Projection Boundary      FROZEN

Normative Source Materialization    READY TO PRODUCE
Reference Oracle                   SPECIFIED / IMPLEMENTATION REQUIRED
Conformance Corpus                 SPECIFIED / FULL MATERIALIZATION REQUIRED
```

当前没有发现需要重新增加Core primitive或新增Authority Plane才能解决的cross-module contradiction。

### 25.2 v1 Mandatory Surface

`runtime-core@1` 的最低闭包包括：ACE1/AFIDL、Canonical IR、Executable Plan、Run Admission/RunManifest/ExecutionEpoch、Canonical Journal、Pure Reducer、SemanticExecutionState、13 Opcode、Runtime Command、Task Attempt/Started Barrier、Signal/Await/Timer/Child、Structured Concurrency、Cancellation/Timeout、Effect Intent/Grant/Receipt、UNKNOWN/Reconciliation、Value/Artifact/Provenance、Failure/Recovery separation、Identity/Delegation/Policy/InfoFlow以及Deterministic Replay。

`durable-runtime@1` 在此基础上还必须支持 Durable Journal、atomic AppendTurn、Command Outbox、Handler durable acceptance、PendingSemanticFact、Crash Recovery、Checkpoint replay equivalence、Effect crash-gap处理、Command idempotency与cancel tombstone。

`distributed-runtime@1` 再增加 Scheduler/Placement、AttemptLease fencing、AttemptExecutionGeneration、CommandDispatchFence、RunOwnershipLease、RunOwnerGeneration、JournalWriterFence、cross-region durable candidate ingress与owner failover recovery。

`secure-runtime@1` 必须完整实现 Principal/Subject/Delegation、RunAuthorityEnvelope、Scope attenuation、Policy/ProtectedIntent/Obligation、ResourceHandle、EffectGrant、Credential Broker boundary、InfoFlow与Security Audit；不能把简单RBAC包装成Secure Runtime兼容。

### 25.3 v1 Optional Surface

以下能力可以作为Optional Profile，不要求每个Core实现都具备，但一旦声明支持就不能自行改变语义：Distributed Scheduler、Multi-region Ownership、Secure Runtime、Audit Compliance、Advanced Effect Recovery、具体Worker transport、MCP/A2A adapter、语言SDK、Debugger/Public Projection、External Audit Anchoring等。

“Optional”表示**可以不实现**，不是“实现方式随意”。具体Provider、数据库、消息系统、UI框架、调度算法和Authoring语言仍属于实现/生态自由，只要满足相应Conformance。

### 25.4 v1 Explicit Deferred Matrix

以下能力明确不属于v1：

| 能力 | v1结论 |
| --- | --- |
| Per-Run active-active canonical multiwriter | `DEFERRED_VNEXT` |
| CRDT Journal merge | `DEFERRED_VNEXT` |
| Detached structured-concurrency child | `DEFERRED_VNEXT` |
| Started Task live migration | `DEFERRED_VNEXT` |
| Process/VM snapshot作为Workflow recovery truth | `DEFERRED_VNEXT` |
| Checkpoint作为canonical authority | `DEFERRED_VNEXT` |
| Destructive semantic Journal squashing / new genesis | `DEFERRED_VNEXT` |
| Dynamic Executable Plan graph mutation | `DEFERRED_VNEXT` |
| Runtime `latest` package/registry semantic resolution | `DEFERRED_VNEXT` |
| Marketplace动态新增Kernel Opcode | `DEFERRED_VNEXT` semantic extension mechanism |
| 通用exactly-once provider side effect保证 | v1不承诺 |
| 隐式业务transaction rollback | v1不承诺 |
| Generic mutable workflow StateStore | v1明确禁止，未来也需重新论证 |
| Raw Credential作为Workflow Value | 明确禁止 |
| Generic `Any` semantic escape hatch | 明确禁止 |
| Debugger任意force-edit运行状态 | 明确禁止 |

### 25.5 实现团队现在可以稳定依赖的事实

```text
Kernel primitive set will not expand inside v1.
The 13 runtime opcodes are the complete v1 control kernel.
Runtime Command set remains 11 in v1.
Journal → Reducer is the only SemanticExecutionState mutation path.
Task effect-capable code starts only after the canonical Started Barrier.
External effects go through Effect Intent / Grant / Resource Adapter / Receipt.
UNKNOWN remains first-class and blocks unsafe retry until recovery closes.
Run baseline is immutable except explicit Migration to a new ExecutionEpoch.
Per-Run canonical writer is a single fenced writer.
Checkpoint never becomes semantic authority.
Audit never becomes Workflow semantic input.
Public/SDK/Visual convenience APIs cannot create new semantic mutations.
```

实现团队可以自由选择语言、数据库、physical journal layout、queue、scheduler实现、service/process topology、region topology、cache、checkpoint frequency、worker pool、transport encoding和observability backend；但不能自由改变 Journal ordering、Reducer transition、Opcode semantics、Command/Event identity、Started Barrier、UNKNOWN handling、Cancellation/Recovery、Authority attenuation、Run baseline pinning和writer fencing。

### 25.6 Authority Plane最终归类

任何新增correctness-critical状态都必须能归类到已定义Authority Plane：

```text
Canonical Workflow Truth
    → RunManifest / Plan / Journal / Reducer semantics

Security Authority
    → Delegation / Policy / Grant / ResourceHandle

External Reality Evidence
    → EffectIntent / ProviderDispatch / EffectReceipt

Coordination Authority
    → RunOwnership / AttemptLease / CommandDispatchFence

Accounting Authority
    → Budget / Usage ledgers

Security History
    → Audit

Data Lineage
    → Provenance
```

如果新数据库表声称“Runtime correctness依赖它”却无法归类，默认应视为可能正在创造未经规范的新Truth，必须先走架构审查。

### 25.7 Final Cross-module Consistency Check

最终一致性检查均已闭合：

```text
Journal vs State
    SemanticExecutionState = materialize(Plan, Baseline, Journal)

State vs Command
    only Reducer transition emits RuntimeCommand

Command vs External Fact
    ACK never changes Semantic State; results return as Candidate→Journal

Worker vs Effect
    no effect-capable code before canonical TaskAttemptStarted

Effect vs Retry
    UNKNOWN/PARTIAL unresolved => no unsafe forward retry

Run Baseline vs Deployment
    existing Run never follows mutable Deployment pointer

Ownership vs Semantics
    owner failover changes RunOwnerGeneration only, not ExecutionEpoch semantics

Checkpoint vs Journal
    checkpoint accelerates replay, never replaces canonical history

Audit vs Workflow
    Audit cannot drive Reducer or replace Primary Security Evidence

SDK/Visual vs Kernel
    every mutation maps to an existing normative Core operation

Agent vs Kernel
    Agent is a Composite; no Agent-specific runtime opcode required
```

### 25.8 Release Candidate Blocking Conditions

进入真正的v1 Release Candidate之前，以下项应视为blocking：

```text
All normative AFIDL sources materialized
All closed semantic tables materialized
All Specification IDs / Rule IDs allocated
No duplicate field_no / case_no
No unresolved cross-reference
ACE1 golden corpus frozen
Core digest / semantic-ID golden corpus frozen
Journal / Reducer / StateDigest golden corpus frozen
13 Opcode corpus frozen
Runtime Command / cancel-tombstone corpus frozen
Effect crash-gap / UNKNOWN / recovery vectors frozen
Ownership fencing vectors frozen
Checkpoint replay-equivalence vectors frozen
Security negative / Audit integrity corpus frozen
SpecificationReleaseManifest generated
CompatibilityManifest / Upcaster Registry generated
Independent Reference Interpreter passes Core corpus
At least two independent ACE1 implementations agree
No unresolved normative inconsistency
```

Mandatory Exact项失败即阻断相应Core/RC。Scheduler fairness等Invariant失败只阻断声明相关Distributed Profile的发布；Agent统计评测不参与Kernel语义合格判定。

### 25.9 最终项目状态

因此现在最准确的项目描述不是“架构仍在设计”，而是：

```text
Architecture                       FROZEN
v1 Semantic Specification          FROZEN
Normative Source Materialization   READY
Reference Interpreter              SPECIFIED / NOT YET IMPLEMENTED
Conformance Corpus                 SPECIFIED / NOT YET FULLY MATERIALIZED
Production Runtime                 OUTSIDE CURRENT SPECIFICATION WORK
```

从这个节点以后，核心工作的性质已经改变：不是继续寻找新的Agent框架抽象，而是把现有规范变成真正可编译、可测试、可发布、可由第三方独立实现的协议制品。

---

## 参考来源与设计脉络

以下资料用于支撑本报告中的参考理念；本报告中的最终抽象与规范是基于这些来源和项目目标进行的架构综合，并非对任一现有框架的直接复制。

- **[R1] OpenCode**：Agent、Tool、Permission、Client/Server 等实现思路。  
  https://opencode.ai/  
  https://github.com/sst/opencode

- **[R2] Temporal Durable Execution**：持久化 Workflow Execution、Task/Worker、History/Recovery。  
  https://docs.temporal.io/temporal

- **[R3] Model Context Protocol / Tasks Extension**：Tool schema、外部能力和异步 Task 边界。  
  https://modelcontextprotocol.io/  
  https://modelcontextprotocol.github.io/ext-tasks/specification/2026-07-28/tasks.html

- **[R4] SPIFFE**：Workload identity、SPIFFE ID、Trust Domain、SVID。  
  https://spiffe.io/docs/latest/spiffe/concepts/  
  https://spiffe.io/docs/latest/spiffe-specs/spiffe-id/

- **[R5] RFC 8693 OAuth 2.0 Token Exchange**：Delegation 与 Impersonation。  
  https://www.rfc-editor.org/info/rfc8693/

- **[R6] Open Policy Agent**：PDP/PEP、结构化 Policy input/decision。  
  https://www.openpolicyagent.org/docs/  
  https://www.openpolicyagent.org/docs/deploy

- **[R7] Kubernetes Scheduling**：filter/score、taint/toleration、affinity、resource fit。  
  https://kubernetes.io/docs/reference/scheduling/config/  
  https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration/

- **[R8] Ray Scheduling / Placement Groups**：resource-aware placement、gang scheduling。  
  https://docs.ray.io/en/latest/ray-core/scheduling/index.html  
  https://docs.ray.io/en/master/ray-core/scheduling/placement-group.html

- **[R9] Protocol Buffers**：长期 wire compatibility、field number 不复用、滚动升级。  
  https://protobuf.dev/best-practices/dos-donts/  
  https://protobuf.dev/programming-guides/editions/

- **[R10] Agent2Agent (A2A) Protocol**：Agent Card、Task、Artifact、Streaming、跨 Agent 协议。  
  https://a2a-protocol.org/latest/specification/

- **[R11] OCI Image/Descriptor Specifications**：content address、descriptor、Merkle DAG、multi-implementation index。  
  https://specs.opencontainers.org/image-spec/  
  https://specs.opencontainers.org/image-spec/descriptor/

- **[R12] Sigstore**：identity-based signing、short-lived certificate、Rekor transparency log。  
  https://docs.sigstore.dev/cosign/signing/overview/

- **[R13] SLSA Provenance**：软件制品构建来源、可验证 build provenance。  
  https://slsa.dev/spec/v1.2/provenance

- **[R14] in-toto**：供应链 Attestation Framework。  
  https://in-toto.io/docs/specs/

- **[R15] OpenTelemetry**：vendor-neutral traces / metrics / logs。  
  https://opentelemetry.io/docs/

- **Agno（原 Phidata）**：Agent/Team/Workflow/Runtime 高层组织。  
  https://docs.agno.com/

- **LangGraph**：graph/state/checkpoint/HITL/durable workflow。  
  https://docs.langchain.com/oss/python/langgraph/overview

- **Blockly / Node-RED**：约束式块编程与可视化 workflow 交互参考。  
  https://developers.google.com/blockly  
  https://nodered.org/

---

## 结论

如果只用一句话概括最终冻结后的架构方向：

> **不要做一个“可拖拽的 Agent 框架”，而要做一门具备显式 Effect、Durability、Authority、Resource、Version、Compiler、Runtime Semantics 与机器级 Conformance 的可视化 Agent Workflow Language/Platform；Agent只是这门语言中最重要的一类高级 Composite。**

原始问题是如何把 Harness、Loop、Agent 等理念抽象成用户可以自由组合的最小组件。最终v1答案已经从“Agent Node / Tool Node / Prompt Node”收敛为：

```text
Semantic Kernel
=
Value
+ Task
+ Await
+ Control
+ Resource

cross-cut by:
Effect
Identity / Authority
Policy
InfoFlow
Budget
Recovery
```

再由13个结构化Control Opcode建立可持久化的Workflow machine。Agent、Planner、Tool-use、Human-in-the-loop、Remote Agent、MCP/A2A等不再各自发展成独立Runtime，而是同一个Semantic Kernel上的Composite或Adapter Projection。

本轮整理后，Canonical Encoding/AFIDL、Semantic IR、Plan/Resolver、WorkflowVersion/Deployment/Run Admission、Journal/Reducer/SemanticExecutionState、Runtime Command/Handler、Worker Started Barrier、Effect/Resource、Value/Provenance、Failure/Recovery、Identity/Policy/InfoFlow、Budget/Scheduler、Multi-region Ownership、Checkpoint、Audit、13 Opcode、Conformance/Reference Oracle、Specification Packaging以及SDK/Visual/Public Projection均已达到v1语义冻结状态。**Architecture Discovery可以正式关闭。**

剩余工作不再是“继续发现核心架构”，而是将冻结内容实际物化为AFIDL、Closed Tables、Rule Registry、Golden Corpus、Reference Interpreter、Compatibility/Upcaster Graph与Content-addressed Specification Release，并用跨语言、Crash/Race/Fencing/Security Conformance证明独立实现具有同一语义。

这也给后续实现划出了稳定边界：生产系统可以自由选择语言、存储、队列、调度器、部署拓扑和UI技术，但不能改变Canonical Journal、Reducer、Opcode、Effect reality、Authority、Recovery与Version Pinning这些机器语义。Scratch式交互、AI自动生成工作流、第三方组件市场、分布式Worker和远程Agent协作都应继续作为这套统一语义的Projection，而不是重新建立第二套执行模型。
