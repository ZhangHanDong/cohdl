# 设计与网络：design、inst、net、nc、属性

> **定位**：板级作者写的那一半语言。只有这一层会产生真实元件和连接。前置：[连接模型](../connections.md)。基线：CoHDL 0.7.0 / `0e3d770`；示例是 `book/examples/syntax-tour/src/main.cohdl` 的顶层设计。

```cohdl
{{#include ../../../examples/syntax-tour/src/main.cohdl:design}}
```

## `design`

一个包在 `cohdl.toml` 的 `[design] top` 里指定顶层设计名。`design` 体内允许四类语句：`inst`、`net`、`nc`、`layout { … }`，再加 `subdesign` 使用点和 `fn` 调用。所有 `fn` 展开、所有 `subdesign` 端口合并之后，编译器在这里做一次必需引脚的穷尽检查。

## `inst`：让元件出现

`inst j: SOCKET_2X3_254_SMD` 用 part 名做类型，实例直接绑定物料。`inst c: MLCC<100nF, 16V>[C0402]` 用器件加参数做类型，`build` 时按精确匹配找 part。

数组实例 `inst leds: [LED_RED_0603; 2]` 声明**一个**数组，里面每个元素都是完整的真实实例，内部名是 `leds_0`、`leds_1`，各有自己的位号和连接义务。引用必须带下标 `leds[0]`，裸写 `leds` 是 E211，越界是 E202。长度和下标目前必须是字面量整数。

**PCB 对应**：一条 `inst` 就是一个位号。数组的每个元素独立占 BOM 数量，独立放置。

## `net`：连接

`net V3V3 [3.3V]: j.P4, c_byp.A, r_pwr[0..=1].A`

- 成员是 `实例.引脚`。同一个引脚出现在两条 `net` 里，两条网络合并成一个电气节点，这也是 `fn` 内部网络接到外部的机制。
- 方括号里最多一个注解：一个 `Voltage` 字面量，或者标记 `gnd`。网络的电压和“是地”这两个事实**只**来自注解，从不解析网络名；D003/D004 另外还看连通图和引脚的电气角色。合并的网络注解冲突（不同电压，或 `gnd` 与电压同在）是编译错误。
- `net _: …` 是匿名网络，`fn` 和 `subdesign` 内部的常态；导出名按调用路径生成。
- 范围扇出 `leds[0..=1].Cathode` 和列表扇出 `d[0, 4, 8].Cathode` 只是逐个元素的缩写，把它们放进**同一条**网络。扇出只能出现在 `net` 成员里，`place` 和 fn 参数只接受单个元素。

**PCB 对应**：`net` 就是网表里的一个网络，`build` 后可以在 `out/*.net` 里按名字找到它的所有焊盘端点。注解是给 DRC 的电气事实：`[3.3V]` 让 D001 能和器件的 `voltage_rating` 比较，`[gnd]` 让 D002 知道阳极不该接到这里。

## `nc`：显式不接

`nc: j.P6`。`required` 引脚必须在 `net` 或 `nc` 二选一。`nc` 不是第二种连接机制，列在这里的引脚不属于任何网络，也不会出现在 `.enet` 网表里。

**PCB 对应**：这是“我看过数据手册，这个脚确实可以悬空”的书面决定，编译器记录这个决定，不核实它。对一个漏接的电容脚写 `nc` 能让结构检查通过，但电路已经不是原来的电路了，见[诊断指南](../diagnostics.md)。

## 属性：给工具的事实

属性写在声明前一行，格式固定为 `#[name(...)]`。它们分三组：合法的属性不改变电气判定，只进不同的产物；`#[designator]` 是例外，它直接决定位号。属性自身的单位和引用照常检查，`#[high_current(500mV)]` 是 E110：

| 属性 | 挂在 | 进哪里 |
| --- | --- | --- |
| `#[designator("J1")]` | `inst` | 位号覆盖，写入 `design.lock`，与自动分配冲突时报错 |
| `#[intent("…")]` | 任意声明，至多一条 | 纯注释，进 API 文档 |
| `#[placement_hint("…")]` | `inst` | `layout.json` |
| `#[doc("路径")]` | 库声明，可多条 | 注册表文档 |
| `#[ground(primary)]`、`#[high_current(500mA)]`、`#[impedance(50ohm, frequency: 1GHz)]` | `net` | Quilter 物理约束 CSV |
| `#[bypass(j.P4, 100nF)]`、`#[crystal_oscillator(u, XIN, XOUT)]`、`#[switching_converter(inductor: l1)]`、`#[bga_fanout]` | `inst` | Quilter 物理约束 CSV |

后两行是 RFC-027 的七个物理属性。参数是有类型的：`high_current` 要 `Current`，`impedance` 要 `Resistance` 和 `Frequency`，写错单位是 E110，结构错误是 E1009。只要设计里有任何一条物理属性，`build` 就会输出一组八个 Quilter CSV 文件，本例的 `out/` 里可以看到。

## 位号从哪里来

`build` 之后打开 `design.lock`：

```toml
"SyntaxTour::__fn0_drive_led::r" = "R1"
"SyntaxTour::leds_0" = "D2"
"SyntaxTour::lpf::c" = "C2"
```

键是实例的层级路径，值是位号。前缀来自器件所实现 trait 的 `designator_prefix`，有多个时取 trait 名字典序最小的那个，都没有时是 `U`。同一路径下次构建保持同一位号；删掉的实例进入 `[tombstones]`，位号不再复用。`fn` 展开出来的实例路径带 `__fn0_` 这样的段，这是编译器保留的命名空间，用户不能以 `__` 开头命名。

**PCB 对应**：`design.lock` 是丝印上的位号与源码之间的契约。稳定的是**路径**：路径不变、沿用 `design.lock`，位号就不变；给实例改名会得到新位号，旧位号留在墓碑表里。`fn` 展开的路径含调用序号，所以交换两条 `fn` 调用的先后，它们内部元件的路径从 `__fn0_` 变成 `__fn1_`，位号跟着变（实测 R1 变成 R5）。本例的顶层具名实例，以及 subdesign 直接声明的具名实例 `lpf::r`、`lpf::c`，路径保持不变。如果 subdesign 内部又调用了 `fn`，该 fn 生成的对象仍带调用序号，不能由外层 subdesign 的具名路径推断其位号稳定。

## 这一层编译器不检查什么

它不知道两条本该独立的网络被你误接到一起了，只要类型允许，编译就通过；[十路 RC 实操](../../course/03-rc-workflow.md)里那个短接探针就是这样通过的。它也不检查网络名是否有意义、电压注解是否符合实际电源。
