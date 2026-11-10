<p align="center">
  <img src="docs/brand/logo.png" alt="vmr" width="640">
</p>

# vmr (Rust)

A cross-platform SDK version manager rewritten in Rust, CLI only (no TUI). Commands and the
on-disk layout mirror `../version-manager/vmr-go`; version discovery is still performed by Lua
plugins.

The package installation backend has moved from **Miniconda/conda** in the Go version to
**pixi's Rust components**: dependency solving uses `rattler_solve` and prefix installation uses
`rattler::install::Installer` (the same underlying implementation pixi uses). conda/miniconda is
therefore no longer required on the host.

## Features

- Install, switch, and uninstall `sdk@version`; the active version is represented by the
  `<versions>/<sdk>_versions/<sdk>` symlink.
- Lua plugins discover versions (HTML scraping, GitHub Release API, pixi/conda-forge sources).
- Multiple versions coexist; `vmr local` reads directly from disk.
- Project-level version pinning: `vmr use -l` writes `.vmr.lock`, and `vmr cdhook` is invoked by
  the shell's cd hook.
- Session mode: `vmr use -s` injects environment variables into a child shell temporarily and
  restores them on exit.
- Global environment injection: on Unix-like platforms it writes `~/.vmr/vmr.sh` / `vmr.fish`
  (dynamic environment lines only); on Windows it writes the `HKCU\Environment` registry key and
  broadcasts the change. Windows `PATH` keeps a single entry (`~/.vmr/bin`): each active SDK
  command is a shim (a hardlink to `vmr.exe`, or a generated `.cmd` wrapper) that injects the
  prefix's `PATH` into the child, so the registry value stays constant regardless of how many
  SDKs or pixi packages are installed. Shell configuration needs only one line,
  `eval "$(vmr init zsh)"` (fish: `vmr init fish | source`).
- Automatic refresh (bash/zsh/fish): that line also defines a `vmr` wrapper, so a successful
  `vmr use` or `vmr uninstall` reloads the environment into the current shell instead of
  requiring a new terminal. The refresh removes vmr-managed `PATH` entries and re-applies the
  env file, so repeated switches do not stack duplicates; variables vmr owns (e.g. `GOROOT`) are
  unset first, so values removed by an uninstall do not linger. `command vmr ...` bypasses the
  wrapper.
- Windows PowerShell: `vmr init powershell` also defines `vmrsource` and wraps `vmr`, so
  `vmr use` / `vmr uninstall` refresh the current window's `PATH` (and vmr-managed variables such
  as `GOROOT`) immediately instead of requiring a new terminal. `vmrsource` can be run by hand to
  re-read `HKCU\Environment`; unrelated `PATH` entries added by the current session are kept.
- Multi-threaded downloads, checksums, reverse proxies, local proxies, and `HTTP(S)_PROXY`
  environment proxies.
- All network downloads show `indicatif` progress bars: SDK packages, plugin bundles, pixi
  repodata, and pixi packages.
- Dotted spinners while parsing/searching pixi repodata, calling the GitHub API, or issuing Lua
  plugin HTTP requests.
- Any conda-forge package can be installed through the pixi backend
  (`vmr use -p <pkg>@<version>`).

## Build

```bash
cargo build --release
./target/release/vmr --help
```

Optionally, put the binary in a directory that is already on `PATH` and add one initialization
line to your shell config.

```bash
install -m755 target/release/vmr ~/.local/bin/vmr
echo 'eval "$(vmr init zsh)"' >> ~/.zshrc
```

`vmr init <shell>` supports `bash`, `zsh`, `fish`, and `powershell`; when omitted, the shell is
inferred from `$SHELL` (on Windows, where PowerShell/cmd do not set it, the default is
`powershell`; git-bash still reports `bash`). Shell completions are emitted by `vmr init`
itself, so no separate install step is needed. On zsh, if `compinit` has not run yet (or
`vmr init` is evaluated before `compinit`), completion registration is deferred to the first
prompt and `compinit` is initialized automatically when needed.

### Windows MSI

```powershell
powershell -ExecutionPolicy Bypass -File packaging\windows\build-msi.ps1
```

Builds `target\windows\vmr-<version>-x64.msi`: a per-machine install under `Program Files\vmr`
that also adds the install directory to the machine `PATH`. The first run downloads the portable
WiX Toolset v3.14 binaries into `target\wix314` (no admin install needed). Pass `-SkipBuild` to
package the existing `target\release\vmr.exe`.

## Commands

| Command | Alias | Description |
|---|---|---|
| `version` | `v` | Show the version |
| `init [shell]` | | Print the shell integration script (`eval "$(vmr init zsh)"`) |
| `template <kind>` | `-d/--dir` | Print a plugin template: `github` / `html` / `json` / `pixi` |
| `plugins [-u]` | | List plugins and the SDKs they support; `-u`/`--update-all` refreshes `gvcgo/vmr_plugins` first |
| `search <sdk>` | `s` | List versions available for the current platform; `-p`/`--pixi` uses the pixi package source (`--conda` is a compatible alias); `-n`/`--no-cache` ignores caches (plugin `<sdk>.versions.json` and pixi repodata) |
| `use <sdk>@<version>` | `u`/`h` | Install and switch (global by default) |
| `use -s/--session ...` | | Apply to the current new session only |
| `use -l/--lock ...` | | Write the project `.vmr.lock` and enter a new session |
| `cdhook` | | cd hook: read `.vmr.lock` and inject the environment (called by the shell hook) |
| `use -p ...` | | Install any package through the pixi backend (also `--pixi`/`--conda`) |
| `use -f/--file <file>` | | Install in bulk from a freeze file (one `sdk@version` per line, optional `pixi` suffix) |
| `freeze [file]` | `fz` | Export installed SDKs/versions to `vmr_<YYYYMMDD>.txt` (written to the current directory by default) |
| `mcp` | | Run an MCP server over stdio (Model Context Protocol) |
| `local [sdk]` | `l` | Installed versions, with `<current>` marking the active one; without `sdk`, list every installed SDK grouped |
| `local -c [sdk]` | | Only the active version: with `sdk`, print that SDK's active version; without, print the `SDK/CURRENT` table for all SDKs; prints nothing when there is no result |
| `uninstall <sdk>@<version>` | `uni`/`r` | Uninstall; `@all` removes every version of the SDK |
| `nested` | `ns` | Allow/forbid nested sessions |
| `session` | `se` | Query whether the shell is in a vmr session |

### Examples

```bash
vmr plugins
vmr search go
vmr use go@1.22.0
vmr local go
vmr uninstall go@1.22.0

# Project pinning
cd my-project
vmr use -l node@20.11.0
vmr cdhook          # usually run automatically by the shell's cd hook

# Any pixi/conda-forge package
vmr search -p lua
vmr use -p lua@5.4.6

# Migration/cloning: export local versions and reinstall on a new machine in one command
vmr freeze                      # -> vmr_20261104.txt
vmr use -f vmr_20261104.txt
```

`vmr freeze` writes a plain-text file similar to `requirements.txt`: lines starting with `#` are
comments, and each remaining line is `sdk@version`; SDKs with no Lua plugin that can only use the
pixi backend get a `pixi` suffix. The active version of each SDK is placed last, so a sequential
`vmr use -f` leaves the same versions active as when the file was exported. `-f` cannot be
combined with `<sdk>@<version>`, `-s`, `-l`, or `-p`.

## MCP

`vmr mcp` runs a Model Context Protocol server over stdio: stdin carries one JSON-RPC message per
line, stdout carries protocol messages only, and logs/progress go to stderr.

```json
{
  "mcpServers": {
    "vmr": { "command": "vmr", "args": ["mcp"] }
  }
}
```

It implements both protocol eras: the stateless `2026-07-28` revision (`server/discover`,
per-request `_meta`, `resultType`) and the legacy `initialize` handshake (`2025-06-18` and
earlier). Clients automatically use the path that matches their own version.

| Tool | Description |
|---|---|
| `vmr_list_plugins` | List local plugins and their SDKs |
| `vmr_search_versions` | Query available versions for an SDK (`backend`: `plugin` / `pixi`) |
| `vmr_list_installed` | Installed versions and the active version (omit `sdk` to list everything) |
| `vmr_install` | Install and globally switch `sdk@version` (supports `latest`) |
| `vmr_uninstall` | Uninstall a single version or `@all` |
| `vmr_freeze` | Export a freeze file |
| `vmr_env` | Return the `PATH` directories and environment variables of the active version so clients can run SDK binaries directly |

Note: the MCP server cannot modify the client's process environment. `vmr_install` only updates
global files such as `~/.vmr/vmr.sh`, which take effect in newly opened shells; to run SDK binaries
inside the client, use the values returned by `vmr_env`. `tools/call` executes synchronously and
does not send progress notifications, so installing large packages may exceed the client's
timeout; clients should confirm write operations with the user before invoking them.

## Disk layout

```
~/.vmr/
├── conf.toml                 # Configuration (PascalCase keys)
├── vmr.sh | vmr.fish         # Dynamic env lines (PATH/GOROOT); the cd hook is emitted by vmr init
├── env_keys.txt              # non-PATH variables cleared before an env refresh
├── bin/                      # Windows shims (the only vmr PATH entry)
├── shims.json                # Windows shim → real executable mapping
├── plugins/*.lua             # SDK plugins
├── cache/
│   ├── <plugin>/<version>/   # Legacy plugin download cache
│   ├── pixi_repodata/        # zstd-compressed repodata cache
│   └── pixi_pkgs/            # pixi/rattler package cache
├── temp/
└── versions/
    └── <sdk>_versions/
        ├── <plugin>-<version>/   # Actual install directory (conda-meta lives here too)
        └── <sdk> -> <plugin>-<version>
```

`VMR_SDK_INSTALLATION_DIR` can move `versions/` and `cache/` elsewhere.

## pixi backend

- The channel defaults to `https://conda.anaconda.org/conda-forge`.
- `VMR_PIXI_CHANNEL` overrides the channel (the legacy `VMR_CONDA_CHANNEL` is still accepted).
  Example mirrors:

  ```bash
  export VMR_PIXI_CHANNEL=https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/conda-forge
  # or https://mirrors.bfsu.edu.cn/anaconda/cloud/conda-forge
  # or https://mirrors.ustc.edu.cn/anaconda/cloud/conda-forge
  vmr search -p lua
  vmr use -p lua@5.4.6
  ```

  The variable replaces both repodata and package download URLs. pixi's own `[mirrors]` config is
  not read because vmr-pixi embeds rattler directly instead of launching the pixi process.
- Version queries and installs both read the current platform's repodata and append `noarch` as
  needed.
- Solving is done by `rattler_solve` (resolvo) and installation by `rattler::install::Installer`,
  matching pixi's handling of symlinks, noarch, prefix rewriting, and `conda-meta`.
- conda/miniconda is not required on the host; package caches live in `~/.vmr/cache/pixi_pkgs`.

The name `vmrSearchByConda` in Lua plugins is kept for compatibility with existing plugins; it
actually queries the pixi channel described above.

## Configuration and environment variables

`~/.vmr/conf.toml`:

```toml
ProxyUri = ""
ReverseProxy = ""
SDKIntallationDir = ""
VersionHostUrl = ""
ThreadNum = 1
AllowNestedSessions = false
GithubToken = ""
CacheRetentionTime = 86400
DisableCache = false
```

Common environment variables:

| Variable | Description |
|---|---|
| `VMR_SDK_INSTALLATION_DIR` | SDK/versions root directory |
| `VMR_HOST` | Version metadata host |
| `VMR_LOCAL_PROXY` | Local proxy |
| `VMR_REVERSE_PROXY` | Reverse proxy prefix |
| `VMR_DOWNLOAD_THREADS` | Number of download threads |
| `VMR_ALLOW_NESTED_SESSIONS` | Allow nested sessions |
| `VMR_PIXI_CHANNEL` | pixi package channel |
| `VMR_GITHUB_TOKEN` | GitHub API token (falls back to `GITHUB_TOKEN`) |
| `VM_DISABLE` | Session-mode guard (set by vmr) |

`HTTP_PROXY` / `HTTPS_PROXY` / `NO_PROXY` are handled directly by reqwest.

GitHub API requests use `GithubToken` from `conf.toml`, else `VMR_GITHUB_TOKEN`, else
`GITHUB_TOKEN`; with none of them set the requests are anonymous (60 requests/hour), which is
only enough for a handful of plugins.

## Plugin system (live crawling)

Plugin directory: `~/.vmr/plugins/*.lua`. Plugins are no longer compiled into the binary: when the
directory is empty they are downloaded from `gvcgo/vmr_plugins` automatically, or refresh them
manually with `vmr plugins -u`; pre-generated `.version.json` files from `gvcgo/vsources` are not
used. The full plugin format and binding reference is in
[docs/plugin-api.md](docs/plugin-api.md) ([简体中文](docs/plugin-api.zh-CN.md)).

Each plugin makes live upstream requests in `crawl()` and returns a version table; vmr caches the
result for the current platform in `~/.vmr/cache/<plugin>/<plugin>.versions.json` and uses it
directly when installing.

Plugin contract:

```lua
local C = dofile(__vmr_plugin_dir .. "/_vmr_common.lua")

plugin_name = "ripgrep"
sdk_name = "ripgrep"
plugin_version = "0.1"
homepage = "https://github.com/BurntSushi/ripgrep"
ic = C.installer({ flag_files = { linux = { "rg" } }, flag_dir_excepted = true })

function crawl()
    return C.github({ repo = "BurntSushi/ripgrep", os = { ... }, arch = { ... } })
end
```

- `C.github`: GitHub Release crawling (tag/file filtering, OS/arch mapping, installer)
- `C.pixi`: conda-forge/pixi package versions (installs are handled by the embedded pixi/rattler)
- `C.installer`: build the `ic` install config
- Complex sources can use the `vmrGetResponse`, `vmrInitSelection`, `vmrInitGJson`, etc. bindings
  directly

`vmr plugins -u` overwrites files of the same name using `gvcgo/vmr_plugins` as the source of
truth; vmr does not bundle plugins.

Key bindings include:

- HTML: `vmrGetResponse/vmrInitSelection/vmrFind/vmrEq/vmrAttr/vmrText/vmrEach`
- JSON: `vmrInitGJson/vmrGetString/vmrGetInt/vmrGetByKey/vmrMapEach/vmrSliceEach`
- GitHub: `vmrGetGithubRelease`
- Installer config: `ic = vmrNewInstallerConfig()` and related bindings
- Utilities: `vmrGetOsArch/vmrTrimSpace/vmrSprintf/vmrUnarchive/...`

The plugins currently shipped in `vmr_plugins` include go, lua, coursier, rustup, and miniconda.

## Code structure

```
crates/
├── vmr-core          # Path conventions, conf.toml, mirror/proxy policy
├── vmr-net           # Requests, chunked downloads, GitHub API
├── vmr-pixi          # pixi/rattler solving and installation, repodata cache
├── vmr-lua           # Lua runtime, plugin lifecycle, vmr* bindings
├── vmr-installer     # unarchiver / executable / pixi / coursier installers
├── vmr-env           # PATH/additional environment collection and injection
├── vmr-shell         # bash/zsh/fish environment files and cd hook
├── vmr-pty           # Session-mode child shell
├── vmr-utils         # Version sorting, extraction, copying, directory lookup
└── vmr-cli           # clap CLI
```

## Verification

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Manual smoke test (isolated HOME to avoid polluting your machine):

```bash
export HOME=/tmp/vmr-smoke
./target/release/vmr plugins
./target/release/vmr search go
./target/release/vmr use go@1.22.0
./target/release/vmr local go
./target/release/vmr uninstall go@1.22.0
```
