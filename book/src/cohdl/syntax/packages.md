# 包与注册表：依赖怎么锁、库怎么发

> **定位**：每个真实工程的第一步。`cohdl.toml` 声明依赖，`cohdl.lock` 记录依赖的内容哈希，六个命令行动词负责改它们，registry.cohdl.org 负责分发。规范来自 RFC-029、RFC-030；基线 CoHDL 0.7.0 / `0e3d770`。本页所有报错都是在 `book/examples/syntax-tour/` 的临时副本上实际触发的。

## 一个工程从 `cohdl.toml` 开始

```toml
[package]
name = "syntax-tour"
version = "0.1.0"

[design]
top = "SyntaxTour"

[dependencies]
connectors = "0.1.6"
passive = "0.2.2"
std = "0.3.0"
```

三段各管一件事：`[package]` 是这个包自己的身份，发布时用；`[design]` 指定顶层设计名；`[dependencies]` 列出用到的库。`std` 也在依赖里，它是每个包的隐式前导，但版本要显式钉住，std 不是特殊包。

版本只接受精确的 `X.Y.Z`。写 `passive = "^0.2"` 会在解析 manifest 时就被拒绝：

```text
error[E1101]: dependency `passive`: `^0.2` is a version range — CoHDL requires exact
versions (a hardware library's "patch" can move real copper; every bump is an explicit
`cohdl update`)
  = help: did you mean `passive = "0.2.0"`?
```

消息里那句话就是理由：软件库的补丁版本改的是行为，硬件库的补丁版本可能改的是焊盘尺寸。没有一种范围是安全的，所以语言永久不支持范围。整段 `[dependencies]` 缺失是 E1104，帮助信息会给出要补的段落。

**PCB 对应**：`[dependencies]` 是这块板用到的元件库清单。版本号钉死，等于把「这批封装和参数」钉死。

## `cohdl.lock`：内容哈希，每次编译先验

```toml
{{#include ../../../examples/syntax-tour/cohdl.lock}}
```

每个依赖一行版本、一个 SHA-256。哈希覆盖包的全部内容：`.cohdl` 源码、`#[doc]` 引用的文档、封装。每次 `check` 和 `build` 在打开任何 `.cohdl` 文件之前先重新计算并比对，不一致是硬错误。把 `connectors` 那行哈希改掉一个字符：

```text
error[E1103]: locked package `connectors 0.1.6` has changed on disk: locked sha256:eca0…,
found sha256:dca0…
  = help: the content of a locked version must never change; if this bump is intentional,
    publish it as a new version and run `cohdl update`
```

三条行为规则，都实测过：

- manifest 里有、lock 里没有的依赖，第一次编译时补一行。这是普通 `check`/`build` 唯一会写 lock 的情况。
- 没有 `cohdl.lock` 文件，第一次编译直接生成。
- 已有的行，普通编译从不改写。改哈希只有一条路：`cohdl update`。

lock 文件要提交进版本库，和 `design.lock` 一样。它是几个月后回答「这块板当时用的是哪份库」的唯一凭据，不需要联网。手改坏了是 E1107，帮助信息说从版本控制恢复或删掉重新解析。

**PCB 对应**：`cohdl.lock` 是制造记录的一部分。版本号是人给的标签，哈希是内容本身；D001 比较的耐压、E807 比较的焊盘号，都来自被哈希锁住的那份文件。

## 包从哪里来

按依赖名逐个查找，顺序固定，std 和任何包走同一条规则：

1. 工程自己的 `deps/<name>/`
2. 编译器的库根目录 `lib/<name>/`
3. 注册表缓存 `~/.cohdl/registry`，由 `install`、`add`、`update` 填充

版本永远来自包自己 manifest 的 `[package] version`，目录名只是习惯。一个目录下可以并排放同一个包的多个版本，各自用任意目录名；两个目录声明同一个身份是 E1106。找不到声明了指定版本的包是 E1102，帮助信息列出搜索过的位置并提示 `cohdl install`。

本仓库的 `lib/` 就是官方包的源码，所以在仓库里跑 syntax-tour 不需要联网。换一台机器、在仓库外新建工程，就要靠 `cohdl install` 把锁定的版本拉进缓存。

## 六个动词各改什么

| 命令 | 改 `cohdl.toml` | 改 `cohdl.lock` | 需要注册表 |
| --- | --- | --- | --- |
| `cohdl add NAME[@X.Y.Z] [PATH]` | 加一行 | 加一行，连同它的传递依赖 | 先查本地，本地没有再取 |
| `cohdl remove NAME [PATH]` | 删一行 | 删一行 | 否 |
| `cohdl install [PATH]` | 否 | 补缺的行 | 缓存里没有的才取 |
| `cohdl update [PATH] [--dep NAME]` | 改到最新精确版本 | 重写 | 先查注册表，不可达则用本地 |
| `cohdl login` | 否 | 否 | 是，存 token |
| `cohdl publish [PATH]` | 否 | 否 | 是 |

在 syntax-tour 的副本上：

```sh
cohdl add led@0.1.2 .
#   added led 0.1.2 — official (bare name — reserved for CoHDL's own packages)
```

`cohdl.toml` 多了 `led = "0.1.2"`，`cohdl.lock` 多了一段带哈希的 `[[package]]`。输出顺便报出了这个名字所在的命名空间层级。

```sh
cohdl remove connectors .
#   removed connectors
cohdl check .
#   error[E202]: unresolved `use` path `connectors::headers::smd_254::SOCKET_2X3_254_SMD`
```

`remove` 只改两个文件，不检查源码是否还在用，下一次 `check` 才报名字找不到。删一个本来不在清单里的名字是 E1205，帮助信息列出当前依赖。

`update` 是唯一被允许改动已锁定哈希的命令，也是把旧工程迁移到 RFC-029 格式的命令。它会连 `std` 一起重新锁定并重写整个文件。

**传递依赖**：解析走完整闭包，每个被解析的包自己的 `[dependencies]` 也加入编译，lock 记录整个闭包。根工程的 pin 是唯一权威，静默压过任何依赖的 pin；两个依赖钉了同一个包的不同版本而根工程没钉，是 E1108，帮助信息让你在根工程钉一个。

## 三级命名空间

包的层级写在名字里，不靠元数据：

```toml
[dependencies]
passive = "0.2.2"                 # 裸名：CoHDL 官方，保留给项目自己的账号
"@raspberrypi/mcu" = "0.1.2"      # @brand/name：人工核验过的厂商账号
"@contrib/awesome-leds" = "0.3.0" # @contrib/name：任何登录账号，先到先得
```

带 `@` 的名字在 TOML 里要加引号；在 `.cohdl` 源码里，它变成下划线命名空间 `raspberrypi_mcu::RP2350A_QFN60`。`@sparkfun/power` 和 `@contrib/power` 是两个互不冲突的名字。

命名空间在客户端预检，服务端再检一次。`publish` 到一个没拥有的裸名或未核验的 `@brand` 会被拒绝（E1202）；`publish` 还要求 manifest 有 `license`，没有直接拒绝，不校验许可证名单。登录缺失或 token 被拒是 E1201。向注册表要一个不存在的版本是 E1203，例如：

```text
error[E1203]: `@not_verified/thing 1.0.0` is not published on the registry
```

注册表不可达是 E1204。它和 E1103 是刻意分开的两个码：一个是「拿不到」，一个是「拿到的和锁不一样」，永远不混用。发布时客户端算的哈希和服务端算的不一致是 E1206 警告，写进别人 lock 的是服务端那个。

**PCB 对应**：三层就是元件来源的信任等级。官方包、厂商官方包、社区包，从名字上一眼可分，和采购里「原厂、授权代理、第三方」的区分是同一件事。

## 库作者还会用到的

- `#[doc("docs/xxx.pdf")]` 可以多条，路径相对包根。编译器从不打开，注册表的包页面把它列为参考文档。
- `cohdl docs [PATH]` 从一个能干净通过 `check` 的包提取一份确定性的 API 文档 JSON；`publish` 成功后自动尽力上传，失败只警告不影响发布。它不进包的 tar，所以不影响哈希。
- `cohdl search QUERY` 不需要工程也不需要登录，搜包名和最新发布版本里的 `pub part`，结果有界，只报「还有没有更多」，不报总数。

## 这一层编译器不检查什么

它验证「拿到的是不是锁住的那份」，不验证那份库本身对不对：封装尺寸是否和数据手册一致、`alt` 料是否真能替换、一个 `@contrib` 包的作者是否可信。命名空间保证的是「谁发的」，不是「发得对」。
