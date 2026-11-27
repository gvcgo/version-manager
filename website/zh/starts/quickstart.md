# 快速上手

vmr 支持 Linux、macOS 和 Windows，提供 x86_64 与 aarch64 预编译包。下面先安装 vmr，再让当前 shell 接收版本切换。

## 安装 vmr

### Homebrew（macOS / Linux）

```bash
brew install gvcgo/tap/vmr
```

### Scoop（Windows）

```powershell
scoop bucket add gvcgo https://github.com/gvcgo/scoop-bucket
scoop install gvcgo/vmr
```

### APT（Debian / Ubuntu）

```bash
curl -fsSL https://gvcgo.github.io/vmr-apt/vmr.gpg | sudo gpg --dearmor -o /usr/share/keyrings/vmr.gpg
echo "deb [signed-by=/usr/share/keyrings/vmr.gpg] https://gvcgo.github.io/vmr-apt stable main" | sudo tee /etc/apt/sources.list.d/vmr.list
sudo apt update
sudo apt install vmr
```

### pacman（Arch Linux）

```bash
curl -fsSLO https://gvcgo.github.io/vmr-pacman/vmr.gpg
sudo pacman-key --add vmr.gpg
sudo pacman-key --lsign-key C387E9AE5C936248D446868C44E5A0323545E858
```

在 `/etc/pacman.conf` 中添加仓库，再安装：

```ini
[vmr]
SigLevel = Required DatabaseOptional
Server = https://gvcgo.github.io/vmr-pacman/$arch
```

```bash
sudo pacman -Syu vmr
```

### MacPorts（macOS）

```bash
sudo mkdir -p /opt/local/var/macports/sources/github.com/gvcgo
sudo git clone https://github.com/gvcgo/vmr-macports /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports
echo "file:///opt/local/var/macports/sources/github.com/gvcgo/vmr-macports [nosync]" | sudo tee -a /opt/local/etc/macports/sources.conf
sudo portindex /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports
sudo port install vmr
```

如果 MacPorts 前缀不是 `/opt/local`，请替换上述路径。

### 从源码安装

需要 Rust **1.88 或更新版本**及 C 工具链：macOS 使用 Xcode Command Line Tools，Windows 使用 MSVC Build Tools，Linux 安装 `build-essential` 和 `pkg-config`。

```bash
git clone https://github.com/gvcgo/version-manager
cd version-manager
cargo install --path vmr/crates/vmr-cli --locked
```

将 `~/.cargo/bin` 加入 `PATH`。更新源码后可在安装命令末尾加 `--force`。这些 crate 尚未发布到 crates.io，不能使用 `cargo install vmr`。

### 下载预编译包

从 [Releases](https://github.com/gvcgo/version-manager/releases) 下载对应系统和架构的压缩包，校验附带的 `.sha256` 文件，然后解压并将 `vmr`（Windows 为 `vmr.exe`）放入 `PATH`。例如 macOS Intel：

```bash
shasum -a 256 -c vmr-x86_64-apple-darwin.tar.xz.sha256
tar -xJf vmr-x86_64-apple-darwin.tar.xz
mkdir -p ~/.local/bin
install -m 755 vmr-x86_64-apple-darwin/vmr ~/.local/bin/vmr
```

确认 `~/.local/bin` 已在 `PATH` 中，接着运行 `vmr version`。

## 接入 shell

首次交互运行时，vmr 可能提示自动添加集成配置。也可以手动将对应命令加入配置文件，然后重新打开终端：

```bash
# ~/.zshrc
eval "$(vmr init zsh)"
# 或 ~/.bashrc
eval "$(vmr init bash)"
```

```fish
# ~/.config/fish/config.fish
vmr init fish | source
```

```powershell
# PowerShell 的 $PROFILE
Invoke-Expression (& vmr init powershell)
```

集成会刷新当前 shell 的环境、注册项目目录钩子，并使 `vmr use` 和 `vmr uninstall` 的变更立即生效。没有集成时命令仍可运行，但切换后需重新打开终端。

## 安装第一个 SDK

```bash
vmr plugins
vmr search go
vmr use go@1.22.0
vmr local go
```

`<current>` 标记当前版本。版本列表首次查询需要联网；`vmr search go -n` 可绕过缓存重新查询。继续阅读[核心概念](/zh/starts/concepts)和[使用教程](/zh/guides/tutorial)。
