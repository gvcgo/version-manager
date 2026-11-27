# 核心概念

## 插件与版本

插件负责发现和安装 SDK 的可用版本。`vmr plugins` 列出插件，`vmr search go` 查询当前平台可安装的 Go 版本。安装多个版本后，`vmr use go@1.22.0` 选择全局使用的版本；`vmr local go` 查看已安装版本。插件开发见[插件文档](/zh/plugins/)。

## 全局、项目与会话

默认的 `vmr use` 设置全局版本。`vmr use -s python@3.12.2` 则进入临时子 shell，退出后恢复原环境，不改变全局选择。

在项目根目录执行 `vmr use -l node@20.11.0` 会写入 `.vmr.lock` 并进入会话。配置 shell 集成后，进入锁定项目时钩子会加载版本；缺少版本时会询问是否安装，离开项目时恢复环境。建议将锁文件提交到仓库。

默认不允许嵌套会话；需要时使用 `vmr nested` 切换该设置。目录钩子将项目位置缓存在 `VMR_CD_CACHE`，避免在同一项目内反复查找。

## Shell 集成和数据位置

`vmr init` 输出适用于当前 shell 的集成脚本，包装 `vmr` 命令并处理目录变化。安装方式见[快速上手](/zh/starts/quickstart)。

默认数据位于 `~/.vmr`：`plugins/` 保存插件，`cache/` 保存缓存，`versions/` 保存 SDK 版本，`vmr.sh` 或 `vmr.fish` 保存动态环境。`VMR_SDK_INSTALLATION_DIR` 可更改版本与缓存根目录。

Windows 使用 `%USERPROFILE%\.vmr\bin` 中的 shim 暴露 SDK 命令，全局非 PATH 环境变量写入用户注册表。shim 只为启动的子进程注入相应前缀，避免将每个 SDK 路径都加入全局 PATH。

## conda-forge 与清单

`vmr search -p` 和 `vmr use -p` 使用内置 pixi 能力，无需预装 conda。安装后可执行文件会加入环境，但包的 `etc/conda/activate.d` 脚本设置的变量不会自动导入。

`vmr freeze` 导出已安装版本的文本清单，`vmr use -f` 在另一台机器按清单安装。清单每行一个 `sdk@version`，`#` 开头的行是注释；每个 SDK 当前版本写在最后，以便重放时恢复选择。详见[使用教程](/zh/guides/tutorial)。
