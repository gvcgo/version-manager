# vmr Rust 重写计划（plan.md）

> 依据：`vmr-go/description.md`（模块功能说明书）+ 用户 9 条重写要求。
> 目标：用 Rust 将 vmr-go 重写为 `vmr-rust`，仅 CLI（无 TUI）。
> 状态：规划稿，待用户确认后按阶段实施。

---

## 0. 重写要求（用户）

1. **不再从 `github.com/gvcgo/vsources` 获取版本信息**，全部改为 Lua 插件实时获取；
2. **不再使用 Miniconda/conda 安装任何软件**，参考 pixi 实现「从 conda 源安装软件到 vmr 目录」；
3. GitHub Repo Release 的版本信息获取必须用 GitHub API；
4. conda 的版本信息获取同第 2 条机制（查 conda 源）；
5. 独立 Installer 模块，处理不同类型文件安装（压缩包、可执行文件等）；
6. 独立环境变量处理模块；
7. 各种 shell 的 hook，支持自动切换 SDK 版本；
8. 独立请求模块、下载模块（多线程下载、代理设置）；
9. 只实现命令行，无 TUI。

---

## 1. 现状分析与关键结论（来自对 vmr-go 源码的核实）

| 事项 | 现状（Go） | 重写处理 |
|---|---|---|
| vsources 版本信息 | `cnf.GetSDKListFileUrl/GetVersionFileUrlBySDKName/GetSDKInstallationConfFileUrlBySDKName` 三个函数**无任何调用点**（死代码） | 直接移除；SDK 列表来自插件目录扫描，版本来自 Lua `crawl()` |
| GitHub 版本获取 | `vmrGetGithubRelease` → `gh.NewGh`（GitHub REST API，分页拉 `/repos/{repo}/releases`，内置只读 token） | 保留该机制，Rust 侧自写轻量 GitHub API 客户端（reqwest，2 个端点 + 分页 + token） |
| conda 安装 | `CondaInstaller` 执行 `conda create --prefix`；前置检查强制装 Miniconda；版本查询执行 `conda search` 命令 | 全部移除；改自研 conda 源客户端（见 §5） |
| 插件机制 | 插件目录 `~/.vmr/plugins/*.lua`，全局变量 `sdk_name/plugin_name/plugin_version/homepage/ic` + `crawl()/postInstall()/install()`；50 个 `vmr*` 全局函数；插件目录缺失自动从 `gvcgo/vmr_plugins` 下载 | **完整保留**（现有插件必须原样可跑）：Lua 运行时 + 50 个同名同语义绑定 |
| 插件常量缺口 | 插件引用的 `vmrInstallerUnarchiver` / `vmrInstallerExecutable` 等全局常量 Go 侧**未注册**（nil → 默认落 unarchiver） | Rust 侧**显式注册**这些常量（`vmrInstallerUnarchiver`/`vmrInstallerExecutable`/`vmrInstallerConda`/`vmrInstallerCoursier`…） |
| TUI | `internal/tui/`（table/cmds/cliui 三层） | 不实现；`show/search/local/installed-*` 改为文本表格输出 |
| 磁盘契约 | `<versions>/<sdk>_versions/<sdk>-<version>` 目录 + `<versions>/<sdk>` 符号链接（win junction）表达已装/当前版本，无清单文件 | 必须保留（与旧安装兼容） |
| 配置 | `~/.vmr/conf.toml`（PascalCase 键，因 struct tag 异常 `json,toml:`）、`~/.vmr/customed_mirrors.toml`；env 是运行时权威 | 保留文件位置与键名（待实测确认）；配置统一由 core 层读写，env 回写机制保留 |

---

## 2. 总体架构（Cargo workspace）

```
vmr-rust/
├── Cargo.toml                  # workspace
├── crates/
│   ├── vmr-utils/              # 基础工具：版本解析排序、解压、复制、找目录、命令执行
│   ├── vmr-core/               # 路径约定、VMRConf、镜像/反代/代理策略、URL 拼接
│   ├── vmr-net/                # 请求模块 + 下载模块（多线程分片、代理、镜像替换）
│   ├── vmr-conda/              # conda 源客户端（repodata、.conda/.tar.bz2 提取、安装到 prefix）
│   ├── vmr-lua/                # Lua 运行时 + 插件生命周期 + 50 个 vmr* 绑定（含 github/conda 桥）
│   ├── vmr-installer/          # 安装器：unarchiver / executable / conda / coursier / dpkg/rpm + post 后处理
│   ├── vmr-env/                # 环境变量：收集（BinaryDirs/AdditionalEnvs）、全局设置、临时注入
│   ├── vmr-shell/              # shell hook：bash/zsh/fish/powershell/git-bash 环境注入 + cd hook
│   ├── vmr-pty/                # 会话模式交互子 shell（PTY/ConPTY）
│   ├── vmr-self/               # vmr 自身：install-self / uninstall-self / 更新脚本 / 旧版本清理
│   ├── vmr-completions/        # shell 补全脚本生成
│   └── vmr-cli/                # 命令行入口（clap），全部子命令，文本输出
└── plugins/                    # （可选）插件样例
```

依赖方向（与 Go 侧一致，纯单向）：
```
vmr-utils ← vmr-core ← vmr-net ← vmr-lua ← vmr-installer ← vmr-cli
                              ↘ vmr-conda ↗          ↑
                                         vmr-env ←───┘
                                         vmr-shell / vmr-pty / vmr-self / vmr-completions
```

### 关键第三方依赖选型（建议）

| 用途 | crate | 说明 |
|---|---|---|
| CLI | `clap` | 子命令 + 别名 + flags |
| 异步运行时 | `tokio` | 下载/请求/PTY |
| HTTP | `reqwest` | 代理（http/socks5）、UA、重试 |
| 多线程下载 | 自研（reqwest Range 分片） | 仿 goutils `GetMultiPartFile`：`.part%v` 临时文件 + 合并 |
| Lua 运行时 | `mlua`（vendored lua5.4） | 兼容 gopher-lua 的 Lua 5.1 语法需开启 `lua54` feature 并注意 API 差异；若需严格 5.1 语法可换 `rlua`/`mlua lua51` [待定] |
| 压缩解压 | `tar`、`zip`、`flate2`、`bzip2`、`xz2`、`zstd` | 对应 mholt/archives 覆盖格式；`gz` 单文件解压到 temp 再识别 |
| TOML | `toml`（serde） | conf.toml、customed_mirrors.toml |
| JSON | `serde_json` | 版本元数据、GitHub API |
| 正则 | `regex` | 版本解析、Lua `vmrRegexpFindString` 桥 |
| PTY | `portable-pty` | 跨平台 PTY/ConPTY（替代 Go 的 creack/pty） |
| 符号链接 | `std::os` + win junction（`mklink /j` 子进程） | 保留 Go 行为 |
| conda 生态 | `rattler_conda_types`、`rattler_repodata_gateway`、`rattler_package_streaming`、（可选 `rattler_solve`） | pixi 同款；见 §5 |

---

## 3. 模块设计（对应 9 条要求的落地）

### 3.1 vmr-core —— 配置中枢（无依赖叶子）

- 路径函数：`work_dir/conf_path/versions_dir/cache_dir/temp_dir/plugin_dir`（`~/.vmr`，`VMR_SDK_INSTALLATION_DIR` 覆盖 versions/cache）。
- `VMRConf`：`conf.toml` 读写；**保留 PascalCase 键名与 `SDKIntallationDir` 拼写 quirk**；Load 后回写 env（env 为运行时权威）。
- 策略函数：`GetReverseProxyUri`（本地代理优先、gitee 不反代、github 默认反代 `https://proxy.vmr.dpdns.org/proxy/`）、`UseCustomedMirrorUrl`（含 gradle `%s` 特殊分支）、`GetDownloadThreadNum`。
- **移除**：vsources 三个 URL 函数（死代码）；`install_confs` 目录不再创建（安装配置全走 Lua `ic`）。
- `customed_mirrors.toml` 缺失时仍从 `https://proxy.vmr.dpdns.org/proxy/https://raw.githubusercontent.com/gvcgo/vsources/main/mirrors/customed_mirrors.toml` 拉取（镜像表非版本信息，保留）【或内置默认镜像表，待定】。

### 3.2 vmr-utils —— 基础工具

- `semver.rs`：移植 Go 排序语义——正则解析 `Major/Minor/Patch/Build/Beta/RC`，beta/rc 缺失置最大整数哨兵（稳定 > beta > rc），降序/升序逐级比较；解析失败回退字符串比较。⚠️ 不直接用 `semver` crate（prerelease 语义不匹配）。
- `extract.rs`：多格式解压（zip/tar/tar.gz/bz2/xz/gz），优先系统命令（`unzip`/`tar -xf`）失败回退库实现；`gz` 单文件先解到 temp；压缩单文件可设可执行位/重命名（对应 `vmrUnarchive`）。
- `copy.rs`：顺序复制（无进度无并发），符号链接重建，跳 `.Trashes/.DS_Store`。
- `find_dir.rs`：DFS 找解压后家目录（flagFiles 匹配，跳 `__MACOSX`）。
- `exec.rs`：命令执行封装（win 走 cmd `/c`）。
- `symlink.rs`：`CreateSymLink`（win 用 `mklink /j` junction）。

### 3.3 vmr-net —— 请求 + 下载（要求 8）

- `fetcher.rs`：reqwest 封装，默认无 UA/重试/超时（对齐 Go 行为，可配置）；代理：`VMR_LOCAL_PROXY` 优先，回退 `GVC_DEFAULT_PROXY`，支持 http/https/socks5。
- `download.rs`：**多线程分片下载**——HEAD 拿 `Content-Length` → 按线程数切 Range → 并行拉 `.part%v` → 合并；仅大文件（非 `.json`/`.toml`）启用；校验和（sha1/sha256/sha512/md5）+ 大小注入；`<cache>/<sdk>/<v>/<file>` 幂等缓存。
- 镜像/反代/代理优先级链（对齐 Go）：镜像替换先于反代；仅镜像未改 URL 才叠加反代；gitee 不反代也不用本地代理。
- `github.rs`：GitHub REST API 客户端（`/repos/{repo}/releases?page=N` 分页、contents API 列文件）；token：配置 `GithubToken` 优先，否则内置只读 token；供 Lua 桥 `vmrGetGithubRelease` 与 `update-plugins`（列插件文件）使用（要求 3）。

### 3.4 vmr-lua —— 插件系统（要求 1、3、4 的核心）

- **运行时**：mlua LState；`init()` 注册 **50 个 `vmr*` 全局函数**（清单见 `description.md` §4.4，签名逐一对齐）：
  - req：`vmrGetResponse(url, timeout, headers)`（代理读 `VCOLLECTOR_PROXY`）
  - goquery：`vmrInitSelection/vmrFind/vmrEq/vmrAttr/vmrText/vmrEach`（HTML 解析，UserData 传递 Selection）
  - gjson：`vmrInitGJson/vmrGetString/vmrGetInt/vmrGetByKey/vmrMapEach/vmrGetByIndex/vmrSliceEach`
  - utils：24 个（os/arch、regex、字符串、url/path join、文件读写删、拷贝、命令执行）
  - version：`vmrNewVersionList/vmrAddItem/vmrMergeVersionList`（Item/VersionList 数据结构）
  - github：`vmrGetGithubRelease(repo, tagFilter, versionParser, fileFilter, archParser, osParser, installerGetter)` → 调 vmr-net 的 GitHub API，回调 7 个 Lua 函数过滤（要求 3）
  - installer_config：`vmrNewInstallerConfig/vmrAddFlagFiles/vmrEnableFlagDirExcepted/vmrAddBinaryDirs/vmrAddAdditionalEnvs`（全局变量 `ic`）
  - conda：`vmrSearchByConda` → 调 vmr-conda 查 repodata（要求 4）
  - proxy：`vmrGetProxy`
  - extractor：`vmrUnarchive`
- **新增**：显式注入全局常量 `vmrInstallerUnarchiver`/`vmrInstallerExecutable`/`vmrInstallerConda`/`vmrInstallerCondaForge`/`vmrInstallerCoursier`/`vmrInstallerDpkg`/`vmrInstallerRpm`（修复 Go 侧缺口）。
- `plugins.rs`：扫描 `~/.vmr/plugins/*.lua`；目录缺失自动 `UpdatePlugins()`（从 `gvcgo/vmr_plugins` 仓库 main.zip 下载 → 按 `go.lua`/`LICENSE` 标记找目录 → 复制 `*.lua` → gh API 列文件写 `plugins.json`）。
- `plugin.rs`：加载、执行 `crawl()`、按 `Item.Os/Arch` 过滤、`GetInstallerConfig()`（读回 `ic`）、`GetSortedVersions`、`GetVersion`、`GetLatestVersion`、自定义 `install`/`postInstall` 回调。
- **版本缓存策略**：Go 侧有 `<cache>/<plugin>/<plugin>.versions.json` 缓存；要求「实时获取」→ 默认**每次实时执行 crawl**，缓存仅作离线兜底/可配置开关。

### 3.5 vmr-conda —— conda 源客户端（要求 2、4）

参考 pixi（prefix-dev），实现「从 conda 源把包安装到 vmr 版本目录」，**不依赖本机 conda/miniconda**：

- **channel 配置**：默认 `https://conda.anaconda.org/conda-forge`，支持自定义 channel + 镜像映射（走 vmr-core 镜像表）；platform 映射：`linux-64/osx-arm64/osx-64/win-64`（由 os/arch 推导）。
- **repodata 获取**：拉取 `<channel>/<platform>/repodata.json`（支持 `.zst` 压缩），缓存到 `<cache>/conda_repodata/`（etag/时间戳增量，参考 rattler_repodata_gateway 策略）。
- **版本信息（要求 4）**：查 repodata 中包名的全部版本 → 组装 `VersionList`（每个版本一个 `Item`，`installer="conda"`，`url` 指向包文件）；供 `vmrSearchByConda` 与 CLI `search -c`。
- **包下载与提取**：`.conda`（zip 容器：`info-*.tar.zst` 元数据 + `pkg-*.tar.zst` 文件）或 `.tar.bz2`；提取到 vmr 版本目录 `<versions>/<sdk>_versions/<sdk>-<version>`（即 conda 的 prefix）；保留包 metadata（`info/paths.json`、`run_exports.json`）。
- **依赖解析**：两阶段：
  - 阶段一（最小可用）：**单包安装，不递归依赖**（vmr 管理的语言工具多可独立运行；go/node 等已有 unarchiver 路径）；
  - 阶段二（可选）：引入 `rattler_solve`（resolvo）做完整传递依赖解析，行为对齐 pixi。
- **环境变量**：安装后从 `ic` 或包 metadata 推导 env（`prefix/bin` 加 PATH；`activate.d/*.sh` 若存在则记录到 env 模块），替代 conda 的 activate 机制。
- 实现方式：直接用 `rattler_conda_types` + `rattler_repodata_gateway` + `rattler_package_streaming`（pixi 同款，正确性优先）；如编译/依赖负担过重，退化为自研（`serde_json` 解析 repodata + 自写 `.conda`/zst 提取）【决策点 D1，默认 rattler】。

### 3.6 vmr-installer —— 独立安装器（要求 5）

- `SDKInstaller` trait：`initiate / set_install_conf / get_install_dir / get_symbol_link_path / install`。
- 分派（按 `Item.Installer`）：
  - `unarchiver`（默认）：缓存下载 → 解压到 temp → 找家目录 → 复制到版本目录；
  - `executable`：独立可执行文件（exe/deb/rpm/sh 直接放置）；
  - `conda`/`conda-forge` → vmr-conda（§3.5）；**移除** `CheckAndInstallMiniconda`；
  - `coursier`：`cs install --install-dir`（保留，独立方式）；
  - `dpkg`/`rpm` → executable 分支（原 Go 行为）。
- `Install()` 调度：前置检查（仅 coursier 需 `CheckAndInstallCoursier`）→ 未装则安装 → `post/` 后处理（zig/upx chmod、rust rustup-init、php opcache 绝对路径、moonbit bundle、clojure jar、bun bunx——按插件名注册表）。
- 模式：`Globally`（符号链接 + 全局 env）/ `Sessionly`（临时 env + 子 shell）/ `ToLock`（`.vmr.lock` + 子 shell）；`Uninstall`（删版本目录，当前版本则删符号链接 + 撤 env）。

### 3.7 vmr-env —— 独立环境变量模块（要求 6）

- `collect`：按平台取 `ic.BinaryDirs.{windows|linux|darwin}`（空则版本目录本身）+ `ic.AdditionalEnvs`，仅收集存在的路径。
- `set_global`：经 vmr-shell 写 shell 配置 / Windows 注册表（`HKCU\Environment` + `WM_SETTINGCHANGE` 广播）。
- `unset_global`：从 shell 配置移除。
- `set_temporary`：`VMR_ADD_TO_PATH_TEMPORARILY=1` 时注入当前进程 PATH（会话/锁模式用）。
- `remove_global_sdk_path`：从 PATH 摘除当前 SDK 符号链接路径。

### 3.8 vmr-shell —— shell hook 自动切换（要求 7）

- `sheller` trait：`set_env/unset_env/set_path/unset_path/conf_path`。
- bash（`~/.bashrc` + `~/.vmr/vmr.sh`）、zsh（`~/.zshrc` + cd hook `vmr use -E` + `VMR_CD_INIT` 首启）、fish（`config.fish` + `vmr.fish` + `--on-variable=PWD` hook）、powershell（`profile.ps1` + Set-Alias）、git-bash/mingw（`VMR_VERSIONS` 修补）。
- **cd hook 自动切换**：进入含 `.vmr.lock` 的目录时自动 `vmr use -E`（按 lock 内容切换 SDK 版本）；`VM_DISABLE` 守卫（会话内禁用注入）；`vmr use -E` 命令端点保留。
- `UpdateVMRShellFile`：按 `# cd hook start/end` 标记幂等替换。

### 3.9 vmr-pty —— 会话终端

- `RunTerminal`：交互子 shell（unix `sh -c` + PTY + SIGWINCH resize；win ConPTY）；会话前设 `VM_DISABLE=111`、移除全局 SDK 路径、注入临时 env。
- 用 `portable-pty` 替代 Go 的 creack/pty + ConPTY 手写层。

### 3.10 vmr-self / vmr-completions

- `install_self`：复制自身到 `~/.vmr/<bin>` → 写 shell env → 更新/卸载脚本（`scripts.vmr.dpdns.org`）→ **交互式**设置 SDK 安装目录（改为命令行参数或默认值，无 TUI）【决策点 D2】。
- `uninstall_self` / `old_versions` 清理。
- completions：`vmr completion <shell>` 输出 → 写 `~/.vmr/vmr_completions.*` → 追加到 shell 配置。

### 3.11 vmr-cli —— 命令行（要求 9）

clap 实现，保留原命令名/别名（脚本兼容）：

| 命令 | 别名 | 说明 |
|---|---|---|
| `use <sdk>@<ver>` | `u`/`h` | `-E` 项目锁 / `-s` 会话 / `-l` 锁版本 / `-c` conda 安装 |
| `uninstall <sdk>@<ver>\|all` | `uni`/`r` | 卸载 |
| `show` | `S` | 列出 SDK（插件扫描），**文本表格** |
| `search <sdk>` | `s` | 版本列表，**文本表格**；`-c` 走 conda repodata |
| `local <sdk>` / `installed-sdks` / `installed-info` | `l`/`in`/`ii` | 已装版本，**文本表格**（当前版本标 `<current>`） |
| `set-proxy` / `set-reverse-proxy` / `set-download-threads` / `toggle-customed-mirrors` | `sp`/`sr`/`st`/`tm` | 配置 |
| `nested-sessions` / `is-session-mode` | `ns`/`ism` | 会话开关/查询 |
| `update-plugins` | `up` | 更新 Lua 插件 |
| `add-completions` | `ac` | 补全 |
| `install-self` / `uninstall-self` | `i`/`is`、`Uins` | 自管理 |
| `version` | `v` | `gitTag(gitHash[:7])` |

**不实现**：`tui` 及其任何残留（`cliui` 的输出逻辑并入文本表格）。

---

## 4. 磁盘与环境兼容性契约（必须逐项保留）

1. 路径：`~/.vmr`、`~/.vmr/conf.toml`、`~/.vmr/versions`、`~/.vmr/cache`、`~/.vmr/temp`、`~/.vmr/plugins`；`VMR_SDK_INSTALLATION_DIR` 覆盖。
2. 已装表达：`<versions>/<sdk>_versions/<sdk>-<version>` + `<versions>/<sdk>` 符号链接（win junction）。
3. 环境变量名：`VMR_SDK_INSTALLATION_DIR`/`VMR_HOST`/`VMR_REVERSE_PROXY`/`VMR_LOCAL_PROXY`/`VMR_DOWNLOAD_THREADS`/`VMR_USE_CUSTOMED_MIRRORS`/`VMR_ALLOW_NESTED_SESSIONS`/`VM_DISABLE`/`VMR_ADD_TO_PATH_TEMPORARILY`/`VMR_CD_INIT`/`VMR_VERSIONS`/`GVC_DEFAULT_PROXY`/`VCOLLECTOR_PROXY`（含 `Donwload` 拼写 quirk）。
4. `.vmr.lock` 项目锁（JSON 或 `name@version`），向上查找。
5. Lua 插件目录与 50 个 `vmr*` 函数签名（现有 `gvcgo/vmr_plugins` 必须原样运行）。
6. `conf.toml` 键名（PascalCase，待实测确认）+ 拼写 quirk（`SDKIntallationDir`）。
7. 版本排序语义：稳定 > beta > rc。
8. 下载/代理/镜像优先级链（§3.3）。
9. 更新/卸载脚本 URL：`https://scripts.vmr.dpdns.org`。

---

## 5. 实施阶段（依赖顺序）

| 阶段 | 内容 | 验证 |
|---|---|---|
| P0 | workspace 脚手架 + vmr-utils（semver/extract/copy/find_dir/exec/symlink）+ vmr-core（路径/配置/策略） | 单测：版本排序、解压各格式、conf.toml 读写 |
| P1 | vmr-net（fetcher/多线程下载/镜像/反代/代理/github API） | 单测：分片合并、校验和；手动下载验证 |
| P2 | vmr-lua（mlua 运行时 + 50 绑定 + 插件加载/更新） | 跑现有 go.lua/coursier.lua 插件出版本列表 |
| P3 | vmr-conda（repodata 查询 + 包提取安装） | `search -c` 出版本；装一个纯单包验证 |
| P4 | vmr-installer（4 类安装器 + post 后处理 + 模式调度） | 真实安装 go/node 并 `use` 切换 |
| P5 | vmr-env + vmr-shell（hook 自动切换） | 新终端进入带 `.vmr.lock` 目录自动切换 |
| P6 | vmr-pty + vmr-self + vmr-completions | `use -s` 会话；install-self 全流程 |
| P7 | vmr-cli 组装（文本输出） | 全命令冒烟；与 Go 版对拍 `show/search/local` |
| P8 | 清理：移除 vsources 死代码确认、插件常量注入回归、文档 | 端到端：装→切→锁→会话→卸 |

---

## 6. 待确认决策点

- **D1** conda 实现：直接用 rattler 系列 crate（pixi 同款，正确性优先，编译重） vs 自研轻量（serde_json + 自写 zst/zip 提取，依赖少）。默认：rattler。
- **D2** `install-self` 的「交互式设置 SDK 安装目录」在无 TUI 下的替代：命令行参数（`--sdk-dir`） vs 默认 `~/.vmr` 不询问。默认：命令行参数。
- **D3** 版本缓存：完全实时（每次 crawl） vs 保留短 TTL 缓存。默认：实时 + 可配置。
- **D4** conda 依赖解析：阶段一单包 vs 直接 rattler_solve 完整解析。默认：阶段一单包，后续迭代。
- **D5** Lua 版本：mlua `lua54` vs `lua51`（现有插件按 5.1 风格编写，但未用到 5.1 专属特性，两者均可）。默认：lua54。

---

*计划生成：2026-08-11。基于 vmr-go 源码核实 + vmr_plugins 样例 + pixi 机制研究。*
