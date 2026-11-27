# 命令参考

使用 `vmr --help` 或 `vmr <命令> --help` 查看当前版本的参数。`sdk@version` 中的名称和版本可先通过 `vmr plugins`、`vmr search` 查询。

| 命令 | 别名 | 用途 |
| --- | --- | --- |
| `vmr version` | `v` | 显示 vmr 版本。 |
| `vmr init [shell]` | | 输出 bash、zsh、fish 或 powershell 的集成脚本。 |
| `vmr template [kind] [-d DIR]` | | 输出 `github`、`html`、`json` 或 `pixi` Lua 插件模板；指定目录时写入文件。 |
| `vmr plugins [-u]` | | 列出插件；`-u` / `--update-all` 先更新插件。 |
| `vmr search [sdk] [-p] [-n]` | `s` | 查询版本；`-p` / `--pixi` 查询 conda-forge，`-n` / `--no-cache` 绕过缓存。 |
| `vmr local [sdk] [-c]` | `l` | 列出已安装版本；`-c` / `--current` 只显示当前版本。 |
| `vmr use [sdk@version]` | `u`、`h` | 安装并切换版本，默认全局生效。 |
| `vmr use -s sdk@version` | | `--session`：在临时会话使用版本。 |
| `vmr use -l sdk@version` | | `--lock`：写入项目 `.vmr.lock` 并进入会话。 |
| `vmr use -p sdk@version` | | `--pixi`：通过 conda-forge 安装。 |
| `vmr use -f FILE` | | `--file`：按清单批量安装；不能与版本参数或上述模式选项一起使用。 |
| `vmr freeze [FILE]` | `fz` | 导出已安装版本；默认文件名为 `vmr_<YYYYMMDD>.txt`。 |
| `vmr migrate [MAP] --asdf` | | 从 asdf 生成 `migration_from_asdf.txt`。 |
| `vmr migrate [MAP] --vfox` | | 从 vfox 生成 `migration_from_vfox.txt`。 |
| `vmr migrate [MAP] --system` | | 从 PATH 上的系统命令生成 `migration_from_system.txt`。 |
| `vmr uninstall sdk@version` | `uni`、`r` | 卸载指定版本；`sdk@all` 卸载该 SDK 的所有版本。 |
| `vmr cdhook` | `cd-hook` | shell 集成内部使用：加载项目锁文件，提示安装缺失版本并注入环境。 |
| `vmr session` | `se` | 查询当前 shell 是否处于 vmr 会话。 |
| `vmr nested` | `ns` | 切换是否允许嵌套会话。 |
| `vmr doctor` | | 只读诊断安装、shell 集成和环境。 |
| `vmr mcp` | | 通过 stdio 提供 MCP 服务。 |

`doctor` 支持 `--json`、`--quiet`、`--strict`、`--no-emoji` 和 `--shell <shell>`；`--strict` 在发现警告时返回退出码 1。

`migrate` 只生成清单，不执行安装。可选的映射文件每行写 `<来源名> <vmr-sdk>`，未列出的名称会被跳过。迁移与配置示例见[使用教程](/zh/guides/tutorial)，插件模板见[插件文档](/zh/plugins/)。
