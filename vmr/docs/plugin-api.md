# VMR Lua Plugin API

Complete reference for writing a version-discovery plugin: plugin file format, lifecycle,
data model, and every `vmr*` Lua binding registered by the runtime.

- 简体中文: [plugin-api.zh-CN.md](plugin-api.zh-CN.md)
- Design notes and the mapping from the old `vcollector` crawlers: [plugin-system.md](plugin-system.md)

The Lua runtime is Lua 5.4 (`mlua`, vendored). Each plugin is executed in its own state, so
globals do not leak between plugins. The full Lua standard library, including `dofile`,
`os`, and `io`, is available.

## 1. File layout and discovery

| Item | Value |
|---|---|
| Plugin directory | `~/.vmr/plugins/` (`$HOME/.vmr/plugins`; always created) |
| Plugin files | `*.lua` directly inside that directory, loaded in file-name order |
| Helper files | Files whose name starts with `_` (`_vmr_common.lua`) are **not** treated as plugins |
| Installation directory | `<versions>/<sdk_name>_versions/<plugin_name>-<version>` |
| Active symlink | `<versions>/<sdk_name>_versions/<sdk_name>` |
| Version cache | `<cache>/<plugin_name>/<plugin_name>.versions.json` |

`<versions>` and `<cache>` default to `~/.vmr/versions` and `~/.vmr/cache`; they follow
`VMR_SDK_INSTALLATION_DIR` when it is set.

Plugins are not embedded in the binary. The first `Plugins::new()` that finds no `.lua` files
downloads `gvcgo/vmr_plugins` once per process. `vmr plugins -u` refreshes the directory and
`plugins.json` explicitly; same-name files are overwritten.

`vmr search` / `vmr use` resolve an SDK name first as `plugin_name`, then as `sdk_name`.

### Scaffolding

```bash
vmr template github              # print to stdout
vmr template json --dir ./plugins  # write ./plugins/json.lua
```

Available kinds: `github`, `html`, `json`, `pixi` (also accepted: `gh`, `conda`).

## 2. Plugin file format

A plugin is a plain Lua script. It assigns global variables and defines `crawl()`.

### Required globals

| Global | Type | Meaning |
|---|---|---|
| `plugin_name` | string | Stable plugin id. Used by `vmr <sdk>` lookup, the cache directory, install directory, and the pixi package name. Must be non-empty. |
| `sdk_name` | string | SDK id used by `vmr use <sdk>@<version>`, the install directory, and the active symlink. Must be non-empty. |
| `homepage` | string | Upstream homepage, shown in `vmr plugins`. Must be non-empty. |
| `ic` | `InstallerConfig` userdata | Installer/environment configuration. Build it with `vmrNewInstallerConfig()` and the `vmrAdd*`/`vmrSet*` functions. |
| `crawl` | function | Called with no arguments; returns a `VersionList`. |

### Optional globals

| Global | Type | Meaning |
|---|---|---|
| `plugin_version` | string | Plugin revision shown by `vmr plugins`. Not used for sorting or installs. |
| `prequisite` | string | Legacy metadata; parsed but not enforced by the current CLI. |
| `install(path, version)` | function | Legacy custom-install hook. See §9. |
| `postInstall(path, version)` | function | Legacy post-install hook. See §9. |

A metadata global may also be a zero-argument function returning a string:

```lua
plugin_version = function() return "0.2" end
```

Load validation fails if `plugin_name`, `sdk_name`, or `homepage` is empty, or if `ic` or
`crawl` is missing. `crawl` is additionally checked to be callable when versions are queried.

### Minimal skeleton

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

`__vmr_plugin_dir` is an absolute path injected before the script runs; use it with `dofile`
to load shared helpers.

### `crawl()` contract

- Takes no arguments.
- Returns a `VersionList` userdata, normally from `vmrNewVersionList()`,
  `vmrGetGithubRelease()`, `vmrSearchByPixi()`, or `vmrMergeVersionList()`.
- Returning an empty list is valid; the cache is not written, so the next lookup crawls again.
- Errors and non-`VersionList` return values are reported by `vmr search` / `vmr use`.
  `crawl()` must not call `vmr use` or mutate the installation.

## 3. Data model

### `VersionList`

A keyed set: version string → list of candidate `Item`s. All `vmr*` functions that return a
version list return the same userdata type, so their results can be merged or returned directly
from `crawl()`.

```lua
vl = vmrNewVersionList()
vl = vmrAddItem(vl, "1.2.3", item)
vl = vmrMergeVersionList(vl, other_vl)
```

### `Item`

Passed as the third argument of `vmrAddItem` (a plain Lua table):

| Field | Type | Meaning |
|---|---|---|
| `url` | string | Download URL. Required for `unarchiver`, `executable`, `dpkg`, `rpm`. |
| `os` | string | `linux`, `darwin`, `windows`, `any`, `unix`; `macos`/`osx` are aliases for `darwin`. |
| `arch` | string | `amd64`, `arm64`, `386`, or `any`. |
| `sum` | string | Expected checksum; empty disables verification. |
| `sum_type` | string | `md5`, `sha1`, `sha256`, `sha512`; other values disable checksum verification. |
| `size` | integer | Expected byte size; `0` disables the size check. |
| `installer` | string | Installer kind from §3.1. Unknown values fall back to `unarchiver`. |
| `lts` | string | Metadata only; kept for compatibility, not used by installs. |
| `extra` | string | Metadata only; kept for compatibility, not used by installs. |

Missing fields are read as empty strings / `0`.

#### 3.1 Installer kinds

| Item value | Constant | Behavior |
|---|---|---|
| `unarchiver` | `vmrInstallerUnarchiver` | Download, extract, locate the archive root via `ic.flag_files`, copy it into the install directory. Default for unknown kinds. |
| `executable` | `vmrInstallerExecutable` | Download one file, copy it into the install directory and mark it executable; honors `ic.binary_rename`. |
| `conda` | `vmrInstallerConda` | Install through the embedded pixi/rattler backend; the package name is `plugin_name`. `url`/`sum` are ignored. |
| `conda-forge` | `vmrInstallerCondaForge` | Same backend as `conda`. |
| `coursier` | `vmrInstallerCoursier` | Run the `cs` executable (`VMR_COURSIER_PATH`, default `cs`) with `--install-dir`; requires `cs` on the machine. |
| `dpkg` | `vmrInstallerDpkg` | Treated as a single downloaded file (executable-style handling). |
| `rpm` | `vmrInstallerRpm` | Treated as a single downloaded file (executable-style handling). |

For `conda`/`conda-forge`/pixi, the install directory is named after `plugin_name`, and
`vmr-pixi` resolves `plugin_name` in the conda-forge repodata. A pixi-backed plugin must
therefore use the conda package name as `plugin_name`.

### 3.2 Platform filtering

`crawl()` returns every candidate; VMR keeps only entries matching the current platform:

| Item `os` | Matches |
|---|---|
| `any` | every OS |
| `unix` | `linux` and `darwin` |
| `macos`, `osx` | `darwin` |
| anything else | exact match (`linux`, `darwin`, `windows`) |

Item `arch` matches exactly or is `any`.

If several items for one version match, the **last** matching item wins. Emit the most specific
asset last, or emit only one item per platform.

### 3.3 Version names and ordering

The map key is the exact version string accepted by `vmr use <sdk>@<version>`. There is no
normalization; strip a leading `v` yourself if necessary. Lookups sort version keys with VMR's
semver comparator, newest first; `@latest` selects the first result.

## 4. Lifecycle and cache

```
Plugins::new()
  └─ missing .lua files → download gvcgo/vmr_plugins (once per process)
load_all()
  └─ execute each plugin → read metadata → validate plugin_name/sdk_name/homepage/ic/crawl
get_sdk_versions()
  ├─ fresh cache? → return ~/.vmr/cache/<plugin>/<plugin>.versions.json
  └─ otherwise crawl → filter current os/arch → save cache
use → pick Item → install by Item.installer → post-processing → env injection
```

- The cache stores one `Item` per version, for the current platform only.
- A cache file is considered fresh while its age is below the configured
  `cache_retention_time` (default 86400 seconds) and caching is enabled.
- `vmr search <sdk> -n` (and `--no-cache`) ignores the cache, crawls, and rewrites it.
- The cache is keyed by `plugin_name`, not `sdk_name`.
- Empty crawl results are not cached.

## 5. Installer configuration (`ic`)

`ic` is an `InstallerConfig` userdata. All builder functions return the same userdata, so calls
can be chained or reassigned.

| Function | Effect |
|---|---|
| `vmrNewInstallerConfig()` | Create an empty config. |
| `vmrAddFlagFiles(ic, os, list)` | Append archive-root marker names for one platform (`""` = all three). |
| `vmrEnableFlagDirExcepted(ic)` | When locating the archive root, ignore directory names and match file names only. |
| `vmrAddBinaryDirs(ic, os, list)` | Append **one** relative path group (a bin directory) for one platform (`""` = all three). |
| `vmrSetBinaryRename(ic, name_flag, rename_to)` | Rename an installed executable whose file name contains `name_flag` to `rename_to` (`.exe` is appended on Windows). Used by the `executable` installer. |
| `vmrAddAdditionalEnvs(ic, name, list, version)` | Add one environment variable pointing at one path group beneath the install root. `version` is stored but not used when environments are collected. |

How the fields are consumed:

- `flag_files` (`windows`/`linux`/`darwin`): substring-matched against the concatenated entry
  names of each directory after extraction. The first directory whose entries contain **all**
  flags becomes the install root. Empty flags match the extraction root itself. With
  `flag_dir_excepted`, directory entries are excluded from matching.
- `binary_dirs`: each group is joined under the install root and added to `PATH` if it exists.
  When a platform has no groups, the install root itself is added.
- `additional_envs`: the path group is joined under the install root and exported as
  `NAME=<absolute path>` if it exists. An empty group points at the install root (e.g. `GOROOT`).
- `binary_rename` applies only to the `executable` installer.

Example:

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

Platform argument behavior:

| `os` value | Effect |
|---|---|
| `""` | Append to `windows`, `linux`, and `darwin` |
| `"windows"` / `"linux"` / `"darwin"` | Append to that platform |
| anything else | No change; the config is returned unchanged |

## 6. Runtime helpers

| Global | Meaning |
|---|---|
| `__vmr_plugin_dir` | Absolute plugin directory; intended for `dofile(__vmr_plugin_dir .. "/_vmr_common.lua")`. |
| `print` | Under the MCP stdio server, writes to stderr instead of stdout. |

## 7. API reference

### 7.1 HTTP

#### `vmrGetResponse(url [, timeout [, headers]]) → body`

GET a URL. `timeout` is in seconds (default `180` when omitted or `<= 0`). `headers` is an
optional table of string keys to string values.

Returns an opaque userdata holding the response body. Pass it directly to
`vmrInitSelection` or `vmrInitGJson`; it is not a Lua string. On network error, non-2xx
status, or an empty body, the returned userdata holds an empty string.

```lua
local resp = vmrGetResponse("https://example.com/releases.json", 60, {
    ["Accept"] = "application/json",
})
local json = vmrInitGJson(resp)
```

#### `vmrGetProxy() → scheme, host, port`

Returns the proxy parts from `conf.toml` (`~/.vmr/conf.toml`, else `~/.config/vmr/conf.toml`;
key `ProxyUri`). When it is empty or invalid:
`""`, `""`, `"0"`; a URI without a port yields an empty port string.

### 7.2 HTML selection

A selection is an opaque userdata holding one parsed document and the positions of the selected
nodes. Operations stay node-relative.

#### `vmrInitSelection(body, selector) → selection`

Parse `body` (the userdata from `vmrGetResponse`, or a Lua string) and select CSS matches.
An empty body or invalid selector yields an empty selection.

#### `vmrFind(selection, selector) → selection`

Select descendants of every currently selected node.

#### `vmrEq(selection, index) → selection`

Pick the node at zero-based `index`. Negative or out-of-range indices yield an empty selection.

#### `vmrAttr(selection, name) → string`

Attribute of the first selected node, or `""`.

#### `vmrText(selection) → string`

Concatenate the text of all descendants of every selected node.

#### `vmrEach(selection, callback)`

Call `callback(index, node_selection)` for every selected node. `index` is zero-based;
`node_selection` is a one-node selection suitable for `vmrAttr`/`vmrText`/`vmrFind`.

```lua
vmrEach(vmrInitSelection(resp, "ul.releases li a"), function(_, node)
    local href = vmrAttr(node, "href")
    local version = vmrTrimPrefix(vmrText(node), "v")
end)
```

### 7.3 JSON

`vmrInitGJson` wraps a response body. Paths are dot-separated object keys and array indices:
`"releases.0.version"`. An empty path addresses the root. Missing values return `""` / `0`.

#### `vmrInitGJson(body) → json`

Parse `body` (userdata from `vmrGetResponse`, or a Lua string). Invalid JSON becomes `null`;
lookups then return empty values.

#### `vmrGetString(json, path) → string`

Coerce the value at `path`: strings as-is, numbers and booleans stringified, objects/arrays
serialized as JSON, `null`/missing as `""`.

#### `vmrGetInt(json, path) → integer`

Integer conversion; missing or unparsable values become `0`.

#### `vmrGetByKey(json, path, key) → string`

Resolve `path` to an object, then read `key` from it and stringify as in `vmrGetString`.

#### `vmrMapEach(json, path, callback)`

Iterate an object at `path`: `callback(key, value_json)`. `value_json` is another JSON userdata,
so `vmrGetString(value_json, "url")` works. Non-objects are a no-op.

#### `vmrSliceEach(json, path, callback)`

Iterate an array at `path`: `callback(index, value_json)` with **one-based** indices. `null`
elements are skipped. Non-arrays are a no-op.

#### `vmrGetByIndex(json, path, index) → string`

Read array element `index` (one-based) at `path` and stringify it. `index < 1`, a non-array, or
an out-of-range index yields `""`.

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

### 7.4 Version lists

#### `vmrNewVersionList() → VersionList`

Create an empty list.

#### `vmrAddItem(list, version, item?) → VersionList`

Append `item` (a table; see §3) under `version`. Returns the same list, so calls can be chained.
An empty `version` is ignored. Calling it without a table records no item.

#### `vmrMergeVersionList(destination, source) → VersionList`

Append every item of `source` to `destination` and return `destination`.

### 7.5 GitHub Releases

#### `vmrGetGithubToken() → string`

Returns the configured GitHub token: `GithubToken` from `conf.toml` (`~/.vmr/conf.toml`, else
`~/.config/vmr/conf.toml`), else
`VMR_GITHUB_TOKEN`, else `GITHUB_TOKEN`, else `""` (anonymous, 60 requests/hour).

#### `vmrGetGithubRelease(repo, tag_filter, version_parser, file_filter, arch_parser, os_parser, installer_getter) → VersionList`

Fetch all release pages for `repo` (`"owner/repo"`) and filter them through six callbacks:

| Callback | Called with | Expected result |
|---|---|---|
| `tag_filter(tag)` | Release tag (`tag_name`) | Boolean; only true tags continue |
| `version_parser(tag)` | Release tag | Version string; empty skips the release |
| `file_filter(asset_name)` | Asset file name | Boolean; only true assets continue |
| `arch_parser(asset_name)` | Asset file name | `amd64`/`arm64`/`386`/`any`; empty skips the asset |
| `os_parser(asset_name)` | Asset file name | `linux`/`darwin`/`windows`/`any`/`unix`; empty skips the asset |
| `installer_getter(asset_name)` | Asset file name | Installer kind string from §3.1 |

Each accepted asset becomes an `Item` with `url` = `browser_download_url` and `size` from the
API. Assets whose URL contains `archive/refs/` are always skipped. Callback errors, `nil`, or
wrong types are treated as `false`/`""`. Network and API errors produce an empty list rather
than raising. The result is a `VersionList` and can be returned directly from `crawl()` or
merged.

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

### 7.6 Installer configuration

See §5 for the meaning of each field.

#### `vmrNewInstallerConfig() → ic`

#### `vmrAddFlagFiles(ic, os, list) → ic`

#### `vmrEnableFlagDirExcepted(ic) → ic`

#### `vmrAddBinaryDirs(ic, os, list) → ic`

#### `vmrSetBinaryRename(ic, name_flag, rename_to) → ic`

#### `vmrAddAdditionalEnvs(ic, name, list, version) → ic`

### 7.7 pixi / conda-forge

#### `vmrSearchByPixi(list, package) → VersionList`

Query the conda-forge repodata used by the pixi backend and append one `Item` per available
version. Each item has the current `os`/`arch`, `installer = "conda"`, and no `url`. Search
failures print a warning and return the list unchanged. `package` is the conda package name.

`vmrSearchByPixi` is the current name; `vmrSearchByConda` is a compatibility alias with identical
behavior (no conda executable is invoked). Install resolution uses `plugin_name`, so a pixi
plugin must set `plugin_name` to the conda package name.

```lua
plugin_name = "lua"
sdk_name = "lua"
function crawl()
    return vmrSearchByPixi(vmrNewVersionList(), plugin_name)
end
```

### 7.8 Utilities

#### Platform

| Function | Result |
|---|---|
| `vmrGetOsArch()` | Two strings, Go naming: `linux`/`darwin`/`windows`, `amd64`/`arm64`/`386` |

#### Strings and regex

| Function | Result |
|---|---|
| `vmrRegexpFindString(pattern, content)` | First match, or `""`; Rust regex syntax (no lookaround/backreferences) |
| `vmrHasPrefix(s, prefix)` | Boolean |
| `vmrHasSuffix(s, suffix)` | Boolean |
| `vmrContains(s, needle)` | Boolean |
| `vmrTrimPrefix(s, prefix)` | Remove all leading occurrences of `prefix` |
| `vmrTrimSuffix(s, suffix)` | Remove all trailing occurrences of `suffix` |
| `vmrTrim(s, cutset)` | Trim any characters contained in `cutset` from both ends |
| `vmrTrimSpace(s)` | Trim leading/trailing whitespace |
| `vmrToLower(s)` | Lowercase |
| `vmrSplit(content, sep)` | Returns two values: a table of parts and the count. Empty `content` or `sep` yields an empty table and `0` |
| `vmrSprintf(pattern, args?)` | Replace `%s` placeholders sequentially from the `args` table; extra placeholders stay literal `%s` |
| `vmrUrlJoin(base, path)` | Join with exactly one `/`; empty `base` yields `""`; empty `path` returns `base` |
| `vmrPathJoin(base, path)` | Join as a filesystem path; either argument empty yields `""` |
| `vmrLenString(s)` | Byte length |

```lua
local parts, n = vmrSplit("linux-amd64", "-")  -- {"linux", "amd64"}, 2
```

#### Environment and processes

| Function | Result |
|---|---|
| `vmrGetOsEnv(key)` | Environment value, or `""` |
| `vmrSetOsEnv(key, value)` | Boolean success; affects the vmr process and its children |
| `vmrExecSystemCmd(collect_stdout, workdir, args)` | Returns `stdout, ok`; `args` is a table (`{ "git", "ls-remote", url }`). `ok = false` on a non-zero exit or spawn failure; `stdout` is `""` when `collect_stdout` is false or on failure |

#### Files and archives

| Function | Result |
|---|---|
| `vmrReadFile(path)` | File contents, or `""` on error/non-UTF-8 |
| `vmrWriteFile(path, content)` | Boolean; parent directories are not created |
| `vmrCopyFile(src, dst)` | Boolean |
| `vmrCopyDir(src, dst)` | Boolean |
| `vmrCreateDir(dir)` | Boolean; creates parents; empty path returns `false` |
| `vmrRemoveAll(dir)` | Boolean; succeeds when the path no longer exists |
| `vmrUnarchive(src, dst, single_name, single_exe)` | Extract an archive into `dst`. For a single compressed file, `single_name` overrides the output name (empty = source name); `single_exe = true` adds the executable bit. Supports zip, tar, gzip, bzip2, xz, and zstd (including compressed single files) |

## 8. Constants

| Constant | Value |
|---|---|
| `vmrInstallerUnarchiver` | `"unarchiver"` |
| `vmrInstallerExecutable` | `"executable"` |
| `vmrInstallerConda` | `"conda"` |
| `vmrInstallerCondaForge` | `"conda-forge"` |
| `vmrInstallerCoursier` | `"coursier"` |
| `vmrInstallerDpkg` | `"dpkg"` |
| `vmrInstallerRpm` | `"rpm"` |

## 9. Legacy install hooks

The plugin loader still recognizes `install(path, version)` and `postInstall(path, version)`.
`Plugin::call_handler` passes both arguments as strings and treats the hook as successful only
when it returns the string `"true"`. The current CLI install pipeline dispatches on
`Item.installer` and runs built-in post-processing, so these hooks are not invoked automatically.
Prefer encoding install behavior in `Item.installer` and `ic`; do not rely on the hooks for
portable plugins.

## 10. Shared helpers (`_vmr_common.lua`)

The plugin bundle ships a shared helper module. Load it with:

```lua
local C = dofile(__vmr_plugin_dir .. "/_vmr_common.lua")
```

| Helper | Purpose |
|---|---|
| `C.github(cfg)` | GitHub Releases crawler built on `vmrGetGithubRelease`: `repo`, `tag_filter`, `version_parser`, `file_filter`, `os_parser`, `arch_parser`, `installer` |
| `C.pixi(name)` | Returns `vmrSearchByPixi(vmrNewVersionList(), name)` |
| `C.installer(cfg)` | Builds `ic` from `flag_files`, `binary_dirs`, `flag_dir_excepted`, `additional_envs`, `binary_rename` |
| `C.match(value, rules)` | `contains`/`prefix`/`suffix` asset filtering |
| `C.lookup(value, pairs)` | Ordered pattern → value mapping |
| `C.merge(a, b)` | Merge two version lists |

The helper module lives in `gvcgo/vmr_plugins`, not in this repository. `vmr template` emits the
current canonical shape; use it as the source of truth for helper options.

## 11. Testing and debugging

```bash
# Syntax check (Lua 5.4)
lua5.4 -e 'assert(loadfile("plugins/example.lua"))'

# Metadata and ids
vmr plugins

# Live crawl for the current platform, bypassing the cache
vmr search example -n

# Install and switch
vmr use example@1.2.3
```

Checklist:

1. `plugin_name`, `sdk_name`, `homepage`, `ic`, and `crawl` are all defined.
2. `crawl()` returns a version-list userdata, not a plain table.
3. Every `Item` has `os` and `arch` matching the asset; use `any` only when true.
4. Asset URLs point to downloadable files, not release pages.
5. Checksums use `sum_type` values from §3; omit both `sum` and `sum_type` when unavailable.
6. `flag_files` match entries actually present in the archive; `binary_dirs` exists after
   installation.
7. For pixi plugins, `plugin_name` is the conda package name.
8. Delete `~/.vmr/cache/<plugin_name>/` or use `-n` after changing `crawl()`.
