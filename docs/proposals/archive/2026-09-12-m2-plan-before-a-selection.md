# M2 类型与参数化构造实施计划

> **English revision supersedes this checklist (2026-09-10):** Candidate B is no longer recommended. The English RFC now shares fn/subdesign parameters, indexes, placement ownership and full expansion-graph metering, and proposes uniform declaration validation with an explicit compatibility change. Rebase this plan only after A/B and remaining spelling decisions are resolved; do not execute its earlier direct-local assumptions.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not start compiler implementation from this document alone; it accompanies a Proposed language RFC.

**Goal:** 按中英文 M2 草案交付带类型的结构计算、循环局部器件构造及嵌套参数化复用，并保留已有检查与输出契约。

**Architecture:** 在共享 AST、泛型替换和展开器中扩展能力；值求值没有物理构造效果，fn/inst/net 通过原有流水线贡献真实电路。每次迭代新增词法帧与来源记录，最终引脚义务和残余 DRC 继续处理完整设计。

**Tech Stack:** Rust 单 crate、编译器零新增外部依赖；现有 LSP 依赖例外不变。Registry/Explorer/VS Code 仍是独立工具包。

**状态与基线：** 2026-09-10，实施准备计划，未执行以下任务。源码勘查为文档分支 `9ac0e3c2ec7731eb636cb33b9c9a781e1f144242`。实施时先在隔离分支对选定 main 做差异审查；不能把本文的文件位置或旧快照当成最新规范。最新 main 已于同日拉取至 `0e3d7705ab393a5c64c20836540ad1bc2e49898e`，其中 RFC-032 subdesign 已接受，应作为依据；此前的排除判断已撤回。**本计划待组合修订后更新，不能直接执行：** 需补齐 subdesign 的数量参数/循环、端口义务、布局所属关系、包含图预算遍历及工具表示，再与下文任务逐项对齐。下文文件勘查和步骤仍保留原基线。正式接受、RFC/错误码分配与 note 10 同步尚未发生。

## Global Constraints

- 规范输入是 `docs/proposals/rfc-draft-bounded-compile-time-programming.md` 与完整中文译本；本计划不另增语言语义。实现前按现有治理流程记录接受决定及受影响 Accepted 契约的修订。
- 保留零单位隐式转换、精确 Int/Length 运算、既有四条残余 DRC、物理实例与设计位号义务。
- 100,000 次累计迭代、1,000,000 个生成工作项、64 层活跃循环帧；按 RFC 的全选定设计启用/计数规则执行，不能由主机内存或超时替代。
- 普通电路体循环允许 inst/net/nc/fn/const/for 和放置子集；layout 内不能构造电气对象。design 所属循环局部实例可在可见处放置，fn 内放置限制不变。
- 不在新循环路径上使用字符串宏、第二套泛型解析器、编辑器求值器或不同的索引算术。
- Proposed 文件继续用 `.cohdl.txt`；只有实现及配套工具通过验收后才迁移为可运行教学源。
- 当前文档分支的既有用户改动不得被覆盖。实现分支上的每个阶段可以单独审查，整项功能必须连同检查与工具契约完成后才发布。

## 源码职责与接口

| 当前位置 | 已核对事实 | 实施职责 |
| --- | --- | --- |
| `src/ast.rs` | GenericBound 只有 Unit/Traits；GenericArg 有 Unit/Name/Number；Stmt 没有 const/for | 表达式、结构参数、常量、循环及保留源码跨度的选择器 |
| `src/lex.rs`、`src/parse.rs` | 手写词法/递归下降；数组索引最终是整数值 | 支持表达式及上下文关键字，保留原字面量诊断与 `%` 单位词法 |
| `src/check/generics.rs` | GenericValue 是 Unit/Device，Substitution 为 BTreeMap | 加入 Int，扩展同一个 resolve_generic_args；不伪造 UnitValue |
| `src/check/bodies.rs`、`src/resolve.rs` | 声明期检查与实际展开分别负责；body walker 需要认识新节点 | 新循环作用域中检查可判定的类型/名称错误；保留旧 helper 的兼容边界 |
| `src/check/expand.rs` | walk_body 先实例、后属性、再操作；Scope 同时携带路径/绑定；fn 与匿名网计数器属于 Expander | 词法环境、design/fn 所属关系、迭代局部计数器、预算和物理对象展开 |
| `src/ir.rs`、`src/diag.rs`、`src/pipeline.rs` | 已有平铺 IR、span 与诊断接口 | 持续保留原始位置及展开来源；最终检查共享同一图 |
| `src/fmt.rs`、`src/lsp.rs` | 直接消费编译器语法与检查结果 | 规范化作者表达式，区分符号类型与实际绑定值 |
| `src/emit/docsjson.rs`、`registry/src/worker/apidocs.ts` | 现有 body_summary 是平铺摘要；registry 的普通与大文件验证路径均限制 schema v1 | 按 RFC 文档 v2 契约同步 extractor、上传验证和查看器 |
| `explorer/extractor/src/`、`editors/vscode/` | 独立工具包 | 消费共享展开与诊断，更新语法识别；不建立第二种电气语义 |

建议新增 `src/expr.rs`，只承担共享表达式及类型/值求值；新增 `src/check/elaboration.rs`，承担迭代帧、预算事件及来源上下文。具体 Rust 名称是实现接口草案，调整时须同时修改所有使用方，不影响公开语义。

表达式求值使用同一结果区分“类型已知、值未知”与失败：

```rust
pub enum ExprType {
    Int,
    Unit(crate::units::UnitType),
}
pub enum EvalValue {
    Int(i64),
    Unit(crate::units::UnitValue),
}
pub struct ExprInfo {
    pub ty: ExprType,
    pub known: Option<EvalValue>,
}
```

`Some(ExprInfo { known: None, .. })` 表示合法但尚无具体值，不能等同于错误或数值零。实际 API 接收表达式、当前类型/值/数组长度环境及 Diagnostics；错误返回 None 并记录最小失败 span。单位值可按既有规则转发，但新算术只允许 RFC 列出的 Int/Length 组合。

## 任务依赖

T1 类型/表达式 → T2 泛型与声明验证 → T3 构造/身份/预算 → T4 完整图与输出。

T5 fmt/LSP/Explorer 和 T6 包文档契约依赖前述共享语义，T7 做整体验收。阶段间可以保留尚未对用户开放的内部结构，不可发布只解析、不检查的循环。

### T1：值类型、表达式与源码语法

**文件：** 修改 `src/ast.rs`、`src/lex.rs`、`src/parse.rs`、`src/lib.rs`；新增 `src/expr.rs`、`tests/m2_expressions.rs`。

**输入/输出：** 消费已有 UnitValue/Span；输出带 span 的表达式树和上述 ExprInfo。原 GenericArg::Number 在旧物理量位置仍保留精确错误语义。

- [ ] 先写 `m2_expressions`：MIN/MAX 及邻接越界、一元 MIN、MIN/−1、负数商余数、除零、不精确 Length 除法；以 RFC 表中的具体值与错误码断言，禁止包装或舍入。
- [ ] AST 为 inst 长度、所有 IndexSel 位置、Length 实参、place 坐标/旋转保留表达式；不在 parser 里展开器件。常量和循环保留嵌套 body、标签及绑定变量 span。
- [ ] 词法覆盖 `n-1`、`-1mm`、`10%`、`10 % 3`、`10%3`、`0..10`、`0..=9`、注释，以及现有 const/in 标识符；解析器不得用符号表决定 token。
- [ ] 实现类型已知但值未知的计算；`i / 0` 在类型环境中失败，`1 / i` 在无具体 i 时保留未知。常量依赖图按当前绑定环境记忆化，报告完整环路。
- [ ] 执行 `cargo test --test m2_expressions` 及 `cargo test --test inst_array`，然后审查表达式规范文本与旧字面量拼写保持规则。

### T2：共享泛型替换与声明类型检查

**文件：** 修改 `src/check/generics.rs`、`src/check/bodies.rs`、`src/check/mod.rs`、`src/resolve.rs`；新增 `tests/m2_types.rs`。

**输入/输出：** 消费 T1 表达式信息；为共享 Substitution 增加 Int 值，为 GenericBound/默认值增加明确结构分支；生成可供实际调用绑定的已检查声明。

- [ ] 扩展 resolve_generic_args，保持单位值、Int 值、trait 约束器件类型各自种类；device 上的整数泛型按草案拒绝，不凭解析成功放行。
- [ ] 新循环词法体检查重复/遮蔽 inst、具名单位转发、trait 与 Pin/Instance 种类；未知值不是跳过已知类型错误的理由。不沿调用边递归加强未改动旧 helper 内部。
- [ ] 用以下完整负例作为 `empty_loop_still_rejects_known_unit_mismatch`；同时测试具体调用与未调用定义，预计都包含 E112，不能依赖后续漏接错误作为成功依据。

```cohdl
device SeriesR<R: Resistance> {
    spec { resistance: R }
    pins { required A: 1 [passive], required B: 2 [passive] }
}
fn wrong<C: Capacitance>() {
    for empty: i in 0..0 {
        inst r: SeriesR<C>
    }
}
design Board {}
```

- [ ] 增加跨包 N/R/C 转发、整数默认值、trait 不满足、Pin/Instance 错用、空循环重复 r、常量环路，以及旧未调用 helper 行为对照。
- [ ] 执行 `cargo test --test m2_types`、`cargo test --test modules`、`cargo test --test deps`；检查没有新增依赖解析/联网分支。

### T3：实际构造、局部身份和预算

**文件：** 修改 `src/check/expand.rs`；新增 `src/check/elaboration.rs`、`tests/m2_construction.rs`。

**输入/输出：** 消费声明环境和具体 Substitution；产生已有 IrInstance/连接记录及来源上下文，不产生循环器件。

- [ ] 先写实际构造测试：下面实例数必须是 8，不是 2、4 或 6；每帧有自己的局部数组，所有 A 引脚均显式 nc。

```cohdl
device OnePin { pins { required A: 1 [passive] } }
design B {
    for rows: i in 0..2 {
        inst one: OnePin
        inst many: [OnePin; 3]
        nc: one.A
        for pins: j in 0..many.len {
            nc: many[j].A
        }
    }
}
```

- [ ] 在每帧先收集声明与长度，再实例化、处理属性及操作。父引用绑定到原对象；局部名字只在本帧/后代可见；跨帧访问拒绝。
- [ ] 迭代路径使用标签和实际整数值，局部标量/数组名沿用既有实例构造规则。保存/恢复迭代计数器，不能让新循环消耗父体旧 fn 序号。
- [ ] 用展开前的可达语法判定计量模式。每次实例、调用、连接成员、约束和属性等工作在分配前计数；fan-out 不得先调用当前无界 Vec 展开再检查预算。
- [ ] 检查三个上限的等于/超过边界、空外层与巨大内层、helper 字面量数组、重复成员去重前计数、不同标签重排、范围扩容/缩容及标签改名。
- [ ] 执行 `cargo test --test m2_construction`、`cargo test --test inst_array`、`cargo test --test quilter`。

### T4：最终电路、来源诊断与制造投影

**文件：** 修改 `src/ir.rs`、`src/diag.rs`、`src/pipeline.rs`、展开相关诊断调用点；必要时给 `src/drc.rs` 诊断附加来源但不得改变四规则集合。新增 `tests/m2_graph.rs`、`tests/m2_build.rs`；扩展 `tests/layout.rs`、`tests/json_output.rs`、`tests/error_registry.rs`。

**输入/输出：** 消费 T3 完整对象图；产出既有 verdict/artifacts，新增构造诊断必须含具体调用/迭代信息。

- [ ] 将草案 fixtures 的 LED、独立输入 RC、共享输入嵌套 RC 分别与自己的显式写法比较。按显式对象对应比较规格、连接分区、NC 与放置，不用匿名网名或单独数量证明等价。
- [ ] 重做已记录的错误单位、缺失 c.B、实例传 Pin 探针，并通过真实循环触发；额外短接仍类型正确的通道，必须由拓扑比较发现，不能新增“猜测意图”规则。
- [ ] 以真实已锁定库 part 制作额外 build fixture，验证所有局部元件进入 BOM/锁/适用格式。现有合成接口和空封装只能证明机制，不能替代此门槛。
- [ ] 将 design/fn 所属关系与词法路径分开，支持 design 局部放置；fn 内放置仍 E1007。检查重复放置、rotate/side、现有父数组路径不变。
- [ ] 下游 part/pin/DRC 诊断携带来源；主消息保留区别迭代所需信息，JSON v1 不变，去重键含展开帧。
- [ ] 执行 `cargo test --test m2_graph --test m2_build --test layout --test json_output --test error_registry`，并对本阶段涉及的输出运行其已有测试目标。

### T5：fmt、LSP、Explorer 与编辑器

**文件：** 修改 `src/fmt.rs`、`src/lsp.rs`、`explorer/extractor/src/lib.rs`、`explorer/extractor/src/model.rs`、`editors/vscode/syntaxes/cohdl.tmLanguage.json`；扩展 `tests/fmt.rs`、`tests/lsp.rs`、`explorer/extractor/tests/model.rs`、`editors/vscode/test/grammar.test.mjs`。

**输入/输出：** 消费同一 AST/已检查图/来源信息。只格式化作者表达式，不常量折叠或展开源码。

- [ ] 保留标签、括号和注释；`(n + 1) * 2` 格式化后必须同语义且再次格式化不变，两个条件分别断言。
- [ ] hover 区分 Int 类型、已绑定数值和多调用上下文；未知值不能显示为 0，不能任选一个调用值冒充全局事实。
- [ ] 测试没有 relatedInformation 的客户端及 Explorer 的主消息投影，仍能区分同一源码位置的不同迭代失败。
- [ ] 执行 `cargo test --test fmt --test lsp`、`cargo test --manifest-path explorer/extractor/Cargo.toml`；在 `editors/vscode/` 执行 `npm test` 与 `npm run compile`。

### T6：包 API 文档 v2 的完整消费者链

**文件：** 修改 `src/emit/docsjson.rs`、`docs/apidocs.md`、`registry/src/worker/apidocs.ts`、`registry/src/ui/apidocs-model.ts`、`registry/src/ui/apidocs.tsx`；扩展 `tests/apidocs.rs`、`registry/test/apidocs.test.ts`、`registry/test/apidocs-model.test.ts`。

**输入/输出：** 沿用 RFC 的 `bound: {"const":"Int"}` 与规范化 `body_source`；需要新表示的文档为 schema v2，其余仍 v1。诊断 JSON 版本不受影响。

- [ ] 未实例化 fn 的 N 保持符号；不能把循环报告成零个网络或伪造一个长度。不再输出该 item 不完整的旧摘要。
- [ ] 使用同一个源码 formatter 生成 body_source，viewer 仅转义显示源码，不执行它。
- [ ] 同步普通上传验证和大文件 canonical/流式路径，后者也存在硬编码 v1 前缀与版本校验。测试 v1/v2 接收、错误版本拒绝以及 per-version 原字节存储。
- [ ] sidecar 不进入包 tar/hash identity；publish 的文档上传仍 best-effort。离线 check/build 不读取 registry 的展示结果。
- [ ] 执行 `cargo test --test apidocs`；在 `registry/` 执行 `npm test -- test/apidocs.test.ts test/apidocs-model.test.ts`、`npm run typecheck`。涉及搜索投影时加跑 `test/search.test.ts`。

### T7：完整验收与教学更新

**文件：** 已接受的 note 10/RFC 修订、`docs/compliance-report.md`、`docs/error-codes.md`、M2 fixtures、`book/src/cohdl/programming-proposal.md` 和实际学习记录。

- [ ] 对照草案的每个 acceptance row 关联到一个真正执行的测试名；未覆盖行不能标为完成。
- [ ] 保存旧源/依赖/锁/资源的输出基准；同输入重复构建检查字节一致，编辑保留身份用另一组测试，不能混为一谈。
- [ ] 执行根 `cargo test`、受影响独立工具包测试、`cargo fmt --check`。旧源未使用新构造时 verdict、design.lock、适用输出字节须保持。
- [ ] 只有新语言及工具验收都通过后，将对应教学 `.cohdl.txt` 迁移为可执行例子，并更新实际实验记录；不将检查通过写成制造或实测完成。
- [ ] 执行 `mdbook build book`、`python3 book/tools/check_book.py`，完成一次手写与生成图对照演示后提交完整审查。

## 本次交付与未执行事项

本次已完成源码职责勘查、类型系统关系的中英文说明和任务依赖设计。上面所有复选框均是未来实施任务；没有执行新语法测试，没有改变编译器，也没有把 Proposed 改为 Accepted。实际已运行的旧语法检查仍以 `docs/proposals/fixtures/m2-programmability/rewrite-2026-09-10.json` 为准。
