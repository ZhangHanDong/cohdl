# syntax-tour 的参考资料

这个练习工程只用于演示 CoHDL 当前语法，`#[doc]` 指向本文件而不是数据手册。

- 红色 0603 芯片 LED 的 MPN `LTST-C193KRKT-5A` 取自仓库 `lib/led/src/` 中已核对的 Lite-On 型号；本工程不重新核对其电气参数。
- 电阻、电容和 2x3 插座来自锁定的 `passive` 与 `connectors` 包。
- `Standoff_M2` 是一个无引脚的机械件，只有一个 2.2mm 非电镀定位孔，没有可采购的 MPN；`primary` 里的 `mpn` 字段是占位说明，不是物料。

本工程没有真实信号源、负载或制造目标，`check`/`build` 通过只说明语法与结构检查通过。
