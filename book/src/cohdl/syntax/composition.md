# 复用：模块、fn、subdesign

> **定位**：三种“写一次、用多次”的机制各管什么。它们的工作流对比见[模块、fn、subdesign 与数组](../reuse.md)，两路 RC 的完整实验见 [subdesign 专章](../subdesign.md)；本页只给语法和边界。基线：CoHDL 0.7.0 / `0e3d770`。

## 模块：名字从哪里来

包的模块树就是 `src/` 下的文件树，不需要 `mod` 声明。`lib/connectors/src/headers/smd_254.cohdl` 里的 `SOCKET_2X3_254_SMD`，全名是 `connectors::headers::smd_254::SOCKET_2X3_254_SMD`。

```cohdl
use connectors::headers::smd_254::SOCKET_2X3_254_SMD;   // 导入一个名字
inst j: SOCKET_2X3_254_SMD
inst r: passive::R_1K_F_0603                             // 或者写全路径
```

规则：同一个包内所有文件的顶层名字互相可见，不需要 `use`；跨包名字必须写全路径或 `use`；`use` 一次只导入一个名字，没有通配；`pub` 只在跨包时生效，包内不区分。跨包引用非 `pub` 的项是 E209；同一个短名在多个模块路径下声明、直接裸写是 E207；两条 `use` 把不同路径导入成同一个本地名是 E208。依赖来自 `cohdl.toml` 的 `[dependencies]`，版本精确到 `X.Y.Z`，`cohdl.lock` 记录内容哈希，每次 `check`/`build` 先验哈希再解析源码。

**PCB 对应**：模块只组织名字，不产生元件。把电源相关声明放进单独文件是关注分离，不是复用。

## `fn`：调用即展开

```cohdl
{{#include ../../../examples/syntax-tour/src/filter.cohdl:fn}}
```

参数有三种类型：

- `Pin`，一个已存在的引脚，调用时传 `j.P3` 这样的引用。
- 一个受 trait 约束的**实例**，两种写法等价：`led: impl Indicator` 或显式泛型 `fn drive_led<D: Indicator>(led: D, …)`。体内通过 trait 的抽象角色访问它的引脚，`led.Anode` 经 `impl` 映射落到具体器件。
- 单位类型泛型 `fn power_rail<V: Voltage>(vdd: Pin)`，调用时用涡轮鱼 `power_rail::<3.3V>(mcu.VDD)`。

体内可以写 `inst`、`net`、`nc` 和 `fn` 调用；**每次调用都会新增体内的所有实例**，本例每调用一次多一颗电阻。匿名网络 `net _:` 通过传入的引脚与外部合并。调用可以嵌套到任意深度，循环调用是编译错误并报出整个环。展开出的实例路径形如 `SyntaxTour::__fn0_drive_led::r`，同一个 `fn` 从不同调用点展开得到不同路径。

一个当前的实际限制：把数组元素作为实例参数传给 `fn`，例如 `drive_led(leds[0], …)`，在 0.7.0 上报 `E202 unknown instance leds`，无论用 `impl Trait` 还是显式泛型都一样；数组元素的**引脚**作为 `Pin` 参数是可以的。合规记录把“fn 调用参数”列为数组元素合法位置之一，这一处与实现不符。本例因此给状态灯单独声明了标量实例 `led_status`。

`fn` 体内允许 `layout` 约束，但 `place` 会报 E1007，`board_outline` 报 E1006：`fn` 展开后没有稳定路径可供外部放置。

**PCB 对应**：`fn` 适合“去耦电容跨在电源脚上”这种反复出现、不需要单独摆放的小片段。`pub fn` 可以跨包调用，`passive::bulk_10u(...)` 就是依赖包里的函数。它和 `subdesign` 的区别不在包，而在于没有端口、没有保留的具名层级、不能整体摆放或从外部覆盖内部元件的位置。

## `subdesign`：带端口的电路单元

```cohdl
{{#include ../../../examples/syntax-tour/src/filter.cohdl:subdesign}}
```

声明侧：`ports { required IN: Pin … }` 用 RFC-002 的义务语义，端口类型目前只有 `Pin`；体内是普通的 `inst`、`net`、`nc`、`fn` 调用、嵌套 `subdesign` 和一个可选的 `layout`；可以带和 `device` 同样的泛型参数 `subdesign Filter<R: Resistance, C: Capacitance> { … }`。

使用侧，在 `design` 或另一个 `subdesign` 里：

```cohdl
subdesign lpf: RcLowPass {
    IN: j.P1
    OUT: j.P2
    GND: j.P5
}
subdesign phases: [PhaseDriver; 3]     // 数组形式，元素 phases[0] 等
```

`lpf` 是一个保留层级的节点，路径 `SyntaxTour::lpf`，内部实例是 `SyntaxTour::lpf::r`。它本身没有位号、没有 BOM 行、不出现在网表里；网表里只有它内部的真实元件，端口在装配时被剥掉。

边界由错误码守住：

| 错误 | 含义 |
| --- | --- |
| E1301 | 连了一个未声明的端口 |
| E1302 | `required` 端口在使用点没有接到外部 |
| E1303 | 端口类型不是 `Pin`，或把整个 subdesign 当引脚用 |
| E1304 | 直接或间接的递归包含，报出整个环 |
| E1305 | 放置路径 `lpf.c` 里的某一段找不到 |
| E1306 | 对端口写 `nc`；可选端口直接不接即可 |
| E1307 | 在 `fn` 体内使用 subdesign |

外部对内部**只有一种**访问方式：`layout` 里 `place lpf.c at (…)` 覆盖一个内部实例的位置，详见[布局事实](layout-facts.md)。`net` 不能穿过端口引用内部引脚，`spec` 也不能。

**PCB 对应**：`subdesign` 对应原理图里可以整体挪动的功能块，也对应可以作为独立包发布、被别的板引用的子电路。它保留具名层级，本例直接声明的元件有 `lpf::r`、`lpf::c` 这样的路径；路径保持不变时，扩容或重排仍能沿用它们的位号和放置目标，[十路 RC 实操](../../course/03-rc-workflow.md)验证了这种情况。`fn` 生成的对象即使位于 subdesign 内部，路径也仍带调用序号；不能把直接具名实例的稳定性推广给它们。

## 三者怎么选

| 需要 | 用 |
| --- | --- |
| 只是把文件分开 | 模块 |
| 反复出现的小片段，不需要单独摆放 | `fn` |
| 有明确接口、要整体摆放或单独调整内部元件、需要稳定的层级路径 | `subdesign` |
| N 个同类元件逐个接线、逐个放置 | 数组实例 |

`fn` 和 `subdesign` 体内写好的 `net` 会随每次调用、每个实例展开，这是它们的本职。它们都不会**计算**接线：相邻元素的菊花链、按公式算出的坐标，目前都要逐条手写。让编译器替你算的构造是 [M2 提案](../programming-proposal.md)的内容，还没有被接受。
