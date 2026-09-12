# 第 3 课实操：十路 RC 的连接、布局与扩容

> **定位**：把一组电阻、电容复用成十路，单独移动第七路电容，再扩成十二路。前置：[连接模型](../cohdl/connections.md)、[subdesign 入门](../cohdl/subdesign.md)。本课使用当前语法，代理已实际运行下列验证；你的预测、操作和复述仍待记录。

本课结束时，应能解释三个问题：为什么十路设计共有三十个元件；为什么移动一颗电容不应改变接线；为什么编译通过仍可能不符合“通道独立”的要求。

## 先看一路电路

下面是教学连接图，说明端点关系，不表示 PCB 铜走线或电流方向。

```mermaid
flowchart LR
    IN[接头 P1 输入] --- R[r：1kohm 电阻]
    R --- OUT[接头 P2 输出]
    OUT --- C[c：100nF 电容]
    C --- GND[接头 P5 地]
```

电阻串在输入与输出之间，电容接在输出与地之间。在理想电压源、输出近似空载的条件下，这是一阶 RC 低通：较快变化的输入分量在输出处衰减更明显。截止频率是逐渐衰减过程中的一个参考点，不是完全阻断的开关。[Analog Devices 的 RC 实验](https://wiki.analog.com/university/courses/electronics/lp_hp_filters)给出 `fc = 1 / (2πRC)`。

代入本课数值，理想计算为 `RC = 100us`、`fc ≈ 1.59kHz`。信号源阻抗、后级负载和元件实际值都会影响这个结果。本课没有连接真实信号源，也未测量频响；这些数值用于认识电路，不是手表选型结论。

每路用一个六针接头提供外部连接，其中 P1/P2/P5 分别用于输入、输出、地，P3/P4/P6 显式 `nc`。这里的 GND 是共同电路参考节点，不表示自动连接大地。十路共享这个参考，输入和输出各自独立。

## 用当前 CoHDL 表达

完整文件在仓库 `docs/proposals/fixtures/m2-programmability/rc-workflow/`。虽然目录归属于 RFC 实验资料，三个 `current-*.cohdl` 文件全部采用已实现语法。`R_1K_F_0603` 与 `C_100n_10V_X7R_0603` 是锁定 passive 包的真实 part；接头来自锁定 connectors 包。

```cohdl
{{#include ../../../docs/proposals/fixtures/m2-programmability/rc-workflow/current-10.cohdl:definition}}
```

`IN`、`OUT`、`GND` 是逻辑端口；真正焊到 PCB 上的是 `r`、`c` 及顶层接头。端口叫 `IN` 并不会自动成为驱动器类型，规则仍依赖实际器件引脚的角色。

顶层的关键片段如下。其余九路连接和共享地都在完整文件里，复制这几行不能代替完整设计。

```cohdl
inst connectors: [SOCKET_2X3_254_SMD; 10]
subdesign channels: [RcChannel; 10]
net IN0: connectors[0].P1, channels[0].IN
net OUT0: connectors[0].P2, channels[0].OUT
nc: connectors[0].P3, connectors[0].P4, connectors[0].P6
```

数组从 0 开始，因此第七路是 `channels[6]`。十路共有十颗电阻、十颗电容、十个接头，子设计容器不另占 BOM 项。每路两个独立信号网，加一条共享地，总计 `2 × 10 + 1 = 21` 个网络分区；“分区”指一组互相电连接的端点，不是铜线段数。

## 在临时副本里运行

先按[课程编译器准备](../appendix/course-compiler.md)创建独立工作树并构建固定版本，然后在同一个终端继续。准备步骤导出 `COHDL_BOOK_REPO`、`COHDL_MAIN`、`COHDL_COMPILER`；本课从复制电路开始。命令如有报错，先处理后再继续。

```sh
export RC_FIXTURE="$COHDL_BOOK_REPO/docs/proposals/fixtures/m2-programmability/rc-workflow"
export RC_LAB="$(mktemp -d)"
mkdir -p "$RC_LAB/src"
cp "$RC_FIXTURE/cohdl.toml" "$RC_FIXTURE/cohdl.lock" "$RC_LAB/"
cp "$RC_FIXTURE/current-10.cohdl" "$RC_LAB/src/main.cohdl"
```

依赖须与随附的 `cohdl.lock` 一致。先运行下面的 check；如提示缺包，在这个练习副本中按诊断执行 `"$COHDL_COMPILER" install "$RC_LAB"`，用 `cmp "$RC_FIXTURE/cohdl.lock" "$RC_LAB/cohdl.lock"` 确认锁未变，再重试。不要用 update 更换教学依赖。

```sh
"$COHDL_COMPILER" check "$RC_LAB" --json
"$COHDL_COMPILER" build "$RC_LAB" --json
mkdir -p "$RC_LAB/snapshots"
cp -R "$RC_LAB/out" "$RC_LAB/snapshots/10"
cp "$RC_LAB/design.lock" "$RC_LAB/cohdl.lock" "$RC_LAB/src/main.cohdl" "$RC_LAB/snapshots/10/"
```

先预测，再查看 `$RC_LAB/out/` 的三个文件：`rc-workflow.net` 是端点连接，`rc-workflow-bom.csv` 是物料，`rc-workflow-layout.json` 是位置。`$RC_LAB/design.lock` 记录稳定实例路径与位号。BOM 按物料合并成三行，每行数量十；三行不等于三个元件。

## 移动第七路电容

内部 `r` 在 `(0, 0)`、`c` 在 `(3, 0)`；顶层把第七路放在 `(58, 15)`，所以默认电容在 `(61, 15)`，单位均为 mm。这是局部坐标加整组位置；本例没有旋转或翻面。

每次构建都会更新同一份 out，因此先保留上面的十路快照，后续再分别保存覆盖版与十二路版。第二份完整源码只额外增加下面这一行顶层布局覆盖：

```cohdl
place channels[6].c at (62mm, 17mm)
```

```sh
cp "$RC_FIXTURE/current-10-override.cohdl" "$RC_LAB/src/main.cohdl"
"$COHDL_COMPILER" build "$RC_LAB" --json
cp -R "$RC_LAB/out" "$RC_LAB/snapshots/10-override"
cp "$RC_LAB/design.lock" "$RC_LAB/cohdl.lock" "$RC_LAB/src/main.cohdl" "$RC_LAB/snapshots/10-override/"
```

保留原 `design.lock`。现在在 layout JSON 中查 `RcWorkflow::channels_6::c`：它应移动到 `(62, 17)`，其他器件保持位置。这个路径标识器件，`C7` 等 PCB 位号由锁分配，操作时以实际锁文件为准。

放置覆盖不改变 `r.B/c.A/OUT` 的连接。它允许调整内部位置；外部接线仍通过端口，不能照搬这条内部路径绕过电气边界。[第 2 课](02-edit-check.md)还会讨论已有铜线的 PCB 为什么需要重新检查；本例没有布线。

## 扩成十二路并核对

第三份源码把两个数组长度改成 12，补第十一、十二路接线、地、nc 和位置，同时保留第七路电容覆盖。当前写法需要这些显式操作，单改数组长度会留下未完成的连接。

```sh
cp "$RC_FIXTURE/current-12.cohdl" "$RC_LAB/src/main.cohdl"
"$COHDL_COMPILER" build "$RC_LAB" --json
cp -R "$RC_LAB/out" "$RC_LAB/snapshots/12"
cp "$RC_LAB/design.lock" "$RC_LAB/cohdl.lock" "$RC_LAB/src/main.cohdl" "$RC_LAB/snapshots/12/"
```

| 阶段 | 真实元件 / 无源件 | 网络分区 | 第七路电容，mm |
| --- | --- | --- | --- |
| 十路 | 30 / 20 | 21 | (61, 15) |
| 十路，单件覆盖 | 30 / 20 | 21 | (62, 17) |
| 十二路，保留覆盖 | 36 / 24 | 25 | (62, 17) |

原十路位号和位置应保留。共享地会增加新端点，所以比较连接时，要把十二路结果限制到原有器件的端点再比较；不能要求整个新网表与旧文件字节相同。**相同输入重复构建**的字节一致性是另一项检查。

用下面的小脚本亲自读布局差异：第一步只列出第七路电容，第二步没有旧器件移动。快照还保留了当时的源码、物料、网表和锁，便于逐项查阅。重复本课请重新创建 RC_LAB，避免把目录再次复制进旧快照。

```sh
python3 - "$RC_LAB" <<'PYCODE'
import json, sys, tomllib
from pathlib import Path
root = Path(sys.argv[1]) / 'snapshots'
def stage(name):
    folder = root / name
    layout = json.loads((folder / 'rc-workflow-layout.json').read_text())
    places = {p['instance']: p['at'] for p in layout['placements']}
    refs = tomllib.loads((folder / 'design.lock').read_text())['designators']
    return places, refs
for old, new in [('10', '10-override'), ('10-override', '12')]:
    before, old_refs = stage(old)
    after, new_refs = stage(new)
    assert all(new_refs[k] == v for k, v in old_refs.items())
    changes = {k: (v, after[k]) for k, v in before.items() if v != after[k]}
    print(old, '→', new, changes)
    expected = {'RcWorkflow::channels_6::c'} if new == '10-override' else set()
    assert set(changes) == expected
PYCODE
```

## 故意出错，观察不同边界

每个实验都从 `current-10.cohdl` 恢复后单独修改，再执行 check；不要把多个错误叠在一起。下表是本次实际输出，不是推测。

| 单独修改 | 实际结果 | 要理解什么 |
| --- | --- | --- |
| 第一处 `(10mm, 15mm)` 改成 `(10V, 15mm)` | E1007 | 坐标要求 Length；读消息确认单位，不能只凭编号猜类别 |
| 定义里的 `net _: GND, c.B` 改成 `net _: GND` | E701 | 十路电容的真实 B 引脚仍需连接 |
| 删除 `net OUT0: connectors[0].P2, channels[0].OUT` | E1302 与 E701 | 子设计外部端口义务与接头物理引脚义务分别存在 |
| 第一条 `place channels[0]` 改为 `place channels[10]` | E202 | 十路数组没有第 11 个元素 |
| 额外添加 `net SHORT: channels[0].OUT, channels[1].OUT` | check 通过，无诊断；网络 21→20 | 类型允许的连接仍可能破坏通道独立性 |

最后一项把两个三端点输出网合成一个六端点网。当前模型的这些引脚没有驱动冲突，编译器也没有收到“输出必须相互隔离”的要求。独立拓扑对照能发现这次变化。恢复方法是删除额外连接；给漏接的电容写 `nc` 虽能满足结构处置，却没有恢复滤波电路。

完整复验命令会自动执行三个正例和五个探针，并检查全部端点、物料、位置、位号与重复构建。失败探针的 check 退出 1 是预期；脚本全部断言符合预期后才退出 0。

```sh
python3 "$RC_FIXTURE/verify.py" \
  --compiler "$COHDL_COMPILER" \
  --compiler-source "$COHDL_MAIN" \
  --record "$RC_LAB/verification.json"
```

## 接下来怎样连接到硬件与语言开发

| 本课已能核对 | 仍需补充的证据 |
| --- | --- |
| 器件和网络符合独立通道模型 | 实际信号源、负载、容差与频响是否满足需求 |
| 元件有坐标、单件覆盖有效 | 封装间距、布线、回流路径、板框与制造检查 |
| 可复用定义和手写重复操作能完成任务 | M2 如何减少重复而保留这些契约；候选尚未实施 |

想参与编译器开发，可沿[诊断指南](../cohdl/diagnostics.md)进入[展开器](https://github.com/conol-ai/cohdl/blob/0e3d770/src/check/expand.rs)和[现有 subdesign 测试](https://github.com/conol-ai/cohdl/blob/0e3d770/tests/subdesign.rs)。本轮另外在本地 `tests/subdesign.rs` 增加数组元素内部布局覆盖回归，核对兄弟器件和网络不变；它尚未进入上述固定提交。

把“我还不懂的硬件事实”和“当前语言表达的具体困难”分别写进[记录](../learning/template.md)。先解释为什么共享地不会自动合并输出，再用自己的话说明 `channels[6].c` 的位置变化。完成这些操作与复述后再更新学习进度。

**版本与验证：** 基线 CoHDL 0.7.0 / main `0e3d770`；本次结果在仓库 `docs/proposals/fixtures/m2-programmability/rc-workflow/book-results-2026-09-10.json`，含编译器源码提交、版本、二进制及输入哈希。见[实际记录](../learning/2026-09-10-book-review-repair.md)。本次未测量硬件、未运行 KiCad 物理 DRC，未执行 Proposed M2。
