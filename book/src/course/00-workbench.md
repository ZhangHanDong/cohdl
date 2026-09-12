# 第 0 课：准备工作台

> **定位**：让 Codex 通过 Konnect 读到你正在查看的 KiCad 工程。前置：已安装 KiCad。教材：教程初版；已通过临时 MCP 连接验证 GUI/IPC 与练习板，Codex 内置连接重新加载待验证。

本课只建立可核对的环境。结束时，你要能指出练习工程的位置、当前打开的板、工具实际连接的版本。

## 工程放在哪里

本次学习使用以下本地目录。其他读者替换成自己的路径即可，后续命令在同一个终端中运行。

```sh
export KONNECT_SRC="/Users/zhangalex/Work/CoHDL/Konnect"
export DEMO_ROOT="/Users/zhangalex/Work/CoHDL/kicad-demos"
export LAB_ROOT="/Users/zhangalex/Work/CoHDL/watch-lab"
```

普通赋值会定义 shell 变量，例如 `cd "$KONNECT_SRC"` 中的路径由当前 shell 展开，因此不加 `export` 也能工作。这里统一使用 `export`，让后续从该终端启动的脚本和工具也能通过环境变量读取这些值。

`export` 不会自动写入终端配置文件，也不会更新已经运行的进程。另开独立终端或已启动的 Codex 会话时，不应假定它已获得这些变量；按需重新执行上面的三行。相关问题记录见 [变量与 export](../learning/2026-09-07-shell-variables.md)。

完整 demo 留作参照，操作发生在练习副本。复制整个项目可以保留 `.kicad_pro`、原理图、PCB 和本地库。

```sh
mkdir -p "$LAB_ROOT/lessons"
test ! -e "$LAB_ROOT/lessons/01-sonde" && cp -R "$DEMO_ROOT/sonde xilinx" "$LAB_ROOT/lessons/01-sonde"
```

目标目录已存在时，第二条命令不会复制；继续用现有练习副本，或换一个新的课次目录。用 KiCad 项目管理器打开副本中的 `sonde xilinx.kicad_pro`，再打开 PCB 编辑器。

## 接通 Konnect

先查看实际安装状态。当前资料基线见 [资料页](../appendix/sources.md)；课程记录中的历史状态不保证你的机器此刻相同。

已有可用 Konnect 时直接检查它。源码构建路径如下，需要 Rust、`protoc` 和 `cmake`：

```sh
cd "$KONNECT_SRC"
cargo build --release -p konnect
./target/release/konnect init --client codex
./target/release/konnect status --client codex
codex mcp add konnect -- "$KONNECT_SRC/target/release/konnect" --client codex
```

`init --client codex` 安装 Konnect 提供的共享 skills，注册 MCP 则让客户端启动服务器。两者是两个步骤。模型在 Codex 中选择，`--client codex` 不选择 GPT-6。注册后按客户端提示重新加载 MCP 或开启能看到新工具的会话。

在 macOS，Konnect 的用户级配置文件位于 `~/Library/Application Support/konnect/config.toml`。这是 TOML 文件，不是 shell 脚本，下面的两项不加 `export`，也不要直接作为终端命令执行。

在终端创建目录并打开编辑器；文件不存在时，保存会创建它：

```sh
mkdir -p "$HOME/Library/Application Support/konnect"
nano "$HOME/Library/Application Support/konnect/config.toml"
```

把以下内容放在文件顶层，即任何 `[section]` 表之前。已有同名键时修改原行，不要重复追加；其他配置保留：

```toml
kicad_cli = "/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli"
kicad_binary = "/Applications/KiCad/KiCad.app/Contents/MacOS/kicad"
```

在 nano 中按 `Ctrl+O`、回车保存，再按 `Ctrl+X` 退出。`kicad_cli` 用于命令行导出和检查；`kicad_binary` 用于启动 KiCad 界面程序。这里使用绝对路径，让后台 MCP 进程也能找到程序。

修改后需要重启或重新加载 Konnect MCP 进程。Konnect 也可能优先读取工作目录或程序旁的配置，启动参数还可指定配置文件；若路径未生效，核对实际启动配置，并用 `get_installation_info` 确认运行中的工具路径。

本次实际配置状态见 [创建 Konnect 配置](../learning/2026-09-07-konnect-config.md)。它记录文件已创建；运行中的 MCP/IPC 是否可用仍由下面的操作验证。

如果 `load_toolset` 返回成功，`get_active_toolsets` 也列出需要的工具集，但客户端仍不能调用新增工具，先检查客户端是否刷新了工具列表。2026-09-07 的 Codex 会话实际遇到了这种情况。Konnect 0.11.0 提供启动时加载全部工具集的兼容选项：

```toml
eager_toolsets = true
```

需要时把它加入同一配置文件的顶层，保留其他键，再重启 Konnect MCP 进程并让客户端重新发现工具。它会增加初始工具上下文；正常支持动态刷新的客户端可保持默认关闭。`auto_load_toolsets` 不能替代它：客户端根本看不到工具时，无法发出触发自动加载的调用。本次后续已应用该选项，新启动的临时 MCP 会话首次 `tools/list` 返回了完整工具集；Codex 内置连接的重新加载仍待验证，见 [首次实时读板](../learning/2026-09-07-first-live-board.md)。

在 KiCad 的 Preferences / Plugins 中启用 KiCad API。若需要显式 `ipc_address`，使用本机实际检测到的端点；不要直接照抄其他机器的 socket 路径。仅能读取磁盘文件，还不能证明实时 IPC 已连通。

AI 可以通过可用的桌面操作工具完成打开工程、启动 PCB 编辑器、启用 API 和查找器件。若需要操作这些步骤，可以直接提出完整请求；每一步仍应核对实际窗口。Konnect 的 `open_project` 是 IPC 核对工具，桌面工具负责进入界面，二者用途不同。

2026-09-07 本机启用 API 后实际出现 `/tmp/kicad/api.sock`，随后把 `ipc_address = "ipc:///tmp/kicad/api.sock"` 写入已有服务器配置。新的只读 MCP 会话在获得本机 socket 访问权限后确认了目标板。受限执行环境中无法访问进程或 socket 也可能表现为“未运行/IPC 不通”；应根据权限错误申请所需访问，不能据此反复改 KiCad 设置。

## 请 AI 做第一次核对

可以把下面这段作为本课的操作请求：

> 我正在学习 KiCad。先调用 Konnect 的 `get_installation_info`，检查实际版本；通过 `list_toolboxes` 发现并加载需要的工具。检查 KiCad GUI 和 IPC，核对当前打开的工程是否是我的 `watch-lab/lessons/01-sonde` 副本。然后读取板信息、器件清单和网络清单，解释各自代表什么。

实际工具入口包括 `check_kicad_ui`、`open_project`、`get_board_info`、`get_component_list` 和 `get_nets_list`。参数以本次工具 schema 为准。`open_project` 在这个版本主要用于列出/核对已打开的 PCB，不应仅凭名字就把它当成 GUI 启动器。

把原理图窗口、PCB 窗口与工具返回的路径对照起来。若有多个板打开，给后续调用明确的目标路径。

这一步分三层：Codex 能调用 Konnect；Konnect 能通过 IPC 连接 KiCad；IPC 返回的目标板路径与你的练习副本一致。前一层成功不代表后一层成功。项目管理器打开也不代表 PCB 编辑器已经打开板。

`get_installation_info` 的 KiCad 版本来自实际 CLI 探测；`get_project_info` 中的生成器版本来自保存的工程文件。旧 demo 记录 9.0、当前安装返回 10.0.6 可以同时成立。还要留意 Konnect 0.11.0 的 `open_project.kicad_ui_running` 实际使用 IPC ping 结果，不能凭它为 `false` 就断言 GUI 没有运行；用 `check_kicad_ui` 的进程检测或实际窗口补充核对。实测经过见 [重启后的 MCP 与 IPC 核对](../learning/2026-09-07-mcp-ipc-check.md)。

## 常见分岔

| 观察 | 下一步 |
| --- | --- |
| 工具列表没有 Konnect | 检查 MCP 注册和进程启动，重新加载客户端 |
| 工具集已加载，客户端仍没有新增工具 | 检查工具列表刷新；必要时配置 `eager_toolsets` 后重启 |
| 磁盘能读、IPC 不通 | 检查 API 设置、PCB 是否打开、实际端点 |
| 返回的是原始 demo | 打开练习副本，重新核对路径 |
| GUI 修改未出现在 CLI 检查中 | 保存实际 PCB 后重新检查；分清内存状态与磁盘状态 |

## 本课完成条件

记录版本和三个路径：原始 demo、练习副本、当前打开的 PCB。工具确认了目标板及 IPC 状态，你也能在 GUI 找到该板的至少一个元件。按 [模板](../learning/template.md) 保存本课记录，再更新 [进度](../learning/progress.md)。

下一课：[看懂一条连接](01-read-board.md)。安装依据：[Konnect README](https://github.com/mixelpixx/Konnect/blob/1f96aad/README.md)、[工具目录](https://github.com/mixelpixx/Konnect/blob/1f96aad/tool-directory.md)。基线日期：2026-09-07。
