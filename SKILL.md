# VMR (Version Manager) — Go 实现总结

> 本文档用于后续 Rust 改写参考。描述 vmr-go 项目的完整架构、模块职责、数据流和关键设计决策。

---

## 1. 项目概览

**VMR** 是一个跨平台（Windows/Linux/macOS）、多语言 SDK 版本管理器。类比 `asdf-vm`，但支持 Windows、内置 TUI、无需手动安装插件（开箱即用）。

- **仓库地址**: `github.com/gvcgo/version-manager`
- **入口**: `cmd/vmr/main.go` → cobra CLI
- **Go 模块路径**: `github.com/gvcgo/version-manager/vmr-go`

### 核心功能
| 功能 | 说明 |
|------|------|
| 多 SDK 版本安装 | 支持 40+ 语言/工具 (go, node, python, rust, jdk, zig, ...) |
| 版本搜索 | TUI/CLI 列出可用版本 |
| 版本切换 | `vmr use <sdk>@<version>` 全局或会话级切换 |
| 版本锁定 | 项目目录下自动切换到锁定版本 (cd-hook) |
| Shell 集成 | bash / zsh / fish / powershell |
| 代理支持 | 反向代理 + 本地代理 + 自定义镜像 |
| Lua 插件系统 | 每个 SDK 对应一个 `.lua` 爬取+安装配置脚本 |
| 自安装/自更新 | `install-self` / `uninstall-self` |

---

## 2. 目录结构

```
vmr-go/
├── main.go                          # 空壳（全是注释掉的测试代码）
├── cmd/vmr/
│   ├── main.go                      # 真正的入口：import cnf(init) → cli.New().Run()
│   └── cli/
│       ├── cli.go                   # cobra 根命令 + 所有子命令注册
│       ├── use.go                   # `vmr use` 命令
│       ├── proxy.go                 # 代理设置命令
│       ├── install_self.go          # 自安装
│       ├── uninstall_self.go        # 自卸载
│       ├── download_threads.go      # 下载线程数
│       ├── customed_mirrors.go      # 自定义镜像
│       └── vcli/                    # 子命令组
│           ├── completions.go       # shell 自动补全
│           ├── local.go             # 本地已安装列表
│           ├── plugin.go            # 插件管理
│           ├── search.go            # 版本搜索
│           ├── sessions.go          # 会话模式
│           ├── show.go              # 显示 SDK 信息
│           └── uninstall.go         # 卸载版本
├── internal/
│   ├── cnf/                         # 配置系统
│   │   ├── conf.go                  # VMRConf 结构体 + 目录路径辅助函数
│   │   └── common.go                # URL构建、代理、Fetcher工厂
│   ├── completions/                 # shell 补全
│   ├── download/                    # 下载器
│   │   └── sdk_file.go             # Downloader 结构体
│   ├── install/                     # 旧版安装包（已废弃，不再使用）
│   ├── installer/                   # 核心安装器（当前使用）
│   │   ├── installer.go            # Installer 主逻辑 + 策略分发
│   │   ├── installed.go            # 已安装版本查找
│   │   ├── locker.go               # 版本锁定 + cd-hook
│   │   ├── prequisite.go           # 前置依赖（miniconda/coursier）
│   │   ├── search_by_conda.go      # conda 搜索
│   │   ├── cached.go               # 缓存管理
│   │   ├── install/                # 安装策略
│   │   │   ├── common.go           # 公共数据结构
│   │   │   ├── unarchiver.go       # 解压安装器（默认策略）
│   │   │   ├── executable.go       # 可执行文件安装器
│   │   │   ├── conda.go            # conda 安装器
│   │   │   ├── coursier.go         # coursier 安装器（scala系）
│   │   │   ├── compile.go          # 编译安装（桩）
│   │   │   └── rustup.go           # rustup 安装（桩）
│   │   └── post/                   # 安装后处理
│   │       ├── post.go             # 处理器注册表
│   │       ├── rust.go, bun.go, php.go, zig.go, upx.go, moonbit.go, clojure.go
│   ├── luapi/                       # Lua 插件系统
│   │   ├── lua.go                  # 空壳
│   │   ├── lua_global/             # Go → Lua 全局函数绑定
│   │   │   ├── lua.go              # gopher-lua VM 初始化 + 50+ vmr* 函数
│   │   │   ├── installer.go        # 安装配置结构体 + Lua 绑定
│   │   │   ├── req.go, gjson.go, goquery.go  # HTTP/JSON/HTML 解析
│   │   │   ├── version.go          # 版本列表操作
│   │   │   ├── utils.go, unarchiver.go, proxy.go
│   │   │   ├── github.go, conda.go
│   │   │   └── gh/gh.go            # GitHub API
│   │   └── plugin/                 # 插件管理器
│   │       ├── plugins.go          # Plugins 管理器
│   │       ├── plugin.go           # Plugin 类型（Load, GetSDKVersions, crawl）
│   │       ├── sdk.go              # Versions 类型（缓存+配置获取）
│   │       ├── fromlua.go          # Lua key 常量定义
│   │       └── download.go         # 插件下载更新
│   ├── self/                        # 自安装/卸载
│   │   ├── install.go              # InstallSelf()
│   │   ├── update_script.go        # 更新脚本
│   │   ├── uninstall_script.go
│   │   ├── source_script.go
│   │   └── old_versions.go         # 旧版本检测移除
│   ├── shell/                       # Shell 集成
│   │   ├── shell.go               # Sheller 接口
│   │   ├── unix.go                 # Unix Shell 管理
│   │   ├── win.go                  # Windows Shell 管理
│   │   └── sh/                     # shell 抽象层
│   │       ├── shell.go            # 公共常量 + 接口
│   │       ├── bash.go, zsh.go, fish.go
│   ├── terminal/                    # 终端（会话模式用）
│   │   ├── terminal.go
│   │   └── term/                   # PTY + fdset
│   ├── tui/                         # TUI 界面（bubbletea 风格）
│   │   ├── cmds/                   # TUI 命令
│   │   ├── cliui/                  # CLI 风格的列表组件
│   │   └── table/                  # 表格组件
│   └── utils/                       # 工具函数
│       ├── check.go, copy.go, exec.go, extractor.go
│       ├── find_dir.go, open_url.go, sort_versions.go
│       └── file_{linux,darwin,win}.go
```

---

## 3. 核心架构

### 3.1 数据流：`vmr use go@1.21.0`

```
用户执行命令
    │
    ▼
cobra CLI (cmd/vmr/cli/use.go)
    │  解析: sdkName="go", versionName="1.21.0"
    ▼
installer.NewInstaller(pluginName, sdkName, versionName, version)
    │  1. 根据 version.Installer 选择策略: Archiver/Exe/Conda/Coursier
    │  2. 加载 Lua 插件获取安装配置 (InstallerConfig)
    ▼
Installer.Install()
    │  1. 检查/安装前置依赖 (miniconda/coursier)
    │  2. 调用具体策略的 Install()
    │     ├─ ArchiverInstaller: 下载→解压→目录查找→复制到versions/→建软链接
    │     ├─ ExeInstaller:      下载→运行安装程序→复制到versions/
    │     ├─ CondaInstaller:    conda create --prefix=<dir> <sdk>=<ver>
    │     └─ CoursierInstaller: cs install --dir=<dir> <sdk>:<ver>
    │  3. 运行 post-install handler (如 rust 的 rustup-init, bun 的 bunx 软链接)
    │  4. 按模式处理环境变量:
    │     ├─ 全局模式:   创建软链接 → shell配置写入PATH → 全局生效
    │     ├─ 会话模式:   写入锁文件 → 移除全局PATH → 加入临时PATH → 启动session shell
    │     └─ 锁定模式:   写入锁文件 (用于 cd-hook 自动切换)
    ▼
完成
```

### 3.2 Lua 插件系统

每个 SDK 对应一个 `.lua` 文件，位于 `~/.vmr/plugins/`（首次从 GitHub 下载）。

**Lua 插件必须导出的全局变量/函数：**

| 变量/函数 | 说明 |
|-----------|------|
| `plugin_name` | 插件名（如 `"go"`） |
| `sdk_name` | SDK 名（如 `"go"`） |
| `homepage` | 官网地址 |
| `plugin_version` | 插件版本号 |
| `prequisite` | 前置依赖类型（`"conda"` 或 `"coursier"` 或空） |
| `ic` | InstallerConfig（安装配置 Lua table） |
| `crawl` | 版本爬取函数 → 返回 VersionList |
| `postInstall` | (可选) Lua 层面的安装后处理 |
| `install` | (可选) 自定义安装函数（替代 Go 策略） |

**Go 侧通过 `gopher-lua` 提供 50+ `vmr*` 全局函数：**
- HTTP: `vmrHttpGet`, `vmrGetResponse` → 返回 `{status,body}`
- JSON: `vmrInitGJson`, `vmrGJsonGet`, `vmrGJsonArrayCount` 等
- HTML: `vmrInitSelection`, `vmrFind`, `vmrAttr`, `vmrText` 等（goquery）
- GitHub: `vmrNewGithubParser`, `vmrGetReleaseList` 等
- Conda: `vmrSearchByConda` / `vmrSearchByCondaForge`
- 工具: `vmrSprintf`, `vmrGetOsArch`, `vmrExecSystemCmd`
- 版本: `vmrNewVersionList`, `vmrAddItem`, `vmrMergeVersionList`
- 安装配置: `vmrNewInstallerConfig`, `vmrAddFlagFiles`, `vmrAddBinaryDirs` 等

### 3.3 安装策略（InstallerConfig）

```go
// Go 侧结构体（内部使用）
type InstallerConfig struct {
    FlagFiles       map[string]FileItems   // OS→文件匹配（解压后定位目录）
    FlagDirExcepted bool                   // 排除 FlagFiles 指定的目录
    BinaryDirs      map[string]DirItems    // OS→二进制目录路径
    BinaryRename    map[string]BinaryRename // 二进制文件重命名
    AdditionalEnvs  []AdditionalEnv        // 额外环境变量
}
```

**各策略使用场景：**

| 策略 | 适用 SDK | 核心逻辑 |
|------|---------|---------|
| ArchiverInstaller | go, node, python, jdk... | 下载压缩包→解压→找到目标目录→复制到版本目录 |
| ExeInstaller | miniconda, vscode, erlang, elixir | 下载安装程序→静默安装→复制安装目录 |
| CondaInstaller | python, R, julia | `conda create --prefix=<dir> sdk=version` |
| CoursierInstaller | scala, kotlin | `cs install --dir=<dir> sdk:version` |

---

## 4. 关键模块详解

### 4.1 配置系统 (`internal/cnf/`)

**VMRConf** 结构体（TOML 序列化到 `~/.vmr/conf.toml`）：

| 字段 | 说明 | 默认值 |
|------|------|--------|
| `ProxyUri` | 本地 HTTP 代理 | 空 |
| `ReverseProxy` | 反向代理 URL | `https://proxy.vmr.dpdns.org/proxy/` |
| `SDKIntallationDir` | SDK 安装根目录 | `~/.vmr` |
| `VersionHostUrl` | 版本数据源 | `raw.githubusercontent.com/gvcgo/vsources/main` |
| `ThreadNum` | 下载线程数 | 1 |
| `UseCustomedMirrors` | 使用自定义镜像 | false |
| `AllowNestedSessions` | 允许嵌套会话 | false |
| `GithubToken` | GitHub API Token | 空 |
| `CacheRetentionTime` | 版本缓存有效期(秒) | 86400 |
| `DisableCache` | 禁用版本缓存 | false |

**目录结构（`~/.vmr/`）：**
```
~/.vmr/
├── conf.toml              # 用户配置
├── versions/              # 已安装的 SDK 版本目录
│   ├── go/                # go 的软链接 → go_versions/go-1.21.0
│   ├── go_versions/       # go 各版本文件
│   │   └── go-1.21.0/
│   ├── node/
│   └── node_versions/
├── cache/                 # 下载缓存
├── temp/                  # 临时文件
├── plugins/               # Lua 插件 (.lua 文件)
├── install_confs/         # 安装配置文件缓存
├── vmr.sh                 # shell 环境变量脚本
└── lockfiles/             # 项目版本锁定文件
```

**Fetcher 工厂 (`GetFetcher`)：**
- 对 GitHub 等国外 URL 自动添加反向代理前缀
- 支持自定义镜像域名替换
- 大文件（非 .json/.toml）支持多线程下载
- 跳过 gitee.com 的代理处理

### 4.2 安装器核心 (`internal/installer/installer.go`)

**SDKInstaller 接口：**
```go
type SDKInstaller interface {
    Initiate(pluginName, sdkName, versionName string, version lua_global.Item)
    SetInstallConf(conf *install.InstallerConfig)
    GetInstallDir() string
    GetSymbolLinkPath() string
    Install()
}
```

**InvokeMode 三种模式：**
- `globally` — 全局安装，写入 shell 配置
- `sessionly` — 临时会话，启动交互式 shell，退出后恢复
- `to-lock` — 仅写锁文件，供 cd-hook 使用

### 4.3 版本锁定 (`internal/installer/locker.go`)

**VersionLocker** 在特定目录写入锁文件，记录应使用的 SDK 版本。cd-hook 检测到锁文件后自动切换版本。

### 4.4 Shell 集成 (`internal/shell/`)

**Sheller 接口** 负责：
- 写入 `~/.vmr/vmr.sh`（PATH 和环境变量）
- 配置 shell rc 文件（`.zshrc`/`.bashrc`），添加 `# cd hook start...end` 标记块
- 支持 bash, zsh, fish, powershell

**Windows 支持：**
- 环境变量通过注册表修改
- PATH 永久化依赖 Windows API

### 4.5 TUI 界面 (`internal/tui/`)

- 基于 bubbletea 风格（实际使用 promptkit 库）
- `cmds.NewTUI().ListPluginName()` — 主界面，展示 SDK 列表
- 支持键盘导航、搜索过滤、版本选择

---

## 5. 外部依赖（Go 侧关键库）

| 库 | 用途 |
|----|------|
| `cobra` | CLI 框架 |
| `gopher-lua` | Lua VM 嵌入 |
| `goquery` | HTML 解析（供 Lua 插件使用） |
| `gjson` | JSON 路径查询（供 Lua 插件使用） |
| `promptkit` | TUI 组件 |
| `grequests` / `request` | HTTP 请求（自定义 fetcher） |
| `gutils` | 工具函数（复制目录、执行命令等） |

---

## 6. Rust 改写要点

### 6.1 核心挑战

1. **Lua 插件系统**: `gopher-lua` → Rust 中可用 `mlua` 或 `rlua` 替代，API 映射关系需重新设计
2. **跨平台文件/Shell**: Go 的 `filepath`/`os` → Rust 的 `std::fs`、`dirs` crate
3. **HTTP 下载器**: Go 自定义 `request.Fetcher` → Rust 的 `reqwest`
4. **TUI**: `promptkit` → Rust 的 `ratatui` + `tui-textarea`
5. **TOML 配置**: Go 的 `BurntSushi/toml` → Rust 的 `toml` + `serde`
6. **CLI**: `cobra` → `clap`

### 6.2 架构可保留的设计

- **策略模式**：四种安装策略（archiver/exe/conda/coursier）映射为 Rust trait
- **插件系统**：Lua 全局函数绑定模式（Go 的 `vmr*` 函数 → Rust trait 方法注册）
- **配置系统**：TOML 序列化 + 环境变量覆盖
- **Shell 集成**：接口抽象 + 各 shell 实现
- **工作目录结构**：`.vmr/` 布局不变

### 6.3 可优化点

- 版本缓存可使用 SQLite 替代 JSON 文件
- 下载器可用 `tokio` 异步重写
- 插件系统可考虑 Lua → WASM 或纯 Rust 插件（长期）
- 废弃的 `internal/install/` 包不需要迁移
- `internal/luapi/lua.go`（空壳）不需要迁移

### 6.4 文件量统计

- Go 源文件：约 **100+** 个
- 核心逻辑集中在 `internal/installer/`（~15 文件）和 `internal/luapi/`（~25 文件）
- TUI 层：~10 文件
- Shell 层：~8 文件
- CLI 层：~12 文件

---

## 7. 关键环境变量

| 变量名 | 说明 |
|--------|------|
| `VMR_SDK_INSTALLATION_DIR` | SDK 安装根目录 |
| `VMR_HOST` | 版本数据源 URL |
| `VMR_REVERSE_PROXY` | 反向代理 URL |
| `VMR_LOCAL_PROXY` | 本地代理 |
| `VMR_DOWNLOAD_THREADS` | 下载线程数 |
| `VMR_USE_CUSTOMED_MIRRORS` | 启用自定义镜像 |
| `VMR_ALLOW_NESTED_SESSIONS` | 允许嵌套会话 |
| `VM_DISABLE` | 禁用 VMR（shell 集成） |
| `VMR_ADD_TO_PATH_TEMPORARILY` | 临时添加到 PATH |
