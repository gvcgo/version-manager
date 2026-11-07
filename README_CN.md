<p align="center">
  <img src="vmr/docs/brand/logo.png" alt="vmr" width="640">
</p>

# vmr · 跨平台 SDK 版本管理器

[English](README.md) | 简体中文

`vmr` 帮你把开发工具的多套版本装在一台机器上，并随时切换到需要的那一套：Go、Node.js、Python、JDK、Rust、Lua…… 一条命令装、切、卸，不需要管理员权限，也不会污染系统目录。

- **多版本共存**：同一个 SDK 可以同时装很多版本，切换只是换一个软链接。
- **项目级锁定**：进项目目录自动切到项目需要的版本，离开自动还原。
- **会话隔离**：临时用某个版本跑一条命令或开一个子 shell，退出即恢复。
- **一条命令迁移新机器**：导出已装版本清单，在新机器上批量装回。
- **能装任意 conda-forge 包**：内置 pixi 的安装能力，不需要预装 conda/miniconda。
- **下载走多线程**：支持校验和、反向代理、本地代理与 `HTTP(S)_PROXY`。

支持 Linux / macOS / Windows，x86_64 与 aarch64 均有预编译包。

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

### 6. 下载

到 [Releases](https://github.com/gvcgo/version-manager/releases) 下载对应平台的包：

| 平台 | 包名示例 |
|---|---|
| macOS (Apple 芯片) | `vmr-aarch64-apple-darwin.tar.xz` |
| macOS (Intel) | `vmr-x86_64-apple-darwin.tar.xz` |
| Linux (x86_64) | `vmr-x86_64-unknown-linux-gnu.tar.xz` |
| Linux (ARM64) | `vmr-aarch64-unknown-linux-gnu.tar.xz` |
| Windows (x64) | `vmr-x86_64-pc-windows-msvc.zip` |
| Windows (ARM64) | `vmr-aarch64-pc-windows-msvc.zip` |

### 7. 解压并放到 PATH

Linux / macOS：

```bash
tar -xJf vmr-x86_64-apple-darwin.tar.xz
mkdir -p ~/.local/bin
install -m 755 vmr-x86_64-apple-darwin/vmr ~/.local/bin/vmr
# 确保 ~/.local/bin 在 PATH 中
```

Windows：解压后把 `vmr.exe` 放到任意已在 `PATH` 里的目录。

### 8. 校验（可选）

每个包都附带 `.sha256` 文件，Release 页面还有汇总的 `SHA256SUMS`：

```bash
shasum -a 256 -c vmr-x86_64-apple-darwin.tar.xz.sha256
```

### 9. 确认安装

```bash
vmr version
```

## 接入 Shell

安装完成后，把下面这一行加到你的 shell 配置里（`~/.zshrc`、`~/.bashrc`、`~/.config/fish/config.fish`）：

```bash
# zsh
eval "$(vmr init zsh)"
```

```bash
# bash
eval "$(vmr init bash)"
```

```fish
# fish
vmr init fish | source
```

```powershell
# Windows PowerShell：在 $PROFILE 中加入
Invoke-Expression (& vmr init powershell)
```

这一行会做三件事：

1. 把 vmr 维护的环境变量（`PATH`、`GOROOT` 等）注入当前 shell；
2. 注册 `cd` 钩子，进入带版本锁定的项目目录时自动切换；
3. 定义 `vmr` 包装函数 —— 执行 `vmr use` / `vmr uninstall` 之后，**当前窗口立即生效**，不用再开新终端。

未执行 `vmr init` 也能用命令，只是每次切换版本后需要新开终端。

## 快速上手

```bash
vmr plugins              # 看看支持哪些 SDK
vmr search go            # 列出当前平台可装的 Go 版本
vmr use go@1.22.0        # 安装并切换
vmr local go             # 查看已装版本，<current> 标记当前生效的
vmr uninstall go@1.22.0  # 卸载指定版本
vmr uninstall go@all     # 卸载这个 SDK 的所有版本
```

`vmr search` 的结果来自插件实时抓取，第一次查询会联网；加 `-n` 可忽略本地缓存重新拉取。

## 常用命令

| 命令 | 别名 | 说明 |
|---|---|---|
| `vmr version` | `v` | 显示版本 |
| `vmr init [shell]` | | 输出 shell 集成脚本 |
| `vmr plugins [-u]` | | 列出支持的 SDK；`-u` 先更新插件包 |
| `vmr search <sdk>` | `s` | 列出可安装版本，`-p` 走 conda-forge 源，`-n` 忽略缓存 |
| `vmr use <sdk>@<version>` | `u` / `h` | 安装并切换（默认全局） |
| `vmr use -s ...` | | 仅对本次会话生效 |
| `vmr use -l ...` | | 写入项目 `.vmr.lock` 并进入新会话 |
| `vmr use -p ...` | | 安装任意 conda-forge 包 |
| `vmr use -f <file>` | | 按清单批量安装 |
| `vmr local [sdk]` | `l` | 已装版本列表；`-c` 只显示当前版本 |
| `vmr freeze [file]` | `fz` | 导出已装版本清单 |
| `vmr uninstall <sdk>@<version>` | `uni` / `r` | 卸载，`@all` 卸载全部版本 |
| `vmr cdhook` | `cd-hook` | cd 钩子内部调用：读取 `.vmr.lock` 并注入环境 |
| `vmr session` | `se` | 查询当前 shell 是否处于 vmr 会话中 |
| `vmr nested` | `ns` | 允许 / 禁止嵌套会话 |
| `vmr mcp` | | 以 stdio 方式提供 MCP 服务，供 AI 客户端调用 |

## 项目级版本锁定

想让某个仓库固定使用特定版本，在项目根目录执行一次：

```bash
cd my-project
vmr use -l node@20.11.0
```

这会：

- 在项目根目录写入 `.vmr.lock`（记录本项目需要的 SDK 与版本）；
- 因为执行了 shell 集成，之后每次 `cd` 进这个目录，`cdhook` 会自动切到锁定版本；
- 离开目录后由 shell 钩子还原。

`.vmr.lock` 建议提交到仓库，团队成员 `cd` 进去就自动对齐版本。

## 会话模式

临时用某个版本跑命令，不影响全局设置：

```bash
vmr use -s python@3.12.2
# 退出该子 shell 后自动恢复
```

适合「我只想在这个终端里试一下新版本」的场景。若在会话中再想开一层会话，需要先 `vmr nested` 放行。

## 备份与迁移

```bash
vmr freeze               # 导出，默认生成 vmr_20260101.txt
vmr use -f vmr_20260101.txt   # 在新机器上批量装回
```

清单是纯文本，一行一个 `sdk@version`，`#` 开头为注释。每个 SDK 的当前版本写在最后，所以批量装完后生效版本与导出时一致。

## 安装任意 conda-forge 包

vmr 内置 pixi 的安装能力，因此除了插件里的 SDK，也能直接装 conda-forge 上的包：

```bash
vmr search -p lua        # 从 conda-forge 查询
vmr use -p lua@5.4.6     # 安装
```

默认频道是 `https://conda.anaconda.org/conda-forge`，可用环境变量换成镜像：

```bash
export VMR_PIXI_CHANNEL=https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/conda-forge
vmr search -p lua
```

## 配置

配置文件在 `~/.vmr/conf.toml`，键名为 PascalCase：

| 键 | 说明 |
|---|---|
| `ProxyUri` | 本地代理，如 `http://127.0.0.1:7890` |
| `ReverseProxy` | 反向代理前缀 |
| `SDKIntallationDir` | 版本安装目录（注意键名拼写如此） |
| `VersionHostUrl` | 版本元数据服务地址 |
| `ThreadNum` | 下载线程数 |
| `AllowNestedSessions` | 是否允许嵌套会话 |
| `GithubToken` | GitHub API Token |
| `CacheRetentionTime` | 缓存保留秒数 |
| `DisableCache` | 关闭缓存 |

常用环境变量（优先级高于配置文件）：

| 变量 | 说明 |
|---|---|
| `VMR_SDK_INSTALLATION_DIR` | 版本与缓存根目录 |
| `VMR_HOST` | 版本元数据服务地址 |
| `VMR_LOCAL_PROXY` | 本地代理 |
| `VMR_REVERSE_PROXY` | 反向代理前缀 |
| `VMR_DOWNLOAD_THREADS` | 下载线程数 |
| `VMR_ALLOW_NESTED_SESSIONS` | 允许嵌套会话 |
| `VMR_PIXI_CHANNEL` | conda-forge 频道（支持镜像） |
| `VMR_GITHUB_TOKEN` | GitHub Token，未设置时回退到 `GITHUB_TOKEN` |
| `HTTP_PROXY` / `HTTPS_PROXY` / `NO_PROXY` | 标准代理变量，直接生效 |

GitHub API 在三个来源都为空时以匿名身份请求，每小时 60 次，只够刷新少量 SDK 的版本列表；装得多建议配一个 Token。

## 数据都放在哪

```
~/.vmr/
├── conf.toml            # 配置
├── vmr.sh / vmr.fish    # 动态环境变量（PATH、GOROOT 等）
├── env_keys.txt         # 刷新前需要清掉的非 PATH 变量
├── plugins/             # SDK 版本插件（首次使用自动下载）
├── cache/               # 版本列表与安装包缓存
└── versions/
    └── <sdk>_versions/
        ├── <sdk>-<version>/   # 实际安装目录
        └── <sdk>              # 指向当前版本
```

想放到别的磁盘，设置 `VMR_SDK_INSTALLATION_DIR` 即可。

Windows 上，全局环境变量写入注册表 `HKCU\Environment` 并广播变更，所以 `vmr init powershell` 之外不需要额外修改系统设置。

## 卸载 vmr

```bash
vmr uninstall <sdk>@all   # 先清掉不需要的 SDK 版本
rm -rf ~/.vmr             # 再删除配置、缓存与剩余版本
# 最后把 shell 配置里那行 eval "$(vmr init ...)" 删掉
```

Windows 删除 `%USERPROFILE%\.vmr` 目录，并从 `PATH` 中移除 `vmr.exe` 所在目录。

## 常见问题

**`vmr: command not found`**
可执行文件没进 `PATH`。确认 `which vmr`（Windows 用 `where vmr`）能找到。

**切换了版本，当前终端还是旧的**
没有执行 `vmr init`，或者用了 `command vmr` 绕过了包装函数。补上 shell 集成后重开终端。

**`vmr search` 报限流**
GitHub 匿名请求每小时 60 次。配置 `GithubToken` 或环境变量 `VMR_GITHUB_TOKEN`。

**下载很慢 / 连不上**
先设 `HTTP_PROXY` / `HTTPS_PROXY`；只代理 vmr 时用 `VMR_LOCAL_PROXY`；conda-forge 走镜像用 `VMR_PIXI_CHANNEL`。

**想重新拉版本列表**
`vmr search <sdk> -n`，或删掉 `~/.vmr/cache/` 后重试。

**会话里再执行 `vmr use` 报错**
默认禁止嵌套会话：先 `vmr nested` 放行，或退出当前会话。

**升级 vmr 本身**
下载新版本的包覆盖旧二进制即可，`~/.vmr` 下的版本与配置不受影响。

## 反馈

- 问题与建议：[Issues](https://github.com/gvcgo/version-manager/issues)
- 版本下载：[Releases](https://github.com/gvcgo/version-manager/releases)
- 实现与开发说明见 [vmr/readme.md](vmr/readme.md)
