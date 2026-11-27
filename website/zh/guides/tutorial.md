# 使用教程

以下示例使用的版本号只是示例；先用 `vmr search` 选择当前平台可用的版本。

## 安装、切换和卸载

```bash
vmr plugins
vmr search node
vmr use node@20.11.0
vmr local node
vmr local -c
vmr uninstall node@20.11.0
# 删除某个 SDK 的所有已安装版本：vmr uninstall node@all
```

`vmr use` 在需要时安装并切换到指定版本。添加 shell 集成后，当前终端会立即刷新环境。

## 为项目固定版本

```bash
cd my-project
vmr use -l node@20.11.0
```

这会写入 `.vmr.lock` 并进入会话。将锁文件提交到项目，其他成员配置 shell 集成后进入目录即可加载锁定版本；缺失的版本会先询问是否安装。退出会话或离开项目可恢复原环境。

项目锁文件大小上限为 1 MiB，超出时会被忽略。

只想临时试用版本时，使用 `vmr use -s python@3.12.2`，完成后退出子 shell。会话内如需再次开启会话，先用 `vmr nested` 允许嵌套。

## 使用 conda-forge

```bash
vmr search -p lua
vmr use -p lua@5.4.6
```

无需预装 conda；但 conda 的 `activate.d` 脚本变量不会导入。如需镜像，可设置 `VMR_PIXI_CHANNEL`：

```bash
export VMR_PIXI_CHANNEL=https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/conda-forge
```

## 备份和迁移

在旧机器导出清单，将文件复制到新机器，再批量安装：

```bash
vmr freeze versions.txt
vmr use -f versions.txt
```

也可省略文件名，让 `freeze` 使用 `vmr_<YYYYMMDD>.txt`。清单为纯文本，每行一个 `sdk@version`，当前版本写在每个 SDK 的最后。

从其他管理器或系统收集版本时，先生成清单，检查后再安装：

```bash
vmr migrate --asdf
vmr migrate --vfox
vmr migrate --system
vmr use -f migration_from_asdf.txt
```

迁移命令不会安装，也不会修改来源管理器的目录；它分别生成 `migration_from_asdf.txt`、`migration_from_vfox.txt` 或 `migration_from_system.txt`。系统扫描只取 PATH 上每个命令的第一个可执行文件，并跳过常见版本管理器管理的程序；项目级版本文件不会转换。

可提供名称映射文件，例如 `vmr migrate names.txt --asdf`。每行写 `<来源名> <vmr-sdk>`；只写来源名或将目标写成 `-` 表示跳过。提供映射文件后，未列出的名称也会跳过并报告。

## 配置网络与缓存

vmr 优先读取 `~/.vmr/conf.toml`；不存在时读取 `~/.config/vmr/conf.toml`。两份配置不会合并。两者均不存在时，写回会创建前者。键名为 PascalCase，例如：

配置文件和 vfox 迁移 TOML 文件的大小上限为 1 MiB；版本缓存上限为 16 MiB，超出时会重新获取。

```toml
ProxyUri = "http://127.0.0.1:7890"
ThreadNum = 4
CacheRetentionTime = 3600
```

其他配置键包括 `ReverseProxy`、`SDKIntallationDir`（拼写如此）、`VersionHostUrl`、`AllowNestedSessions`、`GithubToken` 和 `DisableCache`。常用环境变量包括 `VMR_SDK_INSTALLATION_DIR`、`VMR_HOST`、`VMR_LOCAL_PROXY`、`VMR_REVERSE_PROXY`、`VMR_DOWNLOAD_THREADS`、`VMR_ALLOW_NESTED_SESSIONS`、`VMR_PIXI_CHANNEL` 和 `VMR_GITHUB_TOKEN`；也支持 `HTTP_PROXY`、`HTTPS_PROXY`、`NO_PROXY`。GitHub Token 未设置时可回退到 `GITHUB_TOKEN`。

遇到问题可运行 `vmr doctor`，或查阅[常见问题](/zh/guides/faq)和[命令参考](/zh/reference/commands)。
