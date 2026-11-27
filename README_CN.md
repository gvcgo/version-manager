<p align="center">
  <img src="vmr/docs/brand/logo.png" alt="vmr" width="640">
</p>

# vmr · 跨平台 SDK 版本管理器

[English](README.md) | 简体中文

文档：[English](https://gvcgo.github.io/version-manager/) · [简体中文](https://gvcgo.github.io/version-manager/zh/)

`vmr` 帮你把开发工具的多套版本装在一台机器上，并随时切换到需要的那一套：Go、Node.js、Python、JDK、Rust、Lua…… 一条命令装、切、卸，不需要管理员权限，也不会污染系统目录。

- **多版本共存**：同一个 SDK 可以同时装很多版本，切换只是换一个软链接。
- **项目级锁定**：进项目目录自动切到项目需要的版本，离开自动还原。
- **会话隔离**：临时用某个版本跑一条命令或开一个子 shell，退出即恢复。
- **一条命令迁移新机器**：导出已装版本清单，在新机器上批量装回。
- **能装任意 conda-forge 包**：内置 pixi 的安装能力，不需要预装 conda/miniconda。
- **下载走多线程**：支持校验和、反向代理、本地代理与 `HTTP(S)_PROXY`。

支持 Linux / macOS / Windows，x86_64 与 aarch64 均有预编译包。

## 演示

![vmr 演示](vmr/docs/demo.gif)

演示由 [VHS](https://github.com/charmbracelet/vhs) 根据 [`vmr/docs/demo.tape`](vmr/docs/demo.tape) 录制生成。

## 安装

### 1. Homebrew（macOS / Linux）

```bash
brew install gvcgo/tap/vmr
```

公式直接使用各平台的预编译包，不在本地编译。Homebrew 只提供 `vmr` 可执行文件，各 SDK 的版本仍由 vmr 自己装到 `~/.vmr` 下。

### 2. MacPorts（macOS）

MacPorts 没有 Homebrew tap 那样的第三方仓库机制，因此 vmr 维护了一个独立的 port source。添加一次后即可安装：

```bash
sudo mkdir -p /opt/local/var/macports/sources/github.com/gvcgo
sudo git clone https://github.com/gvcgo/vmr-macports \
  /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports
echo "file:///opt/local/var/macports/sources/github.com/gvcgo/vmr-macports [nosync]" \
  | sudo tee -a /opt/local/etc/macports/sources.conf
sudo portindex /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports
sudo port install vmr
```

该 port 使用与 Homebrew、Release 包相同的预编译二进制，不在本地编译。如果你的 MacPorts 前缀不是 `/opt/local`，请自行替换。后续升级：

```bash
sudo git -C /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports pull
sudo portindex /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports
sudo port upgrade vmr
```

详见 [vmr-macports](https://github.com/gvcgo/vmr-macports)。

### 3. Scoop（Windows）

```powershell
scoop bucket add gvcgo https://github.com/gvcgo/scoop-bucket
scoop install gvcgo/vmr
```

### 4. APT（Debian / Ubuntu）

```bash
curl -fsSL https://gvcgo.github.io/vmr-apt/vmr.gpg \
  | sudo gpg --dearmor -o /usr/share/keyrings/vmr.gpg
echo "deb [signed-by=/usr/share/keyrings/vmr.gpg] https://gvcgo.github.io/vmr-apt stable main" \
  | sudo tee /etc/apt/sources.list.d/vmr.list
sudo apt update
sudo apt install vmr
```

### 5. pacman（Arch Linux）

```bash
curl -fsSLO https://gvcgo.github.io/vmr-pacman/vmr.gpg
sudo pacman-key --add vmr.gpg
sudo pacman-key --lsign-key C387E9AE5C936248D446868C44E5A0323545E858
```

把仓库追加到 `/etc/pacman.conf`：

```ini
[vmr]
SigLevel = Required DatabaseOptional
Server = https://gvcgo.github.io/vmr-pacman/$arch
```

然后安装：

```bash
sudo pacman -Syu vmr
```

### 6. Cargo（源码安装）

需要 Rust 1.93+ 和 C 工具链（macOS 安装 Xcode Command Line Tools，Windows 安装 MSVC Build Tools，Linux 安装 `build-essential` 与 `pkg-config`）。workspace 位于 `vmr/` 下，因此先克隆仓库，再安装 `vmr-cli`：

```bash
git clone https://github.com/gvcgo/version-manager
cd version-manager
cargo install --path vmr/crates/vmr-cli --locked
```

`vmr` 会安装到 `~/.cargo/bin`，请确保该目录在 `PATH` 中。更新代码后重新执行并加 `--force`：

```bash
cargo install --path vmr/crates/vmr-cli --locked --force
```

这些 crate 尚未发布到 crates.io，因此暂不支持 `cargo install vmr`。

### 7. 下载

到 [Releases](https://github.com/gvcgo/version-manager/releases) 下载对应平台的包：

| 平台 | 包名示例 |
|---|---|
| macOS (Apple 芯片) | `vmr-aarch64-apple-darwin.tar.xz` |
| macOS (Intel) | `vmr-x86_64-apple-darwin.tar.xz` |
| Linux (x86_64) | `vmr-x86_64-unknown-linux-gnu.tar.xz` |
| Linux (ARM64) | `vmr-aarch64-unknown-linux-gnu.tar.xz` |
| Windows (x64) | `vmr-x86_64-pc-windows-msvc.zip` |
| Windows (ARM64) | `vmr-aarch64-pc-windows-msvc.zip` |

### 8. 解压并放到 PATH

Linux / macOS：

```bash
tar -xJf vmr-x86_64-apple-darwin.tar.xz
mkdir -p ~/.local/bin
install -m 755 vmr-x86_64-apple-darwin/vmr ~/.local/bin/vmr
# 确保 ~/.local/bin 在 PATH 中
```

Windows：解压后把 `vmr.exe` 放到任意已在 `PATH` 里的目录。

### 9. 校验（可选）

每个包都附带 `.sha256` 文件，Release 页面还有汇总的 `SHA256SUMS`：

```bash
shasum -a 256 -c vmr-x86_64-apple-darwin.tar.xz.sha256
```

### 10. 确认安装

```bash
vmr version
```

## 文档

请参阅[文档站](https://gvcgo.github.io/version-manager/zh/)，了解[快速上手](https://gvcgo.github.io/version-manager/zh/starts/quickstart)、[使用教程](https://gvcgo.github.io/version-manager/zh/guides/tutorial)、[命令参考](https://gvcgo.github.io/version-manager/zh/reference/commands)和[插件 API](https://gvcgo.github.io/version-manager/zh/plugins/)。

## 反馈

- 问题与建议：[Issues](https://github.com/gvcgo/version-manager/issues)
- 版本下载：[Releases](https://github.com/gvcgo/version-manager/releases)
- 实现与开发说明见 [vmr/readme.md](vmr/readme.md)

## Star 走势

[![Star History Chart](https://api.star-history.com/svg?repos=gvcgo/version-manager&type=Date)](https://star-history.com/#gvcgo/version-manager&Date)
