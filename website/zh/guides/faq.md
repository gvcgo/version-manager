# 常见问题

## 找不到 `vmr`？

确认可执行文件所在目录已加入 `PATH`。Unix 系统可用 `which vmr`，Windows 可用 `where vmr`。源码安装通常放在 `~/.cargo/bin`。参见[安装说明](/zh/starts/quickstart)。

## 切换后当前终端仍是旧版本？

检查是否在 shell 配置中执行了对应的 `vmr init`，也不要用 `command vmr` 绕过包装函数。配置完成后重新打开终端。运行 `vmr doctor` 查看 shell 集成、环境与 PATH 的诊断信息。

## 查询版本遇到 GitHub 限流？

匿名 GitHub API 请求通常每小时限制 60 次。设置 `VMR_GITHUB_TOKEN` 或配置文件中的 `GithubToken`；也可使用 `GITHUB_TOKEN`。需要重新获取版本列表时运行 `vmr search go -n`。

## 下载慢或无法连接？

可设置 `HTTP_PROXY`、`HTTPS_PROXY`；只为 vmr 配置代理可使用 `VMR_LOCAL_PROXY`。conda-forge 镜像可通过 `VMR_PIXI_CHANNEL` 指定。配置位置见[使用教程](/zh/guides/tutorial)。

## 项目版本没有自动切换？

先确认 shell 已接入 `vmr init`，项目目录存在 `.vmr.lock`。进入项目时，缺失的锁定版本会提示安装；如果拒绝，vmr 会跳过安装。用 `vmr doctor` 检查环境。

## 会话中不能再开启会话？

默认禁止嵌套会话。先运行 `vmr nested` 允许嵌套，或者退出当前子 shell 后再操作。

## 如何从 0.7.x 升级到 0.8.x？

新版会检测旧版自安装在 `~/.vmr/vmr`（Windows 为 `vmr.exe`）的二进制并提示删除，并清理旧的 cd hook，保留 SDK 相关环境配置。删除 shell 配置中旧版 vmr 代码，改为[对应的初始化命令](/zh/starts/quickstart)；PowerShell 用户也应删除旧版写入 `$PROFILE` 的非 SDK 变量和函数。重新打开终端。

## 如何卸载 vmr？

先用 `vmr uninstall <sdk>@all` 删除不再需要的 SDK，再删除 `~/.vmr` 和 shell 配置中的初始化行，最后用原安装方式卸载 vmr 可执行文件。Windows 上还需从 PATH 移除 `%USERPROFILE%\.vmr\bin`；删除数据前请备份需要保留的配置和版本。
