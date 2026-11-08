# VMR Lua 插件 API

本文是编写版本发现插件的完整参考：插件文件格式、生命周期、数据模型，以及运行时注册的全部
`vmr*` Lua 绑定。

- English: [plugin-api.md](plugin-api.md)
- 设计说明与旧 `vcollector` 爬虫的对应关系：[plugin-system.md](plugin-system.md)

运行时为 Lua 5.4（`mlua`，内置 vendored Lua）。每个插件在独立的 Lua state 中执行，全局变量不会
在插件之间泄漏。完整 Lua 标准库可用，包括 `dofile`、`os` 和 `io`。

## 1. 文件布局与发现流程

| 项目 | 取值 |
|---|---|
| 插件目录 | `~/.vmr/plugins/`（`$HOME/.vmr/plugins`，会自动创建） |
| 插件文件 | 目录下的 `*.lua`，按文件名顺序加载 |
| 辅助文件 | 文件名以 `_` 开头（如 `_vmr_common.lua`）的文件**不会**被当作插件 |
| 安装目录 | `<versions>/<sdk_name>_versions/<plugin_name>-<version>` |
| 当前版本软链接 | `<versions>/<sdk_name>_versions/<sdk_name>` |
| 版本缓存 | `<cache>/<plugin_name>/<plugin_name>.versions.json` |

`<versions>` 和 `<cache>` 默认分别是 `~/.vmr/versions` 与 `~/.vmr/cache`；设置
`VMR_SDK_INSTALLATION_DIR` 后随之改变。

插件不内嵌在二进制里。`Plugins::new()` 发现目录中没有 `.lua` 文件时，会在每个进程中自动下载一次
`gvcgo/vmr_plugins`。`vmr plugins -u` 会显式刷新插件目录和 `plugins.json`，同名文件会被覆盖。

`vmr search` / `vmr use` 解析 SDK 名称时，先按 `plugin_name` 匹配，再按 `sdk_name` 匹配。

### 创建模板

```bash
vmr template github                # 输出到 stdout
vmr template json --dir ./plugins  # 写入 ./plugins/json.lua
```

可用类型：`github`、`html`、`json`、`pixi`（也接受 `gh`、`conda`）。

## 2. 插件文件格式

插件就是普通 Lua 脚本：给全局变量赋值，并定义 `crawl()`。

### 必需全局变量

| 全局变量 | 类型 | 含义 |
|---|---|---|
| `plugin_name` | string | 稳定的插件 id。用于 SDK 查找、缓存目录、安装目录以及 pixi 包名。不能为空。 |
| `sdk_name` | string | SDK id，供 `vmr use <sdk>@<version>`、安装目录和当前版本软链接使用。不能为空。 |
| `homepage` | string | 上游主页，`vmr plugins` 会显示。不能为空。 |
| `ic` | `InstallerConfig` userdata | 安装与环境配置。用 `vmrNewInstallerConfig()` 及 `vmrAdd*`/`vmrSet*` 构造。 |
| `crawl` | function | 无参调用，返回 `VersionList`。 |

### 可选全局变量

| 全局变量 | 类型 | 含义 |
|---|---|---|
| `plugin_version` | string | 插件版本，`vmr plugins` 会显示；不参与排序或安装。 |
| `prequisite` | string | 旧元数据；会被读取，但当前 CLI 不强制执行。 |
| `install(path, version)` | function | 旧的自定义安装钩子，见 §9。 |
| `postInstall(path, version)` | function | 旧的安装后钩子，见 §9。 |

元数据全局变量也可以写成返回字符串的无参函数：

```lua
plugin_version = function() return "0.2" end
```

加载校验会在以下情况失败：`plugin_name`、`sdk_name` 或 `homepage` 为空，或者缺少 `ic`、`crawl`。
查询版本时还会检查 `crawl` 是否可调用。

### 最小骨架

```lua
local C = dofile(__vmr_plugin_dir .. "/_vmr_common.lua")

plugin_name = "example"
sdk_name = "example"
plugin_version = "0.1"
homepage = "https://example.com/"

ic = vmrNewInstallerConfig()
ic = vmrAddFlagFiles(ic, "", { "bin" })
ic = vmrAddBinaryDirs(ic, "", { "bin" })

function crawl()
    local vl = vmrNewVersionList()
    vmrAddItem(vl, "1.2.3", {
        url = "https://example.com/example-1.2.3-linux-amd64.tar.gz",
        os = "linux",
        arch = "amd64",
        installer = vmrInstallerUnarchiver,
        sum = "",
        sum_type = "",
        size = 0,
    })
    return vl
end
```

脚本执行前会注入绝对路径 `__vmr_plugin_dir`；配合 `dofile` 可加载共享辅助模块。

### `crawl()` 约定

- 不接收参数。
- 返回 `VersionList` userdata，通常来自 `vmrNewVersionList()`、`vmrGetGithubRelease()`、
  `vmrSearchByPixi()` 或 `vmrMergeVersionList()`。
- 返回空列表是合法的；空结果不会写入缓存，因此下次查询会重新爬取。
- 报错或返回非 `VersionList` 时，`vmr search` / `vmr use` 会报告错误。
  `crawl()` 不应调用 `vmr use`，也不应修改安装目录。

## 3. 数据模型

### `VersionList`

带键的集合：版本字符串 → 候选 `Item` 列表。所有返回版本列表的 `vmr*` 函数都返回同一种
userdata，因此结果可以直接合并，或直接从 `crawl()` 返回。

```lua
vl = vmrNewVersionList()
vl = vmrAddItem(vl, "1.2.3", item)
vl = vmrMergeVersionList(vl, other_vl)
```

### `Item`

作为 `vmrAddItem` 的第三个参数传入（普通 Lua table）：

| 字段 | 类型 | 含义 |
|---|---|---|
| `url` | string | 下载地址。`unarchiver`、`executable`、`dpkg`、`rpm` 必需。 |
| `os` | string | `linux`、`darwin`、`windows`、`any`、`unix`；`macos`/`osx` 视为 `darwin`。 |
| `arch` | string | `amd64`、`arm64`、`386` 或 `any`。 |
| `sum` | string | 期望校验和；为空则不校验。 |
| `sum_type` | string | `md5`、`sha1`、`sha256`、`sha512`；其他值不校验。 |
| `size` | integer | 期望字节数；`0` 不检查大小。 |
| `installer` | string | 安装类型，见 §3.1。未知值按 `unarchiver` 处理。 |
| `lts` | string | 仅元数据，保留兼容，安装时不使用。 |
| `extra` | string | 仅元数据，保留兼容，安装时不使用。 |

缺失字段按空字符串 / `0` 读取。

#### 3.1 安装类型

| Item 取值 | 常量 | 行为 |
|---|---|---|
| `unarchiver` | `vmrInstallerUnarchiver` | 下载、解压，通过 `ic.flag_files` 定位压缩包根目录，再复制到安装目录。未知类型的默认行为。 |
| `executable` | `vmrInstallerExecutable` | 下载单个文件，复制到安装目录并加可执行权限；支持 `ic.binary_rename`。 |
| `conda` | `vmrInstallerConda` | 通过内置 pixi/rattler 安装；包名为 `plugin_name`，忽略 `url`/`sum`。 |
| `conda-forge` | `vmrInstallerCondaForge` | 与 `conda` 使用同一后端。 |
| `coursier` | `vmrInstallerCoursier` | 调用 `cs`（`VMR_COURSIER_PATH`，默认 `cs`）并传 `--install-dir`；要求机器上已安装 `cs`。 |
| `dpkg` | `vmrInstallerDpkg` | 按单文件下载处理（同 `executable`）。 |
| `rpm` | `vmrInstallerRpm` | 按单文件下载处理（同 `executable`）。 |

使用 `conda`/`conda-forge`/pixi 时，安装目录以 `plugin_name` 命名，`vmr-pixi` 也会用
`plugin_name` 在 conda-forge repodata 中解析包。因此 pixi 插件的 `plugin_name` 必须是 conda 包名。

### 3.2 平台过滤

`crawl()` 返回所有候选条目，VMR 只保留与当前平台匹配的条目：

| Item `os` | 匹配范围 |
|---|---|
| `any` | 任意系统 |
| `unix` | `linux` 与 `darwin` |
| `macos`、`osx` | `darwin` |
| 其他 | 精确匹配（`linux`、`darwin`、`windows`） |

Item `arch` 精确匹配，或为 `any`。

同一版本有多个条目匹配时，**最后一个**匹配项生效。请把更精确的资源放在后面，或每个平台只输出
一个条目。

### 3.3 版本名与排序

map 的键就是 `vmr use <sdk>@<version>` 接受的原始版本字符串，不做归一化；需要去掉前缀 `v` 时由
插件自行处理。查询结果使用 VMR 的 semver 比较器按从新到旧排序；`@latest` 取第一条。

## 4. 生命周期与缓存

```
Plugins::new()
  └─ 没有 .lua 文件 → 下载 gvcgo/vmr_plugins（每个进程一次）
load_all()
  └─ 执行插件 → 读取元数据 → 校验 plugin_name/sdk_name/homepage/ic/crawl
get_sdk_versions()
  ├─ 缓存新鲜？ → 返回 ~/.vmr/cache/<plugin>/<plugin>.versions.json
  └─ 否则 crawl → 按当前 os/arch 过滤 → 写缓存
use → 选择 Item → 按 Item.installer 安装 → 后处理 → 注入环境
```

- 缓存只保存当前平台的版本，每个版本只保留一个 `Item`。
- 缓存年龄小于配置的 `cache_retention_time`（默认 86400 秒）且缓存未禁用时视为新鲜。
- `vmr search <sdk> -n`（`--no-cache`）忽略缓存，重新爬取并覆盖缓存。
- 缓存以 `plugin_name` 为键，而不是 `sdk_name`。
- 空爬取结果不会写缓存。

## 5. 安装配置（`ic`）

`ic` 是 `InstallerConfig` userdata。所有构造函数都返回同一个 userdata，可以链式调用或重新赋值。

| 函数 | 作用 |
|---|---|
| `vmrNewInstallerConfig()` | 创建空配置。 |
| `vmrAddFlagFiles(ic, os, list)` | 为某个平台追加压缩包根目录的标记文件名（`""` 表示三个平台）。 |
| `vmrEnableFlagDirExcepted(ic)` | 定位压缩包根目录时忽略目录名，只用文件名匹配。 |
| `vmrAddBinaryDirs(ic, os, list)` | 为某个平台追加**一组**相对路径（一个 bin 目录）；`""` 表示三个平台。 |
| `vmrSetBinaryRename(ic, name_flag, rename_to)` | 安装 `executable` 时，把文件名包含 `name_flag` 的可执行文件重命名为 `rename_to`（Windows 会补 `.exe`）。 |
| `vmrAddAdditionalEnvs(ic, name, list, version)` | 添加一个环境变量，指向安装根目录下的一组路径；`version` 会被保存，但收集环境时不使用。 |

各字段的消费方式：

- `flag_files`（`windows`/`linux`/`darwin`）：解压后，把每个目录的条目名拼接成字符串做子串匹配；
  第一个包含**全部**标记的目录成为安装根目录。标记为空时匹配解压根目录本身。启用
  `flag_dir_excepted` 后，目录条目不参与匹配。
- `binary_dirs`：每组路径拼接到安装根目录下，存在时加入 `PATH`。某平台没有配置时，安装根目录本身
  会加入 `PATH`。
- `additional_envs`：路径组拼接到安装根目录下，存在时导出为 `NAME=<绝对路径>`。空路径组指向安装
  根目录（如 `GOROOT`）。
- `binary_rename` 仅对 `executable` 安装类型生效。

示例：

```lua
ic = vmrNewInstallerConfig()
ic = vmrAddFlagFiles(ic, "linux", { "bin", "LICENSE" })
ic = vmrAddFlagFiles(ic, "darwin", { "bin", "LICENSE" })
ic = vmrAddFlagFiles(ic, "windows", { "bin", "LICENSE" })
ic = vmrEnableFlagDirExcepted(ic)
ic = vmrAddBinaryDirs(ic, "linux", { "bin" })
ic = vmrAddBinaryDirs(ic, "darwin", { "bin" })
ic = vmrAddBinaryDirs(ic, "windows", { "bin" })
ic = vmrAddAdditionalEnvs(ic, "EXAMPLE_HOME", {}, "")
```

平台参数行为：

| `os` 取值 | 效果 |
|---|---|
| `""` | 同时追加到 `windows`、`linux`、`darwin` |
| `"windows"` / `"linux"` / `"darwin"` | 追加到对应平台 |
| 其他 | 不修改，原样返回配置 |

## 6. 运行时辅助变量

| 全局变量 | 含义 |
|---|---|
| `__vmr_plugin_dir` | 插件目录的绝对路径；用于 `dofile(__vmr_plugin_dir .. "/_vmr_common.lua")`。 |
| `print` | 在 MCP stdio 服务下改为写 stderr，避免污染 JSON-RPC 流。 |

## 7. API 参考

### 7.1 HTTP

#### `vmrGetResponse(url [, timeout [, headers]]) → body`

发起 GET 请求。`timeout` 单位为秒（省略或 `<= 0` 时默认 `180`）。`headers` 是可选的
string → string 表。

返回值是保存响应体的不透明 userdata，应直接传给 `vmrInitSelection` 或 `vmrInitGJson`；它不是
Lua 字符串。网络错误、非 2xx 状态或空响应体时，返回的 userdata 保存空字符串。

```lua
local resp = vmrGetResponse("https://example.com/releases.json", 60, {
    ["Accept"] = "application/json",
})
local json = vmrInitGJson(resp)
```

#### `vmrGetProxy() → scheme, host, port`

返回 `~/.vmr/conf.toml` 中 `proxy_uri` 的代理组成。配置为空或非法时返回 `""`、`""`、`"0"`；
URI 没有端口时端口为空字符串。

### 7.2 HTML 选择

selection 是保存一份解析后的文档以及选中节点位置的不透明 userdata。所有操作都相对于选中节点。

#### `vmrInitSelection(body, selector) → selection`

解析 `body`（`vmrGetResponse` 返回的 userdata 或 Lua 字符串）并按 CSS 选择器选中节点。空 body
或非法选择器返回空 selection。

#### `vmrFind(selection, selector) → selection`

在每个当前选中节点的后代中继续选择。

#### `vmrEq(selection, index) → selection`

按从 0 开始的下标取一个节点。负数或越界返回空 selection。

#### `vmrAttr(selection, name) → string`

第一个选中节点的属性值，不存在时返回 `""`。

#### `vmrText(selection) → string`

拼接所有选中节点全部后代的文本。

#### `vmrEach(selection, callback)`

对每个选中节点调用 `callback(index, node_selection)`。`index` 从 0 开始；
`node_selection` 是只含一个节点的 selection，可继续使用 `vmrAttr`/`vmrText`/`vmrFind`。

```lua
vmrEach(vmrInitSelection(resp, "ul.releases li a"), function(_, node)
    local href = vmrAttr(node, "href")
    local version = vmrTrimPrefix(vmrText(node), "v")
end)
```

### 7.3 JSON

`vmrInitGJson` 包装响应体。路径用点分隔对象键和数组下标：`"releases.0.version"`。空路径表示根节点。
缺失值返回 `""` / `0`。

#### `vmrInitGJson(body) → json`

解析 `body`（`vmrGetResponse` 返回的 userdata 或 Lua 字符串）。非法 JSON 变为 `null`，后续查询返回
空值。

#### `vmrGetString(json, path) → string`

把 `path` 处的值转成字符串：字符串原样，数字和布尔值转字符串，对象/数组序列化为 JSON，
`null`/缺失返回 `""`。

#### `vmrGetInt(json, path) → integer`

转成整数；缺失或无法解析时返回 `0`。

#### `vmrGetByKey(json, path, key) → string`

先解析 `path` 得到对象，再读取其中的 `key`，按 `vmrGetString` 的规则转字符串。

#### `vmrMapEach(json, path, callback)`

遍历 `path` 处的对象：`callback(key, value_json)`。`value_json` 是新的 JSON userdata，因此
`vmrGetString(value_json, "url")` 可用。非对象不执行任何操作。

#### `vmrSliceEach(json, path, callback)`

遍历 `path` 处的数组：`callback(index, value_json)`，下标**从 1 开始**。`null` 元素会被跳过。
非数组不执行任何操作。

#### `vmrGetByIndex(json, path, index) → string`

读取 `path` 处数组的第 `index` 个元素（从 1 开始）并转成字符串。`index < 1`、非数组或越界返回 `""`。

```lua
local root = vmrInitGJson(vmrGetResponse("https://example.com/versions.json", 60, {}))
vmrMapEach(root, "releases", function(version, entry)
    vmrAddItem(vl, vmrTrimPrefix(version, "v"), {
        url = vmrGetString(entry, "url"),
        os = vmrGetString(entry, "os"),
        arch = vmrGetString(entry, "arch"),
        sum = vmrGetString(entry, "sha256"),
        sum_type = "sha256",
        size = vmrGetInt(entry, "size"),
        installer = vmrInstallerUnarchiver,
    })
end)
```

### 7.4 版本列表

#### `vmrNewVersionList() → VersionList`

创建空列表。

#### `vmrAddItem(list, version, item?) → VersionList`

把 `item`（table，见 §3）追加到 `version` 下，返回同一个列表，便于链式调用。`version` 为空时忽略。
不传 table 时不会记录条目。

#### `vmrMergeVersionList(destination, source) → VersionList`

把 `source` 的所有条目追加到 `destination`，返回 `destination`。

### 7.5 GitHub Releases

#### `vmrGetGithubToken() → string`

返回配置的 GitHub token：优先 `~/.vmr/conf.toml` 的 `GithubToken`，其次 `VMR_GITHUB_TOKEN`，
再次 `GITHUB_TOKEN`；都没有则返回 `""`（匿名，每小时 60 次请求）。

#### `vmrGetGithubRelease(repo, tag_filter, version_parser, file_filter, arch_parser, os_parser, installer_getter) → VersionList`

拉取 `repo`（`"owner/repo"`）的全部 release 分页，并通过六个回调过滤：

| 回调 | 入参 | 期望返回 |
|---|---|---|
| `tag_filter(tag)` | release tag（`tag_name`） | 布尔值；只有 true 的 tag 继续 |
| `version_parser(tag)` | release tag | 版本字符串；为空则跳过该 release |
| `file_filter(asset_name)` | 资源文件名 | 布尔值；只有 true 的资源继续 |
| `arch_parser(asset_name)` | 资源文件名 | `amd64`/`arm64`/`386`/`any`；为空跳过该资源 |
| `os_parser(asset_name)` | 资源文件名 | `linux`/`darwin`/`windows`/`any`/`unix`；为空跳过该资源 |
| `installer_getter(asset_name)` | 资源文件名 | §3.1 的安装类型字符串 |

每个通过的资源生成一个 `Item`：`url` 为 `browser_download_url`，`size` 来自 API。URL 包含
`archive/refs/` 的资源总是跳过。回调报错、返回 `nil` 或类型错误时按 `false`/`""` 处理。网络或 API
错误返回空列表，不会抛错。结果是 `VersionList`，可以直接从 `crawl()` 返回或合并。

```lua
function crawl()
    return vmrGetGithubRelease(
        "BurntSushi/ripgrep",
        function(tag) return vmrRegexpFindString("^%d+%.%d+%.%d+$", vmrTrimPrefix(tag, "v")) ~= "" end,
        function(tag) return vmrTrimPrefix(tag, "v") end,
        function(name) return vmrHasSuffix(name, ".tar.gz") or vmrHasSuffix(name, ".zip") end,
        function(name)
            if vmrContains(name, "aarch64") then return "arm64" end
            if vmrContains(name, "x86_64") then return "amd64" end
            return "any"
        end,
        function(name)
            if vmrContains(name, "windows") then return "windows" end
            if vmrContains(name, "apple-darwin") then return "darwin" end
            return "linux"
        end,
        function() return vmrInstallerUnarchiver end
    )
end
```

### 7.6 安装配置

各字段含义见 §5。

#### `vmrNewInstallerConfig() → ic`

#### `vmrAddFlagFiles(ic, os, list) → ic`

#### `vmrEnableFlagDirExcepted(ic) → ic`

#### `vmrAddBinaryDirs(ic, os, list) → ic`

#### `vmrSetBinaryRename(ic, name_flag, rename_to) → ic`

#### `vmrAddAdditionalEnvs(ic, name, list, version) → ic`

### 7.7 pixi / conda-forge

#### `vmrSearchByPixi(list, package) → VersionList`

查询 pixi 后端使用的 conda-forge repodata，并为每个可用版本追加一个 `Item`。条目的
`os`/`arch` 为当前平台，`installer = "conda"`，没有 `url`。查询失败会打印警告并原样返回列表。
`package` 是 conda 包名。

`vmrSearchByPixi` 是当前名称；`vmrSearchByConda` 是行为完全相同的兼容别名（不会调用 conda 可执行
文件）。安装解析使用 `plugin_name`，因此 pixi 插件必须把 `plugin_name` 设为 conda 包名。

```lua
plugin_name = "lua"
sdk_name = "lua"
function crawl()
    return vmrSearchByPixi(vmrNewVersionList(), plugin_name)
end
```

### 7.8 工具函数

#### 平台

| 函数 | 结果 |
|---|---|
| `vmrGetOsArch()` | 两个字符串，采用 Go 命名：`linux`/`darwin`/`windows`，`amd64`/`arm64`/`386` |

#### 字符串与正则

| 函数 | 结果 |
|---|---|
| `vmrRegexpFindString(pattern, content)` | 第一个匹配，或 `""`；使用 Rust regex 语法（不支持 lookaround/反向引用） |
| `vmrHasPrefix(s, prefix)` | 布尔值 |
| `vmrHasSuffix(s, suffix)` | 布尔值 |
| `vmrContains(s, needle)` | 布尔值 |
| `vmrTrimPrefix(s, prefix)` | 删除开头所有重复出现的 `prefix` |
| `vmrTrimSuffix(s, suffix)` | 删除结尾所有重复出现的 `suffix` |
| `vmrTrim(s, cutset)` | 删除两端所有属于 `cutset` 的字符 |
| `vmrTrimSpace(s)` | 删除两端空白 |
| `vmrToLower(s)` | 转小写 |
| `vmrSplit(content, sep)` | 返回两个值：分段 table 和数量；`content` 或 `sep` 为空时返回空 table 和 `0` |
| `vmrSprintf(pattern, args?)` | 按顺序用 `args` 表中的值替换 `%s`；参数不足时保留字面 `%s` |
| `vmrUrlJoin(base, path)` | 用恰好一个 `/` 连接；`base` 为空返回 `""`；`path` 为空返回 `base` |
| `vmrPathJoin(base, path)` | 按文件系统路径连接；任一参数为空返回 `""` |
| `vmrLenString(s)` | 字节长度 |

```lua
local parts, n = vmrSplit("linux-amd64", "-")  -- {"linux", "amd64"}, 2
```

#### 环境与进程

| 函数 | 结果 |
|---|---|
| `vmrGetOsEnv(key)` | 环境变量值，或 `""` |
| `vmrSetOsEnv(key, value)` | 布尔成功标志；影响 vmr 进程及其子进程 |
| `vmrExecSystemCmd(collect_stdout, workdir, args)` | 返回 `stdout, ok`；`args` 是 table（`{ "git", "ls-remote", url }`）。非零退出或启动失败时 `ok = false`；`collect_stdout` 为 false 或失败时 `stdout` 为 `""` |

#### 文件与压缩包

| 函数 | 结果 |
|---|---|
| `vmrReadFile(path)` | 文件内容；出错或非 UTF-8 时返回 `""` |
| `vmrWriteFile(path, content)` | 布尔值；不会创建父目录 |
| `vmrCopyFile(src, dst)` | 布尔值 |
| `vmrCopyDir(src, dst)` | 布尔值 |
| `vmrCreateDir(dir)` | 布尔值；会创建父目录；空路径返回 `false` |
| `vmrRemoveAll(dir)` | 布尔值；路径已不存在也算成功 |
| `vmrUnarchive(src, dst, single_name, single_exe)` | 解压到 `dst`。单个压缩文件可用 `single_name` 指定输出名（空 = 源文件名）；`single_exe = true` 会加可执行权限。支持 zip、tar、gzip、bzip2、xz、zstd（包括单文件压缩） |

## 8. 常量

| 常量 | 值 |
|---|---|
| `vmrInstallerUnarchiver` | `"unarchiver"` |
| `vmrInstallerExecutable` | `"executable"` |
| `vmrInstallerConda` | `"conda"` |
| `vmrInstallerCondaForge` | `"conda-forge"` |
| `vmrInstallerCoursier` | `"coursier"` |
| `vmrInstallerDpkg` | `"dpkg"` |
| `vmrInstallerRpm` | `"rpm"` |

## 9. 旧的安装钩子

插件加载器仍识别 `install(path, version)` 和 `postInstall(path, version)`。
`Plugin::call_handler` 会把两个参数作为字符串传入，只有钩子返回字符串 `"true"` 时才视为成功。当前
CLI 的安装流程按 `Item.installer` 分派并执行内置后处理，不会自动调用这两个钩子。请把安装行为编码
到 `Item.installer` 和 `ic` 中；可移植插件不应依赖它们。

## 10. 共享辅助模块（`_vmr_common.lua`）

插件包自带共享辅助模块，加载方式：

```lua
local C = dofile(__vmr_plugin_dir .. "/_vmr_common.lua")
```

| Helper | 用途 |
|---|---|
| `C.github(cfg)` | 基于 `vmrGetGithubRelease` 的 GitHub Releases 爬取：`repo`、`tag_filter`、`version_parser`、`file_filter`、`os_parser`、`arch_parser`、`installer` |
| `C.pixi(name)` | 等价于 `vmrSearchByPixi(vmrNewVersionList(), name)` |
| `C.installer(cfg)` | 从 `flag_files`、`binary_dirs`、`flag_dir_excepted`、`additional_envs`、`binary_rename` 构造 `ic` |
| `C.match(value, rules)` | `contains`/`prefix`/`suffix` 资源过滤 |
| `C.lookup(value, pairs)` | 按顺序做模式 → 值映射 |
| `C.merge(a, b)` | 合并两个版本列表 |

辅助模块在 `gvcgo/vmr_plugins` 仓库中，不在本仓库。`vmr template` 会输出当前的标准写法，可作为
helper 选项的权威参考。

## 11. 测试与排查

```bash
# 语法检查（Lua 5.4）
lua5.4 -e 'assert(loadfile("plugins/example.lua"))'

# 查看元数据与 id
vmr plugins

# 忽略缓存，实时爬取当前平台
vmr search example -n

# 安装并切换
vmr use example@1.2.3
```

检查清单：

1. `plugin_name`、`sdk_name`、`homepage`、`ic`、`crawl` 均已定义。
2. `crawl()` 返回版本列表 userdata，而不是普通 table。
3. 每个 `Item` 的 `os`、`arch` 与资源匹配；只有确实通用时才用 `any`。
4. 资源 URL 指向可下载文件，而不是 release 页面。
5. 校验和使用 §3 中的 `sum_type`；拿不到校验和时同时省略 `sum` 和 `sum_type`。
6. `flag_files` 能匹配压缩包中真实存在的条目；安装后 `binary_dirs` 存在。
7. pixi 插件的 `plugin_name` 是 conda 包名。
8. 修改 `crawl()` 后删除 `~/.vmr/cache/<plugin_name>/` 或使用 `-n`。
