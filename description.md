# vmr-go 模块功能说明书（Rust 重写依据）

> 本文档由 vmr-go 源码（`/home/moqsien/my_projects/go/src/version-manager/vmr-go`）整理而成。
> 目的：作为后续用 Rust 逐步重写各功能模块的依据。所有结论均来自源码，标注了源文件位置。
> 约定：`~/.vmr` 指工作目录（由 `cnf.GetVMRWorkDir()` 确定，即 `$HOME/.vmr`）。

---

## 1. 项目概述

vmr（version-manager）是一个**跨平台（Windows/Linux/macOS）的多 SDK/工具版本管理器**，类似 fnm/nvm/pyenv，但可管理任意语言与工具（go/node/python/jdk/rust/zig/conda 系等）。

核心特性（readme.md）：
- TUI + CLI 双模式，TUI 受 lazygit 启发，无需记忆命令
- 支持按项目锁定 SDK 版本（`vmr use -E` + cd hook）
- 反向代理 / 本地代理 / 自定义镜像，改善下载体验
- 通过 Lua 插件描述 SDK（**可扩展性核心**），无需硬编码 SDK 列表
- 通过 conda 扩展支持数千应用
- 支持 bash/zsh/fish/powershell/git-bash 多 shell 环境注入

架构核心机制：
1. 远程数据源（github.com/gvcgo/vsources 仓库）提供 `sdk-list.version.json`（SDK 清单）、`{sdkname}.version.json`（版本列表）、`install/{sdkname}.toml`（安装配置）。
2. SDK 版本获取主要走 **Lua 插件**（`~/.vmr/plugins/*.lua`），插件内 crawl 函数抓取版本列表，通过 `vmr*` 全局函数与 Go 交互。
3. 安装器按版本条目 `Item.Installer` 类型分派：`unarchiver`（下载+解压）/ `executable`（独立可执行文件）/ `conda` / `coursier`。
4. 已安装版本**没有独立清单文件**，用 `<versions>/<sdk>_versions/<sdk>-<version>` 目录 + `<versions>/<sdk>` 符号链接表达（当前版本指向符号链接）。

---

## 2. 目录结构总览

```
vmr-go/
├── main.go                  # 入口（GitTag/GitHash 编译期注入）
├── go.mod                   # Go 1.23；关键依赖：cobra、gopher-lua、bubbletea、
│                            #   mholt/archives、goutils(请求/PTY/终端UI)、go-toml/v2、
│                            #   goquery、gf(gconv/gjson)、resty、gopsutil
├── cmd/vmr/                 # CLI 层
│   ├── main.go
│   └── cli/
│       ├── cli.go           # 根命令 + 全部子命令注册
│       ├── use.go / proxy.go / install_self.go / uninstall_self.go
│       ├── download_threads.go / customed_mirrors.go
│       └── vcli/            # 供脚本调用的纯 CLI 命令
├── internal/
│   ├── cnf/                 # 配置中枢：路径、env、URL、代理/镜像/反代策略
│   ├── utils/               # 基础工具：版本排序、解压、复制、执行、找目录
│   ├── download/            # 下载器（带缓存与校验）
│   ├── luapi/               # Lua 插件系统
│   │   ├── plugin/          #   插件生命周期、下载/更新、版本缓存
│   │   └── lua_global/      #   Lua 全局 API 绑定层（50 个 vmr* 函数）
│   │   └── lua.go           #   空壳
│   ├── installer/           # SDK 安装器
│   │   ├── install/         #   4 种安装方式 + 目录约定
│   │   └── post/            #   SDK 差异化后处理
│   ├── self/                # vmr 自身安装/卸载/更新/旧版本清理
│   ├── shell/               # shell 环境注入（bash/zsh/fish/win/ps1）
│   │   └── sh/              #   各 shell 实现
│   ├── terminal/            # PTY/ConPTY 会话终端
│   │   └── term/            #   PTY 底层实现
│   ├── completions/         # shell 补全脚本生成
│   ├── install/             # ⚠️ 废弃空壳（4 个文件，仅包声明）
│   └── tui/                 # 交互式 TUI
│       ├── table/           #   bubbletea 表格组件
│       ├── cmds/            #   完整交互层（动作分发中枢）
│       └── cliui/           #   简化只读版（供 vcli 用）
├── scripts/                 # 安装脚本（install.sh/ps1、build、签名）
└── docs/                    # 文档、logo
```

---

## 3. 跨模块契约（Rust 重写必须先对齐）

### 3.1 路径约定（internal/cnf/conf.go）

| 函数 | 返回路径 | 说明 |
|---|---|---|
| `GetVMRWorkDir()` | `$HOME/.vmr` | 工作根目录，MkdirAll |
| `GetVMRConfFilePath()` | `~/.vmr/conf.toml` | 配置文件 |
| `GetVersionsDir()` | `$VMR_SDK_INSTALLATION_DIR/versions`，否则 `~/.vmr/versions` | SDK 安装根 |
| `GetCacheDir()` | `<versions 的父目录>/cache`（默认 `~/.vmr/cache`） | 下载缓存 |
| `GetTempDir()` | `~/.vmr/temp` | 解压中转，安装完成后删除 |
| `GetSDKInstallationConfDir()` | `~/.vmr/install_confs` | 安装配置存放（当前基本未用，安装配置改走 Lua） |
| `GetPluginDir()` | `~/.vmr/plugins` | Lua 插件目录 |

已安装版本的目录约定（internal/installer/install/common.go）：
- `<versions>/<sdkName>_versions/` 为某 SDK 的版本根目录（常量 `VersionDirSuffix = "_versions"`）
- 每个版本子目录：`<sdkName>-<versionName>`
- 当前版本符号链接：`<versions>/<sdkName>` → 指向版本子目录（`utils.CreateSymLink`，Windows 用 junction `mklink /j`）

### 3.2 环境变量（conf.go 常量，注意源码拼写）

| 常量 | 值 | 说明 |
|---|---|---|
| `VMRSdkInstallationDirEnv` | `VMR_SDK_INSTALLATION_DIR` | 自定义 SDK 安装根 |
| `VMRHostUrlEnv` | `VMR_HOST` | 版本元数据 host |
| `VMRReverseProxyEnv` | `VMR_REVERSE_PROXY` | 反向代理 |
| `VMRLocalProxyEnv` | `VMR_LOCAL_PROXY` | 本地代理 |
| `VMRDonwloadThreadEnv` | `VMR_DOWNLOAD_THREADS` | 下载线程数（**拼写 Donwload 是源码原样**，须保留） |
| `VMRUseCustomedMirrorEnv` | `VMR_USE_CUSTOMED_MIRRORS` | 是否启用自定义镜像 |
| `VMRAllowNestedSessionsEnv` | `VMR_ALLOW_NESTED_SESSIONS` | 允许嵌套会话 |
| （shell 模块）`VMDisableEnvName` | `VM_DISABLE` | 会话守卫：=111 时跳过环境注入 |
| （installer）`AddToPathTemporarillyEnvName` | `VMR_ADD_TO_PATH_TEMPORARILY` | =1 时临时把 SDK bin 加入当前进程 PATH |
| （zsh hook） | `VMR_CD_INIT` | 首启自执行 cd |
| （win_patch） | `VMR_VERSIONS` | mingw 下的 versions 目录 |
| （goutils） | `GVC_DEFAULT_PROXY` | Fetcher 默认代理回退 |
| （lua req） | `VCOLLECTOR_PROXY` | Lua 层 vmrGetResponse 的代理 |

### 3.3 配置文件格式

**`~/.vmr/conf.toml`**（VMRConf，conf.go）：
```toml
ProxyUri            = ""   # 本地代理
ReverseProxy        = ""   # 反代前缀
SDKIntallationDir   = ""   # 安装根（拼写原样）
VersionHostUrl      = ""   # 元数据 host
ThreadNum           = 1    # 下载线程数
UseCustomedMirrors  = false
AllowNestedSessions = false
GithubToken         = ""
CacheRetentionTime  = 86400  # 秒，0 时默认 1 天
DisableCache        = false
```
⚠️ **Quirk**：struct tag 写的是 `json,toml:"..."`（单 key），Go 解析为单一键 `json,toml`，因此 go-toml/encoding/json 都识别不到，序列化时**回退为 Go 字段名（PascalCase）**。Rust 重写前应抓一份真实 `~/.vmr/conf.toml` 核实键名（大概率是 PascalCase）。[INFERENCE]

**`~/.vmr/customed_mirrors.toml`**：`map[源URL子串]镜像URL子串`。不存在时自动从 `https://proxy.vmr.dpdns.org/proxy/https://raw.githubusercontent.com/gvcgo/vsources/main/mirrors/customed_mirrors.toml` 下载。

### 3.4 远程数据源契约（internal/cnf/common.go）

- 默认 host：`https://raw.githubusercontent.com/gvcgo/vsources/main`（`VMR_HOST` 可覆盖；`DefaultHostUrl`）
- `GetSDKListFileUrl()` → `host + /sdk-list.version.json`
- `GetVersionFileUrlBySDKName(name)` → `host + /{name}.version.json`
- `GetSDKInstallationConfFileUrlBySDKName(name)` → `host + /install/{name}.toml`
- 注意：URL 拼接**不再**自动叠加反代（相关行已注释）
- 反代默认前缀：`https://proxy.vmr.dpdns.org/proxy/`（`DefaultReverseProxy`）

**GetFetcher 请求构造规则（优先级链）**：
1. 先应用镜像替换 `UseCustomedMirrorUrl(dUrl)`（仅 `VMR_USE_CUSTOMED_MIRRORS=true` 时）
2. `localProxy = $VMR_LOCAL_PROXY`；`reverseProxy = GetReverseProxyUri(dUrl, localProxy)`
3. 仅当 `reverseProxy != "" && 镜像未改动URL` 时：`dUrl = reverseProxy + "/" + dUrl`
4. 多线程：仅非 `.json`/`.toml` 结尾的大文件设置线程数（`GetDownloadThreadNum()`）
5. 代理：仅当**非 gitee 且镜像未改动URL**时设 `fetcher.Proxy = localProxy`

**GetReverseProxyUri 规则**：本地代理非空 → 空串（不反代）；URL 含 `gitee.com` → 空串；`$VMR_REVERSE_PROXY` 非空用之；否则 URL 含 `github` → 默认反代。末尾补 `/`。

**镜像替换特殊分支**：URL 以 `https://gradle.org/releases` 开头且镜像值含 `%s` → 从 URL query 取 `version` 参数 `sprintf` 替换；无 version 返回原 URL。

**Fetcher 默认行为（依赖 goutils）**：无自定义 UA、无重试、无超时；代理回退 `GVC_DEFAULT_PROXY`；支持 http/https/socks5；多线程分片下载 `GetMultiPartFile`（`.part%v` 临时文件、`temp_part_xxx` 目录）。

### 3.5 核心数据结构

**版本条目 `lua_global.Item`**（internal/luapi/lua_global/version.go）：
```go
type Item struct {
    Url       string `json:"url"`        // 下载 URL
    Arch      string `json:"arch"`       // amd64 | arm64
    Os        string `json:"os"`         // linux | darwin | windows
    Sum       string `json:"sum"`        // 校验和
    SumType   string `json:"sum_type"`   // sha1 | sha256 | sha512 | md5
    Size      int64  `json:"size"`       // 字节
    Installer string `json:"installer"`  // conda | conda-forge | coursier | unarchiver | executable | dpkg | rpm
    LTS       string `json:"lts"`
    Extra     string `json:"extra"`
}
type SDKVersion []Item
type VersionList map[string]SDKVersion   // map[版本名][]Item
```

**安装配置 `lua_global.InstallerConfig`**（lua_global/installer.go，Lua 侧全局变量名 `ic`）：
```go
type FileItems struct { Windows, Linux, MacOS []string }  // 按平台
type AdditionalEnv struct { Name string; Value []string }  // 相对路径列表
type InstallerConfig struct {
    FlagFiles       *FileItems      // 标志文件（用于找解压后家目录）
    FlagDirExcepted bool
    BinaryDirs      *FileItems      // 各平台 bin 目录（相对安装根）
    BinaryRename    *BinaryRename   // NameFlag→RenameTo
    AdditionalEnvs  AdditionalEnvList
}
```
另有 TOML 版 `install/common.go` 的 `InstallerConfig`（含 `AdditionalEnv.Value string`、`Version string` 条件），**当前是死代码**（安装配置已改走 Lua）。

**`VMRConf`**：见 3.3。

### 3.6 命令契约

- `vmr use <sdk>@<version>`（CLI）与 TUI 等价动作
- `vmr uninstall <sdk>@<version>|all`
- 会话模式：`vmr use -s` 安装后进入交互子 shell（PTY），退出时 `VM_DISABLE=111`
- 卸载自身：`vmr Uins`（uninstall-self，供脚本调用），随后删除 `~/.vmr`
- 更新脚本 URL：`https://scripts.vmr.dpdns.org`（unix）`/windows`（powershell）

---

## 4. 模块详解

### 4.1 cmd/vmr —— CLI 层（纯命令壳）

**入口 main.go**：import `cnf` 副作用初始化全局配置；`GitTag/GitHash` 编译期注入（ldflags）；调 `cli.New(tag, hash).Run()`。

**cli.go**：cobra 根命令 `vmr`。无参数时 `Run` 回调 → `cmds.NewTUI().ListPluginName()`（进入 TUI）。注册全部子命令（别名在括号中）：

| 命令 | 别名 | 功能 | 底层调用 |
|---|---|---|---|
| `version` | `v` | 显示 `gitTag(gitHash[:7])` | — |
| `set-proxy` | `sp` | 设置本地代理 | `cnf.DefaultConfig.SetProxyUri` |
| `set-reverse-proxy` | `sr`/`srp` | 设置反代 | `cnf.DefaultConfig.SetReverseProxy` |
| `use` | `u`/`h` | 安装并切换版本（flags: `-E` 启用项目锁、`-s` 会话、`-l` 锁版本、`-c` conda 安装） | installer |
| `install-self` | `i`/`is` | 自安装（供脚本） | `self.InstallSelf` |
| `uninstall-self` | `Uins` | 自卸载（仅脚本） | `self.RemoveCurrentVersion` |
| `set-download-threads` | `sdt`/`st` | 设置线程数 | `cnf.DefaultConfig.SetDownloadThreadNum` |
| `toggle-customed-mirrors` | `tcm`/`tm` | 切换镜像开关 | `cnf.DefaultConfig.ToggleUseCustomedMirrors` |
| `show` | `S` | 列出可用 SDK | `cliui.NewSDKSearcher().Show()` |
| `search` | `s` | 查某 SDK 版本（`-c` conda） | `cliui.NewVersionSearcher()` |
| `local` | `l` | 显示已装版本 | `cliui.NewLocalInstalled()` |
| `installed-sdks` | `in` | 列出已装 SDK | `cliui.NewSDKSearcher().PrintInstalledSDKs()` |
| `installed-info` | `ii` | 每个 SDK 已装版本 | `cliui` |
| `uninstall` | `uni`/`r` | 卸载（`@all` 全卸） | `installer.NewIVFinder` / `NewInstaller().Uninstall()` |
| `add-completions` | `ac` | 生成补全 | `completions.AddCompletionScriptToShellProfile` |
| `nested-sessions` | `ns` | 允许嵌套会话开关 | `cnf.DefaultConfig.ToggleAllowNestedSessions` |
| `is-session-mode` | `ism` | 查询会话模式（读 `VM_DISABLE`） | shell/sh 常量 |
| `update-plugins` | `up` | 更新 Lua 插件 | `plugin.UpdatePlugins` |

**use.go 逻辑细节**：`-E` → `installer.NewVLocker().HookForCdCommand()`；否则解析 `args[0] = plugin@version`；`-c` 时构造 `Item{Arch: runtime.GOARCH, Os: runtime.GOOS, Installer: Conda}` 且 `DisableEnvs()`；模式：`-s`→ModeSessionly，`-l`→ModeToLock，默认 ModeGlobally；conda 安装后打印符号链接路径提示手动加 PATH。

### 4.2 internal/cnf —— 配置中枢

- **conf.go**：常量、`VMRConf`、`NewVMRConf()`（Load conf.toml → **回写环境变量**，有副作用）、全部目录函数、Setter/Toggle。
- **common.go**：URL 拼接、反代/镜像/代理策略、`GetFetcher`、`GetGithubToken`、`GetCacheRetentionTime`（0→86400）、`GetCacheDisabled`。
- 关键设计：**conf.toml 载入后写 env，下载/URL 逻辑只读 env → 环境变量是运行时权威**。`NewVMRConf` 仅在非空/满足条件时回写：`SDKIntallationDir`→`VMR_SDK_INSTALLATION_DIR`、`VersionHostUrl`→`VMR_HOST`(去尾 `/`)、`ProxyUri`→`VMR_LOCAL_PROXY`、`ThreadNum>1`→`VMR_DOWNLOAD_THREADS`、镜像开关总是写、`ReverseProxy`→`VMR_REVERSE_PROXY`、`AllowNestedSessions` 仅 true 时写。
- Rust 重写注意：保留拼写 quirk（`SDKIntallationDir`、`VMRDonwloadThreadEnv`）、保留 env 为权威来源、conf.toml 键名待核实。

### 4.3 internal/utils —— 基础工具库（无状态）

| 文件 | 功能 |
|---|---|
| `sort_versions.go` | **版本解析+排序**：正则解析 `Major/Minor/Patch/Build/Beta/RC`；beta/rc 缺失时置 `math.MaxInt` 哨兵（**稳定版 > beta > rc**）；`SortVersions` 降序、`SortVersionAscend` 升序，逐级比较，解析失败回退字符串比较。⚠️ 正则中 `.` 未转义（重写时注意保留行为或修正） |
| `shell.go` | 平台路径分隔符拼接 `JoinPath`（`;`/`:`）；`DNForAPTonLinux` 探测 apt/dnf/yum；`MoveFileOnUnixSudo`（sudo mv）；`CreateSymLink`（win 用 `mklink /j` junction）；`IsMingWBash`；`ConvertWindowsPathToMingwPath` |
| `open_url.go` | `OpenURL`：win `cmd start` / linux `xdg-open` / darwin `open` |
| `find_dir.go` | `HomeDirFinder`：递归 DFS 在解压产物中找家目录（按 flagFiles 名字匹配），跳过 `__MACOSX` |
| `file_*.go` | 平台隔离 `GetFileLastModifiedTime`（秒级） |
| `extractor.go` | 解压总入口 `Extract`：**优先系统命令**（`.zip`→`unzip` 或 powershell `Expand-Archive`；`.tar*`→`tar -xf`），失败回退 Archiver；`handleMultiCompress` 对解压出的 `.zip` 递归再解压 |
| `extract/extractor.go` | 新式解压（mholt/archives）：支持 zip/tar/tar.gz/bz2/xz/gz；`Decompressor`（gz 单文件先解到 temp 再识别）+ `Extractor`；压缩单文件可设可执行位/重命名 |
| `exec.go` | `SysCommandRunner`：exec.CommandContext，win 前插 `/c` 走 cmd，unix `FlushPathEnv`；可收集输出/取消 |
| `copy.go` | `CopyFile/CopyAFile/CopyDirectory`：**无进度无并发**，顺序 io.Copy；符号链接重建；递归跳 `.Trashes/.DS_Store` |
| `check.go` | `IsMinicondaInstalled` / `IsCoursierInstalled`（探测 conda/cs 命令） |

### 4.4 internal/luapi —— Lua 插件系统（扩展性核心）

**加载与执行架构**：
```
lua_global.NewLua() 创建 gopher-lua LState，init() 注册 50 个 vmr* 全局函数
  → plugin.Plugin.LuaDo() DoFile(插件 .lua)
  → 脚本定义全局：sdk_name / plugin_name / crawl() / ic(InstallerConfig) / postInstall / install 等
  → Plugin.Load() 读元数据并校验 crawl 存在
  → GetSDKVersions() 调 crawl 抓版本 → 按 Item.Os/Arch 过滤 → 缓存到 <cache>/<plugin_name>/<plugin_name>.versions.json
  → 安装时 installer 经 GetInstallerConfig() 读回 ic
```

**plugin/ 包**：
- `plugin.go`：`Plugin`（含 Result）、`LuaDo/Load/GetSDKVersions/GetInstallerConfig/GetCustomedInstallHandler/GetPostInstallHandler/Close`、`GetSortedVersions`（转 tui table.Row）、`GetVersion`、`GetLatestVersion`（取排序后最后一个）。
- `plugins.go`：`Plugins` 扫描插件目录加载全部 `*.lua` → `map[plugin_name]*Plugin`；`GetPlugin/GetPluginBySDKName/GetPluginList/GetPluginSortedRows`；**插件目录不存在时自动触发 `Update()` 下载**。
- `fromlua.go`：`LuaConfItem` 类型 + 常量 `SDKName/PluginName/PluginVersion/Prequisite/Homepage/Crawler/PostInstall/CustomedInstall/InstallerConfig`；`GetLuaConfItemString` 从 Lua 全局读字符串或调用函数取值。
- `download.go`：`UpdatePlugins()`：从 `https://github.com/gvcgo/vmr_plugins/archive/refs/heads/main.zip` 下载解压 → 按 `go.lua`/`LICENSE` 标记找目录 → 复制 `*.lua` 到插件目录 → 用 gh API 列文件写 `plugins.json`。
- `sdk.go`：`Versions` 结构体（疑似与 Plugin 平行的旧版/冗余实现，含版本缓存），`PrequisiteHandler` 已注册但调用被注释（废弃）。
- `lua.go`（luapi 根）：**空壳**（14B，仅 package 声明）。

**lua_global/ —— Lua 全局 API 绑定（50 个，全部 `vmr` 前缀）**：

| 组 | 函数 | 说明 |
|---|---|---|
| req | `vmrGetResponse(url, timeout, headers)` | GET 返回响应字符串；代理读 `VCOLLECTOR_PROXY` |
| goquery | `vmrInitSelection/vmrFind/vmrEq/vmrAttr/vmrText/vmrEach` | HTML 解析（Selection 以 LUserData 传递） |
| gjson | `vmrInitGJson/vmrGetString/vmrGetInt/vmrGetByKey/vmrMapEach/vmrGetByIndex/vmrSliceEach` | JSON 解析（gjson.Json 以 LUserData 传递） |
| utils | `vmrGetOsArch/vmrRegexpFindString/vmrHasPrefix/vmrHasSuffix/vmrContains/vmrTrimPrefix/vmrTrimSuffix/vmrTrim/vmrTrimSpace/vmrToLower/vmrSplit/vmrSprintf/vmrUrlJoin/vmrPathJoin/vmrLenString/vmrGetOsEnv/vmrSetOsEnv/vmrExecSystemCmd/vmrReadFile/vmrWriteFile/vmrCopyFile/vmrCopyDir/vmrCreateDir/vmrRemoveAll` | 通用工具 |
| version | `vmrNewVersionList/vmrAddItem/vmrMergeVersionList` | 版本列表构建（Item/VersionList，见 3.5） |
| github | `vmrGetGithubRelease(repo, tagFilter, versionParser, fileFilter, archParser, osParser, installerGetter)` | 拉 GitHub releases，回调 6 个 Lua 函数过滤生成 VersionList |
| installer_config | `vmrNewInstallerConfig/vmrAddFlagFiles/vmrEnableFlagDirExcepted/vmrAddBinaryDirs/vmrAddAdditionalEnvs` | 构建 `ic`（全局变量名 `ic`） |
| conda | `vmrSearchByConda` | 执行 `conda search` 命令解析输出（`CondaSearchCommand`） |
| proxy | `vmrGetProxy()` | 从 cnf 配置读代理，返回 scheme/host/port |
| extractor | `vmrUnarchive(src, dst, compressedSingleFileName, isCompressedSingleExecutable)` | 委托 utils/extract 解压 |

`gh/gh.go`（非 Lua 绑定）：`Gh` GitHub REST API 客户端：`GetReleases`（分页）、`GetFileList`（contents API）；`GetDefaultReadOnly` 解码内置只读 token；定义 `Asset/ReleaseItem/ReleaseList/RepoFile`。

Rust 重写要点：Lua 运行时（mlua/rhai 或内嵌 Lua）必须提供**同名同语义**的全局函数；`Item` 的 Os/Arch 过滤、版本缓存文件位置与格式（`<cache>/<plugin>/<plugin>.versions.json`）、插件自动更新（目录缺失时）都要保留。

### 4.5 internal/installer —— SDK 安装器（核心调度）

**installer.go —— 总调度**：
- `InvokeMode`：`ModeGlobally`（全局，建符号链接+写 shell 环境）/ `ModeSessionly`（会话，进子 shell）/ `ModeToLock`（写项目锁 + 子 shell）。
- `SDKInstaller` 接口：`Initiate / SetInstallConf / GetInstallDir / GetSymbolLinkPath / Install`。
- `NewInstaller` 按 `Item.Installer` 分派：`conda|conda-forge`→`NewCondaInstaller`；`coursier`→`NewCoursierInstaller`；`executable|dpkg|rpm`→`NewExeInstaller`；默认→`NewArchiverInstaller`。然后 `plugin.NewVersions(pluginName).GetInstallerConfig()` 读回 `ic` 并 `SetInstallConf`。
- `Install()` 流程：
  1. 前置：conda 系 → `CheckAndInstallMiniconda()`；coursier → `CheckAndInstallCoursier()`。
  2. 未安装则 `sdkInstaller.Install()`，然后查 `post.PostInstallHandlers[PluginName]` 做后处理。
  3. 全局模式：`CreateSymlink()`（删旧链→`utils.CreateSymLink(installDir, symbolPath)`）→ `SetEnvGlobally()`（按 `ic.BinaryDirs` 按平台取 bin 目录 + `ic.AdditionalEnvs` 收集环境，经 `Shell.SetPath/SetEnv` 写入）→ `AddEnvsTemporarilly()`（`VMR_ADD_TO_PATH_TEMPORARILY=1` 时临时加 PATH）。
  4. 会话/锁模式：`ModeToLock` → `writeLockFile()`（`NewVLocker().Save(plugin, version)` 写 `.vmr.lock`）；`RemoveGlobalSDKPathTemporarily`（把符号链接路径从 PATH 摘除）；`Setenv(VMR_ADD_TO_PATH_TEMPORARILY, "1")`；`AddEnvsTemporarilly`；`terminal.RunTerminal()` 进入交互子 shell。
- `CollectEnvs(basePath)`：按 GOOS 选 `BinaryDirs.{MacOS|Linux|Windows}`，空则默认 `[basePath]` 本身；`AdditionalEnvs` 逐条拼路径，仅存在的路径入结果；`NoEnvs` 时返回空（conda 安装用）。
- `Uninstall()`：删 `<versions>/<sdk>_versions/<sdk>-<version>` 目录；若卸载的是当前版本还删符号链接 + `UnsetEnv()`（从 shell 配置移除 PATH/env）。
- 常量：`AddToPathTemporarillyEnvName = "VMR_ADD_TO_PATH_TEMPORARILY"`。

**locker.go —— 项目版本锁**：`VLocker`：`FindLockerFile` 从当前目录**向上查找** `.vmr.lock`；`Load/Save`（JSON 或 `name@version` 格式）；`HookForCdCommand`（`vmr use -E` 的 cd hook 端点）；`RemoveGlobalSDKPathTemporarily`。

**installed.go —— 已安装版本发现**：`IVFinder.FindAll()`：读符号链接 → 当前版本（`<plugin>-` 前缀）；列版本根目录下 `<plugin>-*` 子目录 → 已装列表。`UninstallAllVersions` 删整个版本根目录 + UnsetEnv。

**cached.go**：`CachedFileFinder`：删 `cache/<plugin>[/<version>]`。

**prequisite.go**：`CheckAndInstallMiniconda` / `CheckAndInstallCoursier`。

**search_by_conda.go**：`CondaSearcher`：`GetCondaPlatform`（win 用 `win-64` 等）、`CondaSearchCommand`、解析输出得版本列表（供 `vmr search -c` 与 TUI conda 搜索）。

**install/ —— 四种安装方式**：
- `unarchiver.go`（ArchiverInstaller，**默认**）：缓存下载 → `Extract` 解压到 `~/.vmr/temp` → `HomeDirFinder` 找家目录 → `CopyDirectory` 到 `<versions>/<sdk>_versions/<sdk>-<version>`。
- `executable.go`（ExeInstaller）：miniconda/vscode/erlang/elixir/独立可执行文件安装（exe/deb/rpm/sh，直接放安装目录，Windows 处理 `.exe`）。
- `coursier.go`（CoursierInstaller）：`cs install --install-dir`（Scala 系）。
- `conda.go`（CondaInstaller）：`conda create --prefix <installDir>`（conda 系，环境变量由用户手动加）。
- `common.go`：目录约定常量 `VerisonDirPattern("%s%s")/VersionDirSuffix("_versions")/VersionInstallDirPattern("%s-%s")`、`GetSDKVersionDir`、`IsSDKInstalledByVMR`；TOML 版 InstallerConfig（死代码）。
- `rustup.go`、`compile.go`：**空壳**（仅注释）。

**post/ —— SDK 差异化后处理**（`PostInstallHandlers[pluginName]` 全局注册表，post.go）：
- `zig.go`/`upx.go`：unix `chmod +x`。
- `rust.go`：复制 rustup-init 到 `Library/bin/rustup`（源码含 `chmox` 拼写错误）。
- `php.go`：把 php.ini 的 opcache `zend_extension` 改为绝对路径（win/unix）。
- `moonbit.go`：chmod、下载 `core-latest.tar.gz` 并 `moon bundle`。
- `clojure.go`：unix 移 jar 到 libexec、改 clojure/clj 脚本；win 生成 ps1。
- `bun.go`：生成 `bunx`（win 复制 / unix 符号链接）。

### 4.6 internal/download —— 下载器

`sdk_file.go`：`SDKFileDownloader`：缓存路径 `<cache>/<sdk>/<v>/<file>`；`Download` 幂等（已存在跳过）；把校验和/大小注入 Fetcher。

### 4.7 internal/self —— vmr 自身生命周期

- `install.go`：`InstallSelf()`：`DetectAndRemoveOldVersions` → 复制自身到 `~/.vmr/<binName>` → `WriteVMEnvToShell`（+win `SetPath`）→ `SetUpdateScript/SetUninstallScript/AddCustomedSourceCmd` → **交互式**设置 SDK 安装目录并 Save。
- `update_script.go`：unix 写 `~/.vmr/vmr-update`（`curl -sSf https://scripts.vmr.dpdns.org | sh`）；win 写 `.bat`（`powershell irm https://scripts.vmr.dpdns.org/windows | iex`）+ mingw `.sh` 包装。
- `uninstall_script.go`：`~/.vmr/vmr-uninstall`：`cd ~; vmr Uins; rm -rf <workdir>`；win 版 `.bat`（rmdir /s /q）+ mingw 包装。
- `source_script.go`：仅 unix：向 shell 配置追加 `alias svmr="export VM_DISABLE='' && source <profile>"`（手动绕过会话守卫重载环境）。
- `old_versions.go`：`DetectAndRemoveOldVersions`（检测旧 `~/.vm` 目录，交互确认后清理）；`RemoveCurrentVersion`（删 versions/cache/workdir + 清理 shell 配置）。

### 4.8 internal/shell —— shell 环境注入

- `shell.go`：`Sheller` = `sh.Sheller` + `SetEnv/UnsetEnv/SetPath/UnsetPath`。
- `sh/`：
  - `shell.go`：接口；`VMDisableEnvName="VM_DISABLE"`、`VMEnvFileName="vmr"`、ModePerm 0644；`UpdateVMRShellFile` 按 `# cd hook start/end` 正则替换；`FormatPathString`（`$HOME`→`~`）。
  - `bash.go`：`~/.bashrc` + `~/.vmr/vmr.sh`；写 cd hook（export PATH）+ source 块。
  - `zsh.go`：`~/.zshrc`；cd hook（`vmr use -E`，`VMR_CD_INIT` 首启自执行）；`if [ -z "$VM_DISABLE" ]; then . vmr.sh; fi`。
  - `fish.go`：`~/.config/fish/config.fish` + `~/.vmr/vmr.fish`；`fish_add_path` + `_vmr_cdhook`（`--on-variable=PWD`）；`set --global`。
- `win.go`：**注册表 `HKCU\Environment` 注入**（PATH 用 `SetExpandStringValue`），`SendMessageTimeoutW` + `WM_SETTINGCHANGE` 广播；`PowershellHook` 写 `~/Documents/WindowsPowerShell/profile.ps1`（cdhook + vmrsource + `Set-Alias`）；`VMEnvConfPath` 返回空（设计）。
- `win_patch.go`：Windows/Mingw bash 修补：注册 `VMR_VERSIONS` 环境；PATH 项写 `~/.bashrc`（`MingwBashExportPattern`）；`VmrMingwBashCdHook` 注入 mingw `.bashrc`；`VersionsDir↔%VMR_VERSIONS%` 替换。

### 4.9 internal/terminal —— PTY/ConPTY 会话终端

- `terminal.go`：`PtyTerminal.Run`：设 `VM_DISABLE=111` 后调 `Terminal.Record`；`ModifyPathForPty`（从 PATH 移除当前 SDK 符号链接路径）；`RunTerminal` 起普通交互子 shell。**不生成 shell 包装**，仅通过 `VM_DISABLE` 与 shell 模块联动。
- `term/`：`Terminal` 接口（`Record(command, envs...)/Size()`）；`term_unix.go`（`sh -c` + pty.Start、SIGWINCH resize、stdin raw、200ms stdout 兜底超时）；`term_win.go`（ConPTY 双向复制）；`copy.go`（unix stdin→master，pipe+select）；`fdset/`（平台 FD_SET/select 封装）；`sys_windows.go`/`pty_windows.go`（ConPTY 底层：CreatePseudoConsole/Resize、CreateProcess 等）。

### 4.10 internal/completions —— 补全

`completions.go`：执行 `vmr completion <shell>` 捕获输出 → 写 `~/.vmr/vmr_completions.ps1` 或 `.sh` → 把 `# VMR Completions` 块追加到 shell 配置（powershell `Import-Module` / 其它 `. source`）。

### 4.11 internal/tui —— 交互式 TUI

分层：`table/`（bubbletea 表格组件）→ `cmds/`（完整交互+动作分发）→ `cliui/`（只读简化版，供 vcli 脚本用）。

**交互模型**：表格选中行通过 `row[0]`（plugin name）传递；动作键回调设置 `List.NextEvent` 字符串常量后 `tea.Quit` 退出，由外层 `cmds` 层 switch 分发动作。数据来自 `plugin.Plugins/Versions`（Lua），动作回调 `installer.NewInstaller().Install()/Uninstall()`。

**按键映射（cmds/）**：
- SDK 列表（list.go）：`o` 开首页（`utils.OpenURL(row[1])`）、`s` 查版本、`l` 本机已装、`r` 移除所有已装、`c` 清缓存、`w` 显示 VMR 已装 SDK。
- 版本列表（search.go）：`i` 全局安装、`s` 会话安装、`l` 锁定、`b` 返回；selected 去 `-lts` 后缀。
- 本地列表（local.go）：`c` 清该版本缓存、`r` 移除、`b` 返回、`l` 锁定、`u` 全局切换、`s` 会话切换；当前版本标 `<current>`。
- 表格导航（table/model.go）：`↑/k` `↓/j` `b/pgup` `f/pgdn/space` `u/ctrl+u` `d/ctrl+d` `home/g` `end/G`；`enter` 搜索/切换焦点、`tab` 焦点切换、`esc/ctrl+c` 退出。

`cliui/` 三个文件为简化版：`RegisterKeyEvents` 基本为空实现（纯展示），供 `show/search/local/installed-*` 输出用；`sdk_list.go` 还负责 `GetSDKInstalledByCondaForge`（读 versions 目录补 conda 装的）。

### 4.12 空壳/废弃文件清单（Rust 重写直接丢弃）

| 文件 | 状态 |
|---|---|
| `internal/install/`（installer.go/locker.go/prequisite.go/download.go） | 全部空壳/半成品 |
| `internal/luapi/lua.go` | 空壳（14B） |
| `internal/installer/install/rustup.go`、`compile.go` | 空壳（仅注释） |
| `internal/luapi/plugin/sdk.go` | 疑似旧版冗余（与 plugin.go 平行），PrequisiteHandler 已废弃 |
| `internal/installer/install/common.go` 的 TOML InstallerConfig | 死代码 |

---

## 5. 模块依赖关系

```
cnf  ←─ utils  ←─ download
  ↑       ↑
  ├───────┴──── luapi (lua_global → plugin)
  │                    ↑        ↑
  │              installer ────┘  （install/ 四种安装器、post/ 后处理）
  │                 ↑  ↑
  │            shell  terminal  ←  self / completions
  │                 ↑
  └─────────────── tui (table → cmds → cliui)
                        ↑
                    cmd/vmr (cli → vcli) ← 根入口 main
```

- 无外部依赖的叶子：`cnf`（仅依赖第三方库）、`utils`、`download`（依赖 cnf）。
- `luapi` 依赖 utils/download/cnf；`installer` 依赖 luapi/cnf/utils/shell/terminal；`tui` 依赖 installer/luapi/cnf；`cli` 依赖全部。
- **建议 Rust 重写顺序**（严格按依赖）：① cnf → ② utils → ③ download → ④ luapi（lua 绑定 + 插件）→ ⑤ installer（含 install/post）→ ⑥ shell + terminal → ⑦ self + completions → ⑧ tui → ⑨ cli/main。①②③④ 是可独立验证的核心骨架；⑤⑥ 是功能主干；⑧⑨ 最后。

---

## 6. Rust 重写通用注意点

1. **保留所有 Quirk/拼写**：`SDKIntallationDir`、`VMRDonwloadThreadEnv`、`chmox`（rust post）、`VerisonDirPattern`——涉及磁盘路径/env/脚本输出，改拼写会破坏兼容。
2. **环境变量是配置运行时权威**（conf.toml 载入后回写 env，逻辑只读 env）。
3. **反代/镜像/本地代理优先级**：镜像替换先于反代；仅镜像未改 URL 才叠加反代；gitee 不反代也不用本地代理；本地代理优先于反代。
4. **多线程下载仅限大文件**（非 json/toml）。
5. **已安装状态无清单文件**，靠 `<versions>/<sdk>_versions/<sdk>-<version>` 目录 + `<sdk>` 符号链接（win junction）表达——重写必须保留此磁盘契约。
6. **会话模式契约**：`VM_DISABLE` 守卫、`VMR_ADD_TO_PATH_TEMPORARILY`、进入子 shell（PTY）、`vmr use -E` cd hook 与 `.vmr.lock`。
7. **Lua 插件是数据源核心**：50 个 `vmr*` 全局函数必须同名同语义；插件目录缺失自动更新；版本缓存 `<cache>/<plugin>/<plugin>.versions.json`。
8. **版本排序语义**：稳定 > beta > rc（beta/rc 缺失用最大整数哨兵），逐级 Major→Minor→Patch→Build→Beta→RC 比较。
9. **TUI 可最后重写**（可用 ratatui 替代 bubbletea），但按键→动作映射（4.11）与 `cmds/` 状态机必须保留。
10. **conf.toml 键名待核实**：源码 struct tag 异常（`json,toml:` 单 key），真实文件可能是 PascalCase 键。[INFERENCE]

---

*文档生成日期：2026-08-11。来源：vmr-go 源码只读分析 + readme/docs。*
