# 单位与器件：trait、device、impl

> **定位**：库作者写的那一半语言。这些声明定义“有什么器件、它属于哪一类”，本身不产生元件。基线：CoHDL 0.7.0 / `0e3d770`；示例来自 `book/examples/syntax-tour/src/led.cohdl`。

## 单位是类型，不是数字

CoHDL 的每个物理量都带单位后缀，后缀直接决定类型。裸数字和错误单位在哪个位置出现，就得到哪个错误码，消息都会写明“期望什么，实际是什么”。本组示例工程在 0.7.0 上的实测：

| 写法 | 位置 | 错误 |
| --- | --- | --- |
| `spec { forward_current: 20 }` | 规格字段 | E111，`a bare number is never valid where a unit-typed value is expected` |
| `ChipLed<20>` | 泛型实参 | E113，提示 `write the unit (e.g. 20A)` |
| `ChipLed<20V>` | 泛型实参 | E112，`expected Current, found Voltage` |
| `net V3V3 [3.3]` | 网络注解 | E010，解析器直接拒绝，提示 `3.3V` 或 `gnd` |

字段值单位不匹配是 E110，与 E112 同属一类。

| 类型 | 后缀 | 例子 | 可为负 |
| --- | --- | --- | --- |
| `Voltage` | `V` | `3.3V`、`5V` | 否 |
| `Current` | `A` | `20mA`、`500mA` | 否 |
| `Resistance` | `ohm` | `1kohm`、`330ohm` | 否 |
| `Capacitance` | `F` | `100nF`、`10uF` | 否 |
| `Inductance` | `H` | `2.2uH` | 否 |
| `Frequency` | `Hz` | `12MHz` | 否 |
| `Time` | `s` | `10ms` | 否 |
| `Power` | `W` | `250mW` | 否 |
| `Temperature` | `C` | `85C`、`-40C` | 是 |
| `Tolerance` | `%` | `1%`、`10%` | 否 |
| `Length` | `mm` | `0.8mm`、`-1.5mm` | 是 |

两条容易踩的规则：`ohm` 和 `C` 只接受 ASCII，写 `Ω` 或 `°C` 会得到专门的提示；数字和后缀之间不能有空格。单位之间没有算术，`10V + 5A` 不是合法源码。

**PCB 对应**：单位就是元件参数和额定值。编译器用它在两处发力，一是泛型替换时拒绝把 `100nF` 塞进电阻值参数，二是残余 DRC 把网络上的电压注解和器件的 `voltage_rating` 比较。它不做的事：不算功耗、不算分压，不知道 `1kohm` 在你的电路里是否合适。

## `trait`：一类元件的契约

```cohdl
{{#include ../../../examples/syntax-tour/src/led.cohdl:trait}}
```

一个 trait 可以声明三样东西：`pins { … }` 是抽象引脚角色（只有 `required`/`optional`，没有引脚号，也没有电气角色），`spec { … }` 是必须提供的规格字段及其单位类型，`designator_prefix` 是位号前缀。`Indicator: Polarized` 表示子 trait，实现子 trait 的器件必须另外有一条满足父 trait 的 `impl`。

std 只提供七个核心 trait：`TwoTerminal`、`Capacitor`、`Resistor`、`Polarized`、`Diode`、`IC`、`Connector`。它们是每个包的隐式前导，所以上面的 `Polarized` 不需要 `use`。

**PCB 对应**：trait 是“元件类别”。位号前缀 R、C、D、U、J 从这里来；`Polarized` 让 D002 极性检查知道谁有阳极阴极。trait 只描述数据形状，没有行为。

## `device`：引脚表和规格

```cohdl
{{#include ../../../examples/syntax-tour/src/led.cohdl:device}}
```

逐项读：

- `<If: Current = 20mA>` 是**单位类型泛型**，带默认值。另一种泛型是 trait 约束的类型参数 `D: Capacitor`，两种都在 `fn` 一节再讲。
- `variants { L0603, L0805 }` 声明一个封闭的封装变体集合，每个变体**必须**有自己的 `pins[变体]` 块，缺一个就是编译错误。没有变体的器件直接写 `pins { … }`。
- `required Cathode: 1 [passive]`：`required`/`optional` 是连接义务，`1` 是物理引脚号，`[passive]` 是电气角色。角色只有六个值：`input`、`output`、`bidirectional`、`passive`、`power_in`、`power_out`，不能省略。
- 一个引脚名可以对应多个引脚号，写成 `required GND: 2, 3, 4 [power_in]`，这是引脚总线，网表里会展开成多个焊盘。
- 没有 `pins` 块的器件是合法的，见[机械件](parts-footprints.md#没有引脚的器件)。

`required` 的含义在最终装配时检查：每个必需引脚要么出现在某条 `net` 里，要么出现在 `nc` 里，两者都没有是 E701，两者都有是矛盾错误。`optional` 引脚可以完全不提。

**PCB 对应**：这就是数据手册的引脚表。`required` 的准确含义是“必须显式交代连接状态”，接进 `net` 或写进 `nc` 都算交代；`optional` 是“可以不提”。手册上的“必须接”与“可悬空”是作者据此做判断的依据，编译器不核实那个判断。角色对应“驱动端”与“负载端”，D003/D004 只看 `output` 和 `power_out`。编译器不知道引脚的实际电压和电流上限；目前只有器件级的 `voltage_rating` 会被 D001 拿来和网络电压注解比较。电流规格的字段和泛型仍有单位检查，但不会自动计算工作电流或检查电流上限。

## `impl`：把器件归入类别

```cohdl
{{#include ../../../examples/syntax-tour/src/led.cohdl:impl}}
```

`impl Trait for Device` 是独立语句，可以写在任何文件里。检查按名字进行：trait 要求的每个引脚角色和规格字段，都要在器件上找到同名且义务/单位相符的项。名字对不上时，`impl` 体内写映射：

```cohdl
impl TwoTerminal for TantalumCap {
    pins { A: Anode, B: Cathode }
}
```

没有结构化类型：形状碰巧一样的器件不会自动满足 trait，必须有显式 `impl`。两处会因此报错：写 `impl Indicator for ChipLed` 却没有 `impl Polarized for ChipLed`，是声明时的 E302；把一个没有满足约束的器件传给 `fn` 的 trait 约束参数，是调用点的 E403。两条消息都会写明缺哪个 trait、哪个具体类型。

**PCB 对应**：`impl` 是库作者对“这颗器件可以当电容用”的断言。它决定位号前缀、决定 `fn` 的 trait 约束参数能不能接受这颗器件，也决定极性检查是否覆盖它。

## 这一层编译器不检查什么

它不核对引脚号与数据手册是否一致，不核对 `20mA` 是不是这颗 LED 的真实额定值，不知道封装变体的功率差异除非写进 `spec[变体]`。库文件里的 `#[doc("…")]` 只是给读者的路径，编译器从不打开它。
