# VMR Lua Plugin System

## 1. Goal

Each SDK is described by a Lua plugin:

- the plugin's `crawl()` fetches the upstream source at runtime;
- VMR caches the result for the current platform.

VMR neither consumes nor produces the pre-generated `<sdk>.version.json` files that
`vcollector` used to publish; the plugins are the only version source.

## 2. Runtime contract

A plugin is a `.lua` file loaded in an isolated Lua state. It must define:

| Global | Meaning |
|---|---|
| `plugin_name` | stable plugin id |
| `sdk_name` | SDK id used under `versions/` |
| `plugin_version` | plugin revision, e.g. `0.1` |
| `homepage` | SDK homepage |
| `ic` | installer config (`vmrNewInstallerConfig()` chain) |
| `crawl()` | returns a `VersionList` (`vmrNewVersionList()` / `vmrAddItem`) |

Optional globals: `install(path, version)` and `postInstall(path, version)` custom handlers.

A version item contains:

```lua
{
  url, arch, os, sum, sum_type, size, installer, lts, extra
}
```

`installer` is one of `unarchiver`, `executable`, `coursier`, `conda`, `conda-forge`,
`dpkg`, `rpm`. `conda`/`conda-forge` items are installed by VMR's embedded pixi/rattler
backend; no conda executable is used.

## 3. Lifecycle

```
Plugins::new
  ├─ if no .lua files: fetch upstream plugin archive
  └─ load_all: execute each .lua, read metadata/ic, validate crawl
        │
        ├─ vmr plugins     → plugin metadata rows
        ├─ vmr search sdk  → crawl → filter current os/arch (cached)
        └─ vmr use sdk@v   → install via installer kind
```

Version cache:

```
~/.vmr/cache/<plugin>/<plugin>.versions.json   # current platform only
```

Platform matching accepts vcollector's `any` and `unix` values: `arch=any` matches any
architecture, `os=unix` matches Linux and macOS, and `macos`/`osx` are aliases for `darwin`.
This keeps single-binary and JVM-style assets installable.

## 4. Shared helpers (`plugins/_vmr_common.lua`)

Plugins load helpers with:

```lua
local C = dofile(__vmr_plugin_dir .. "/_vmr_common.lua")
```

| Helper | Purpose |
|---|---|
| `C.github(cfg)` | GitHub Releases; tag/file filters, version parser, OS/arch mappings, installer |
| `C.pixi(name)` | conda-forge/pixi package versions (`vmrSearchByPixi`; old `vmrSearchByConda` alias) |
| `C.installer(cfg)` | build `ic` from flag files, binary dirs, envs, rename |
| `C.match(value, rules)` | `contains`/`prefix`/`suffix` asset filtering |
| `C.lookup(value, pairs)` | ordered pattern → value mapping |
| `C.merge(a, b)` | merge two version lists |

For unusual sources a plugin can use the raw bindings directly:
`vmrGetResponse`, `vmrInitSelection`, `vmrFind/Eq/Attr/Text/Each`,
`vmrInitGJson`, `vmrGetString/GetInt/GetByKey/MapEach/SliceEach/GetByIndex`,
`vmrSprintf`, `vmrUrlJoin`, `vmrPathJoin`, `vmrUnarchive`, etc.

Custom GitHub-API crawlers can read `vmrGetGithubToken()`; VMR uses the configured
`GithubToken`, else `VMR_GITHUB_TOKEN`/`GITHUB_TOKEN`, else anonymous access (no credential is
embedded in the binary).

## 5. Mapping from vcollector

| vcollector family | Plugin approach |
|---|---|
| `pkgs/crawlers/conda/*` | `C.pixi("<package>")`; install/solve through pixi/rattler |
| `pkgs/crawlers/gh/tools/*` | `C.github` with per-tool filters and installer config |
| `pkgs/crawlers/gh/lans/*` | `C.github` + language-specific asset mapping |
| `pkgs/crawlers/gh/lsp/*` | `C.github` + LSP binary mapping |
| `pkgs/crawlers/official/*` | live JSON/HTML/API parsing via VMR bindings |
| `pkgs/crawlers/mix/*`, `official/fixed/*` | live HTML/API/direct-URL crawl |

Conda-family plugins intentionally use `C.pixi`, per the project direction that conda
installs are replaced by pixi. The full vcollector SDK catalog (82 SDK names) has a
corresponding plugin in the separate `gvcgo/vmr_plugins` repository.

## 6. Plugin source and updates

Plugins are not embedded into the binary. `~/.vmr/plugins` is populated from the
`gvcgo/vmr_plugins` archive when it contains no `.lua` files, and `vmr plugins -u`
refreshes it (downloading the archive and overwriting same-name files). The plugin
sources live in the separate `vmr_plugins` repository.

## 7. Adding a plugin

1. Create `plugins/<sdk>.lua` — `vmr template <github|html|json|pixi> --dir <dir>` prints a
   starting point for the common upstream shapes.
2. Load `_vmr_common.lua`.
3. Set metadata and `ic`.
4. Implement `crawl()` against the live upstream.
5. Syntax check: `lua5.4 -e 'assert(loadfile("plugins/<sdk>.lua"))'`.
6. Test: `vmr plugins`, `vmr search <sdk>`, `vmr use <sdk>@<version>`.
