# 物料与封装：part、pad、footprint

> **定位**：把抽象器件变成可采购、可制造的东西。`check` 不要求这一层，`build` 要求每个实例都绑定到 `part`（否则 E801）。基线：CoHDL 0.7.0 / `0e3d770`。

## `part`：绑定 MPN 和封装

```cohdl
{{#include ../../../examples/syntax-tour/src/led.cohdl:part}}
```

- 冒号后面必须是**完全具体**的器件：每个泛型参数都给了字面量，有变体的器件选定了 `[变体]`。
- 恰好一个 `primary`，零个或多个 `alt`。`primary` 必须有 `mpn` 和 `footprint`；`alt` 必须有 `mpn`；`mfr` 是可选元数据。
- `footprint:` 是**符号引用**，指向一条 `footprint` 声明，写字符串会被专门的解析错误拒绝。

`alt` 的真实写法，来自仓库 [`lib/can/src/sn65hvd230.cohdl`](https://github.com/conol-ai/cohdl/blob/0e3d770/lib/can/src/sn65hvd230.cohdl)：

```cohdl
    alt { mfr: "Texas Instruments", mpn: "SN65HVD230D" }
```

实例绑定 part 有两种方式。按名字：`inst c1: C_100n_10V_X7R_0603`，part 名直接当类型用。按精确匹配：`inst c1: MLCC<100nF, 16V, 10%>[C0402]`，如果作用域里恰有一个 part 的器件和参数完全一致，就绑定它；多个匹配时取名字字典序最小的那个，构建输出会提示。

**PCB 对应**：`part` 就是 BOM 行。`build` 生成的 `*-bom.csv` 按相同 MPN 合并，一行里列出所有位号；`alt` 是可替代料（AVL）。编译器保证“BOM 不撒谎”的方式是拒绝为没有物料的实例产出网表，而不是替你选料。

## `pad`：一个焊盘

```cohdl
{{#include ../../../examples/syntax-tour/src/led.cohdl:pad}}
```

必填四项：`shape`、`size`、`layer` 和 `plating`。`layer` 取 `top_copper`、`bottom_copper`、`through_all`；`plating` 取 `smd` 或 `plated_through_hole`。`size` 的含义由形状决定：

| `shape` | `size` | 几何含义 |
| --- | --- | --- |
| `rect` | `(宽, 高)` | 矩形 |
| `circle` | `(直径)` | 圆形 |
| `oval` | `(宽, 高)` | 长圆形 |
| `annulus` | `(外径, 内径)` | 圆环，要求外径 > 内径 > 0 |

`annulus` 只用于表面铜焊盘：`plating: smd`，`layer` 为 `top_copper` 或 `bottom_copper`。例如 `size: (0.8mm, 0.4mm)` 表示外径 0.8mm、内径 0.4mm 的铜环；它不是宽高分别为这两个数值的图形。仓库 `lib/@contrib/mic` 的麦克风声孔焊盘使用了环形铜面。

通孔焊盘再加 `drill`。`drill: 0.3mm` 是圆孔，`drill: (0.6mm, 1.7mm)` 是长孔（槽），长孔不能放进 `circle` 焊盘，也不能超出焊盘尺寸。

表面焊盘还有一组可选的工艺字段：`corner_radius: 0.2mm` 给矩形倒圆角，`chamfer` 切角，`mask_expansion: 0.05mm` 设定阻焊外扩。钢网开口由 `paste` 单独描述，各种写法的尺寸规则不同：

| `paste` 写法 | 含义与边界 |
| --- | --- |
| 省略 | 开口沿用铜面形状 |
| `none` | 取消钢网开口 |
| `(w, h)` | 居中的矩形开口，宽高不能超过铜面的外包尺寸 |
| `circle(d)` | 居中的圆形开口，直径可以小于或大于铜面的外包尺寸 |
| `segmented_annulus(外径, 内径, 间隙宽度)` | 仅用于 `annulus`，用十字间隙把环形开口分成四个扇区 |

这些语法允许作者记录工艺要求；检查通过不证明所选开口适合实际焊接。本章只展开常用规则；完整边界见已实现的 [provisional 工艺语法](https://github.com/conol-ai/cohdl/blob/0e3d770/docs/provisional-syntax.md) §10。仓库中 `lib/misc` 的测试点使用 `paste: none`，`lib/connectors` 的同轴座使用 `corner_radius`。

焊盘的合法性错误统一是 E805：尺寸非正、槽孔放进圆焊盘、槽孔超出焊盘等，消息会指出具体是哪一项。

**PCB 对应**：`pad` 同时描述铜、阻焊和钢网三层。`smd` 只在一面有铜，`through_all` 贯穿所有层。焊盘定义一次、多个封装引用，改一处所有封装都变，这既是优点也是风险。

## `footprint`：焊盘的位置和其他图形

```cohdl
{{#include ../../../examples/syntax-tour/src/led.cohdl:footprint}}
```

一个封装体内可以出现六种语句：

| 语句 | 作用 | 说明 |
| --- | --- | --- |
| `pad N: 符号 at (x, y) [rotate 角度]` | 放一个焊盘 | `build` 时比较的是**去重后的编号集合**：封装用到的 N 的集合必须等于器件的引脚号集合（E807）。同一个 N 可以出现多次，一个引脚对应多块铜，散热焊盘分段就是这样写的；封装体本身写错是 E806；`rotate` 是 0 到 359 的整数度 |
| `courtyard { shape, at, size }` | 元件装配所需的外框区域，供下游检查占用冲突 | 形状只接受 `rect`、`circle`、`oval`；不接受 `annulus` |
| `silkscreen_ref { at: (x, y) }` | 位号文字的位置 | 只有位置 |
| `silkscreen { … }` | 丝印图形 | 四种图元 `line`、`circle`、`arc`、`polygon`，加两种标记 `pin_1_marker near pad N shape dot\|triangle` 和 `polarity_marker cathode_pin N shape band\|arrow` |
| `mount_hole N: non_plated\|plated [shape: …] at (x, y) diameter D \| size: (w, h)` | 定位孔 | 编号与 `pad` 编号无关，不对应任何引脚 |
| `window { shape, at, size }` | 板上开窗 | 临时语法 §11，仓库 `lib/led/src/sk6812.cohdl` 有真实使用 |

`annulus` 是电气焊盘专用形状，不能用于 `courtyard`、`mount_hole` 或 `window`。

标记是语法糖：`polarity_marker cathode_pin 1 shape band` 在编译期展开成一条真实的线段，位置从 1 号焊盘的**边缘**算起，这样标记不会压在铜上。标记引用的焊盘号必须已经声明，否则报错并列出合法范围。

封装名有一条约定：属于 QFP、QFN、SOIC、SOP、SOT、BGA、CHIP、MELF 这几个族的封装，标识符本身就是 IPC-7351 名（RFC-021）。其他封装不受此限，本例的 `FP_LED_0603_1608Metric` 沿用了仓库 `lib/led` 的命名。

**PCB 对应**：`footprint` 就是 KiCad 里的封装（`.kicad_mod`），`build` 会把每个用到的封装投影成一个文件写到 `out/footprints/`，也会把丝印图元写进 IPC-2581。`courtyard` 对应 KiCad 的 F.CrtYd 层，`mount_hole non_plated` 对应 `np_thru_hole`。编译器不检查焊盘间距、可制造性或标记方向是否真的对着阴极。

## 没有引脚的器件

机械件没有电气引脚，但仍然要出现在 BOM 和板文件里。语言允许 `device` 没有 `pins` 块，封装可以只有定位孔：

```cohdl
{{#include ../../../examples/syntax-tour/src/mech.cohdl:mechanical}}
```

`mpn` 在这里是制造特征的说明，不是可采购物料；这沿用仓库 [`lib/misc`](https://github.com/conol-ai/cohdl/blob/0e3d770/lib/misc/src/misc.cohdl) 的做法。构建后它得到位号 `M1`，在 BOM 里独占一行，在网表里没有任何引脚。

## `#[doc]`

```cohdl
#[doc("docs/README.md")]
```

可以有多条，路径相对于包根目录。编译器从不打开这个文件；注册表的包页面会把它渲染成参考文档。它对判定和产物字节没有影响。
