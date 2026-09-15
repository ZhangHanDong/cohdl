# M2：09-11 最新 Live 评审与减号规则修订

## 评审来源与结论

本次补读的是 Claude Code Live 于 **2026-09-11 10:59:31（北京时间）**给出的 RFC 评审，晚于此前已处理的 08:25 Book 评审。通过本地 Claude 会话记录读取；没有向 Claude 发送消息，也没有将失联的 Live connector 记为已恢复。

评审认为 `lib/` 新旧验证器差异审计的验收条款已经完整，但该审计仍未运行。新增问题是：RFC 允许减法和前缀负号，却没有规定它们如何与既有负数单位字面量共存。这个规范缺口成立。

评审有一个需要校正的源码描述：当前 `n-1` 并不成功产生负数 token。`lex::number` 对裸负数发出 E102 后返回，因此结果只有标识符和 EOF。`n - 1` 与 `-PITCH` 则在减号位置报 E001。

## 实测证据

基线是 main `0e3d7705ab393a5c64c20836540ad1bc2e49898e`。先以 `cargo build --lib --locked --offline` 确认基线库构建成功，再用临时 Rust 程序链接该库，直接调用公开的 `lex::lex`；未修改编译器。完整 21 个输入、token/span、诊断、探针源代码、编译命令和源码哈希保存在 [JSON 记录](evidence/2026-09-11-m2-lexer-baseline.json)。

| 输入 | 当前 lexer 的实际结果 |
| --- | --- |
| `n-1` | E102；负数字符被消费，没有数值 token |
| `n - 1`、`-PITCH` | E001；减号不被支持 |
| `for links: n in -1..N {}` | `in` 是普通 Ident；负下界处 E102 |
| `-1.00mm`、`-40C` | 带符号 UnitValue；原始文本保留 |
| `- 1.00mm`、`-(1.00mm)` | E001；当前没有对应一元表达式支持 |
| `-5V` | E105 |
| `10%`、`10 % 3` | 前者为一个 Tolerance token；后者为三个 token |
| `10%3` | E103；整个 `%3` 后缀不合法 |
| `1mm-1mm` | 两个 Unit token，第二个带负号；词法成功不代表完整源码合法 |

这是对现有 lexer 的测量。没有实现或运行 M2 parser、求值器、新验证器，也没有把片段词法成功记为 `cohdl check` 成功。

## 本次提案修订

英文工作稿与中文全文的 §2、§10 同步修改，修订日期更新为 2026-09-11：

1. 字符串/注释外的 `-` 统一作为独立标点；在允许的表达式语法中，由 parser 区分前缀负号和二元减法。
2. 不采用“前一个 token 是标识符就判二元减法”的全局启发式。循环头的 `in` 本来也是标识符；parser 消费它后，下界才从期待操作数的位置开始。无需符号表，也无需无界前瞻。
3. parser 在字面量/前缀位置组装紧邻的负数单位字面量，保留旧值、完整文本及包含符号的 span。`-1.00mm` 与 `- 1.00mm` 的 AST 和派生值文本区别必须经 fmt 往返保持。
4. 明确 Int MIN 的符号合并时机、旧负裸数的 E102、负 Voltage 的 E105、其他单位诊断及旧字面量消费位置的回归要求。Temperature 算术不在本次新增范围。

这选择了本工作稿中的词法/语法分工；不是修改 Accepted 规范或编译器。A/B、标签和区间拼写的范围决策不变。新增回归条款仍是未来验收要求，不能由上述 21 个旧 lexer 探针替代。

## 独立复核

三位只读审查者分别核对语言语义、源码事实及中英结构，没有发现本轮新增的阻塞问题。事实审查者还在固定基线重放了全部 21 个探针，token、诊断消息、span 与 JSON 记录完全一致。该复核只覆盖本轮减号修订，不是 M2 全部设计的接受结论。

最终机械检查：中英文各 651 行、43 个标题、6 个代码块；标题层级、表格结构、代码块、行内代码/错误码/URL 多重集合一致。`mdbook build book` 成功；`python3 book/tools/check_book.py` 通过 60 个章节、63 个 HTML 页面和 3,092 项本地链接/资源/锚点检查。24 条按日期命名的学习记录各在 SUMMARY 出现一次，新增记录归入“语言设计与 RFC”。`git diff --check` 通过。

## 22:23 全文复审与后续补查

当晚从同一 Claude 本地会话记录读到两条新答复：21:39 的答复认为减号规则已完整，但误称此前规则“原本就有”；**22:23:13 的全文复审**随后更正了归因，确认上午发现的问题由 Codex 的后续修订解决。Latest Live connector 仍失联，本轮未将读取本地记录表述为接口恢复。

Claude 报告全文读取了 651 行英文，核对中英文结构、各节交叉引用，并以真实物料工程验证 `bank.channels[1].c` 的两层内部布局覆盖：目标电容为 (22mm, 17mm)，通道零的默认电容为 (13mm, 15mm)，通道一的电阻为 (18mm, 15mm)。这些是**Claude 本轮报告的实验**；Codex 此次未重跑该嵌套工程。Claude 的结论是“没有新的阻塞项”，可以进入范围决策阶段，并未接受本 RFC。

对该评审留下的两项未验证声明，Codex 补做了独立核对：

| 声明 | 本轮证据 | 结论与边界 |
| --- | --- | --- |
| 未锚定的 subdesign 不产生默认板级放置 | 固定 main `0e3d770` 上执行 `cargo test --manifest-path /private/tmp/cohdl-main-subdesign-ab44257/Cargo.toml --locked --offline --test subdesign unanchored`，两条匹配测试通过；`expand.rs` 仅合成已锚定 owner 的默认位置 | 声明成立；不排除显式内部覆盖，也不意味着 emitter 不会把未放置器件安排到暂存区 |
| API docs 的 inst 列表已区分 subdesign 使用点 | 对当前语法十路 RC 副本实际运行 `cohdl docs`；`/items/0/design/insts` 中的 `channels` 带 `kind: subdesign`、`array: 10`；依赖锁未改变 | 声明成立；当前 schema_version 仍为 1，不代表拟议 M2 文档契约已实现 |

API 输出、源码及编译器哈希见[实际记录](evidence/2026-09-11-m2-docs-kind-check.json)。两条测试分别为 `unanchored_node_contributes_no_placements` 与 `unanchored_outer_override_does_not_shadow_an_anchored_default`。

本轮没有修改中英文 RFC 正文或编译器。下一步仍是 A/B、标签是否必需、是否新增半开 fan-out 的范围决策；统一声明验证器的 `lib/` 新旧差异审计仍是未完成的接受前验收要求。
