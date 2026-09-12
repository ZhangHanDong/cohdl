# 2026-09-07：创建 Konnect 配置

对应章节：[第 0 课](../course/00-workbench.md)。状态：已创建并核对配置文件；Konnect MCP/IPC 连接尚未验证。

用户在学习过程中引用了 macOS 上的两项 KiCad 路径配置。本次检查确认 `kicad-cli` 与 `kicad` 均存在且可执行，CLI 实际返回版本 10.0.6；用户级 `config.toml` 原先不存在。

核对 Konnect 的配置加载源码后，在用户允许工作区外写入的操作中创建了 `~/Library/Application Support/konnect/config.toml`，仅写入 `kicad_cli` 和 `kicad_binary` 两项。创建采用排他模式，已有文件不会被覆盖；随后回读 TOML 并核对内容与两个可执行文件。

课程补充了目录创建、nano 编辑/保存和配置文件顶层键的说明，并区分 shell 的 `export` 与 TOML 的键值语法。配置修改需要新启动的 Konnect 进程加载，实际加载来源还取决于启动参数和配置搜索顺序。

已写入文件不代表正在运行的 MCP 进程已经读取，也不代表 IPC 已连接。下一步继续第 0 课：核对运行中的 Konnect、启用 KiCad API、打开练习副本并确认目标 PCB。用户掌握情况与 GUI 实操仍待记录。
