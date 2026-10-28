**简体中文（默认）** | [English](README.en.md)

<img width="1920" alt="Azeroth Survivors zhCN 游戏画面" src="docs/screenshots/combat.zh-CN.jpg" />

<div align="center">

# Azeroth Survivors

**基于 Rust + [Bevy](https://bevy.org) 的 WoW 1.12.1 客户端引擎，运行在本地的生存类 Roguelite 游戏。**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE-MIT)

</div>

## 项目说明

本仓库 [sqhome-jin/azeroth-survivors-re-cn](https://github.com/sqhome-jin/azeroth-survivors-re-cn)
fork 自 [OutThisLife/azeroth-survivors-re](https://github.com/OutThisLife/azeroth-survivors-re)，用于本地开发与后续功能扩展，中文为默认文档语言。
本分支已加入 zhCN 本地化：默认中文角色/战场选择、Survivors 战斗 HUD 与升级卡，并按 locale 读取客户端 DBC 和 FrameXML 文本。部分自定义或历史硬编码文案仍可能显示英文；切换界面语言时也需要使用对应 locale 的客户端资源。

> **原作与致谢：** Azeroth Survivors 原作由 **[Aethus](https://wow.aethus.pl/)** 制作。
> 可在 [原作网站](https://wow.aethus.pl/) 体验线上版本，参阅
> [发布介绍](https://www.reddit.com/r/wowservers/comments/1x0yidl/azeroth_survivors_roguelike_wow_singleplayer_game/)
> 或加入 [原作 Discord](https://discord.gg/UzxaZuSAk)。
> 本仓库独立维护，不能视为线上版本完整源码或与其功能同步的版本。

选择职业和战场，在敌群中生存：使用 WASD 移动，技能自动释放，击杀敌人获取经验，升级时选择强化，并迎战定时出现的 Boss。
玩法不需要游戏服务器、登录或账号。浏览器版仍需本地 HTTP 服务提供页面和游戏资源；首次构建需要下载依赖与素材。

## 目录

- [当前功能](#当前功能)
- [快速开始](#快速开始)
- [操作方式](#操作方式)
- [客户端语言](#客户端语言)
- [游戏数据](#游戏数据)
- [架构与开发文档](#架构与开发文档)
- [文档语言](#文档语言)
- [许可与声明](#许可与声明)

## 当前功能

<table>
  <tr>
    <td width="50%"><img src="docs/screenshots/menu.zh-CN.jpg" alt="中文角色与战场选择界面"></td>
    <td width="50%"><img src="docs/screenshots/combat.zh-CN.jpg" alt="中文战斗 HUD"></td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/screenshots/level-up.zh-CN.jpg" alt="中文升级强化卡牌"></td>
    <td width="50%"><img src="docs/screenshots/results.zh-CN.jpg" alt="中文战斗结算界面"></td>
  </tr>
</table>

- **角色与战场选择**：八个种族、两种性别、符合种族限制的职业组合，以及十六张战场；记住上次选择。
- **本地生存战斗**：自动施法、敌群波次、经验拾取与定时 Boss，各战场使用对应主题的怪物和 Boss 阵容。
- **升级卡牌**：展示技能或被动强化、品质颜色和升级前后的具体数值，并带有发牌及悬停动画。
- **职业机动技能**：5 级解锁，按 F 向光标方向使用，如闪现、冲锋、疾跑等。
- **镜头控制**：默认俯视锁定，C 解锁旋转和俯仰，滚轮缩放，最远可至 70 码。
- **画面后处理**：G 切换锐化与色彩调整。
- **中文本地化**：默认 zhCN，覆盖 Survivors 界面和客户端本地化数据提供的文本；部分自定义文案仍可能为英文。

## 快速开始

### 环境要求

- 通过 [rustup](https://rustup.rs/) 安装 Rust，项目版本由 [rust-toolchain.toml](rust-toolchain.toml) 指定。
- `make`、`curl`、`unzip` 和 Bash；macOS 还需 Xcode Command Line Tools。
- 浏览器需支持 **WebGPU**，并使用 `localhost` 或 HTTPS 安全上下文；WebGL2 不支持完整世界渲染。
- Windows 请使用 Git Bash 执行脚本，不能直接使用 PowerShell 或 cmd 执行这些 Bash 脚本。

若安装 Rust 后提示 `rustup: command not found`，重新打开终端，或在 Bash / Zsh 中执行：

```bash
source "$HOME/.cargo/env"
```

### 浏览器版

```bash
git clone https://github.com/sqhome-jin/azeroth-survivors-re-cn.git
cd azeroth-survivors-re-cn
make build
make start
```

打开 <http://127.0.0.1:8090/>。首次游戏素材下载约 5.5 GB，另需下载编译依赖和工具链。
浏览器版默认使用 zhCN；可在地址后添加 `?locale=enUS` 切换到英文界面。请同时提供对应 locale 的客户端数据。

`make build` 按顺序运行以下可重复执行的步骤，跳过已完成的相应下载或安装：

| 步骤 | 作用 |
| --- | --- |
| [scripts/fetch-data.sh](scripts/fetch-data.sh) | 下载 WoW 1.12.1 客户端并提取 MPQ |
| [scripts/web-setup.sh](scripts/web-setup.sh) | 配置 WASM 目标、wasm-bindgen、WASI SDK 和 Binaryen |
| [scripts/web-build.sh](scripts/web-build.sh) | 编译浏览器客户端，输出到 `web/dist/` |

### 桌面版

```bash
make native
```

已有素材时，也可以指定自己的 Data 目录直接运行：

```bash
WOW_DATA=/path/to/WoW/Data cargo run --release -p warcraft-survivors
```

运行 `make help` 查看全部目标。**`make clean` 会删除已下载的游戏数据和浏览器构建产物，请谨慎使用。**

## 操作方式

| 按键 | 操作 |
| --- | --- |
| WASD | 移动 |
| Space | 跳跃 |
| F | 职业机动技能，5 级解锁 |
| 鼠标滚轮 | 镜头缩放 |
| C | 切换镜头锁定 |
| Esc | 暂停 |
| 1 / 2 / 3 | 选择升级卡牌 |
| G | 切换画面后处理 |

完整说明见 [CONTROLS.md](CONTROLS.md)（英文）。

## 客户端语言

客户端默认 locale 为 `zhCN`。浏览器版可用 `?locale=zhCN` 或 `?locale=enUS` 选择；桌面版可设置 `WOW_LOCALE=zhCN` 或 `WOW_LOCALE=enUS`。locale 只选择界面读取的语言，不会转换客户端资源；请让 `WOW_LOCALE` 与 `WOW_DATA`/`--data` 指向的客户端版本相匹配。缺失的单条 DBC 文本会回退到 enUS。

## 游戏数据

仓库不包含暴雪游戏资源文件，如模型、贴图、声音、地图、MPQ 内容和 FrameXML。
运行时需要 WoW 1.12.1 客户端的 `Data` 目录。

默认脚本从 [archive.org](https://archive.org/details/WoW_1.12.1-5875) 下载简体中文 **WoW 1.12.1（build 5875，zhCN）**，
将 MPQ 提取到 `wow-data/Data/`。该目录不纳入 Git，下载支持续传，已有数据会跳过下载。
使用其他来源的游戏数据时，请自行确认获取和使用权限。

### 使用已有素材

桌面版通过 `WOW_DATA` 指定路径；浏览器版可直接用 host 的 `--data` 参数：

```bash
cargo run --release -p wenilla-host -- --www web/dist --data /path/to/WoW/Data
```

本地 host 的资源接口不带登录认证，**不要直接暴露到公网**。

### 更换下载地址

```bash
make build WOW_DATA_URL=https://your-mirror.example/client.zip
```

也可修改 [scripts/fetch-data.sh](scripts/fetch-data.sh) 中的 `DEFAULT_DATA_URL`。
地址必须直接指向包含客户端 `Data/*.MPQ` 文件的 ZIP 压缩包。

## 架构与开发文档

玩法主要位于 [crates/benilla-app/src/survivors](crates/benilla-app/src/survivors)，桌面与浏览器启动器共用同一套 Rust 逻辑。
底层依赖两个上游项目：

- [benilla](https://github.com/samwhosung/benilla)：从零实现的 WoW 1.12.1 客户端。
- [wenilla](https://github.com/Arnesen/wenilla)：浏览器 WASM 移植及相关 host / realm 服务。

| 文档 | 内容 |
| --- | --- |
| [中文项目开发指南](docs/PROJECT-GUIDE.zh-CN.md) | 架构、模块、运行流程、功能修改入口和验证建议 |
| [AGENTS.md](AGENTS.md) | 仓库工作约定 |
| [docs/METHOD.md](docs/METHOD.md) | 引擎开发方法与约束 |
| [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) | 上游引擎贡献规范 |
| [docs/MAP.md](docs/MAP.md) | 自动生成的引擎代码索引 |
| [web/README.md](web/README.md) | 浏览器适配与 JavaScript bridge 说明 |

部分继承文档描述原客户端或其他上游组件，不能直接作为本分支的功能清单；请结合当前源码与中文指南阅读。

## 文档语言

- [README.md](README.md)：简体中文，默认项目入口。
- [README.en.md](README.en.md)：英文版，与中文版保持相同章节和核心信息。
- 修改安装命令、功能范围或重要声明时，请同步更新两版 README；新增语言时在各版本顶部添加互链。

## 许可与声明

Azeroth Survivors 原作由 **Aethus** 制作。本项目与 Blizzard Entertainment 无隶属关系，也未获其背书。
仓库不包含暴雪游戏资源文件；截图展示的是从外部客户端资源渲染的画面，相关美术版权归 Blizzard Entertainment 所有。
World of Warcraft 是 Blizzard Entertainment, Inc. 的商标。

仓库引擎代码采用 [MIT](LICENSE-MIT) 或 [Apache 2.0](LICENSE-APACHE) 双重许可，可任选其一。
[third_party](third_party) 中的第三方组件保留各自的上游许可证。代码许可证不授予暴雪素材的再分发权。
