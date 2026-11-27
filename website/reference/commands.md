# Command Reference

Run `vmr --help` or `vmr <command> --help` for built-in help. In the examples, `sdk@version` is a version specification; available SDKs and versions depend on the plugin and platform.

| Command | Aliases | Purpose |
| --- | --- | --- |
| `vmr version` | `v` | Show version information. |
| `vmr init [shell]` | | Print shell integration for bash, zsh, fish, or PowerShell. |
| `vmr template [kind] [-d DIR]` | | Print a Lua plugin template (`github`, `html`, `json`, or `pixi`), or write it to a directory. |
| `vmr plugins [-u]` | | List plugins and supported SDKs; `-u` / `--update-all` refreshes plugins first. |
| `vmr search [sdk] [-p] [-n]` | `s` | List available versions; `-p` / `--pixi` queries conda-forge, `-n` / `--no-cache` refreshes the list. |
| `vmr local [sdk] [-c]` | `l` | List installed versions, optionally only current versions with `-c` / `--current`. |
| `vmr use [sdk@version]` | `u`, `h` | Install and switch globally. |
| `vmr use -s sdk@version` | | Install and switch in a session (`--session`). |
| `vmr use -l sdk@version` | | Write `.vmr.lock` and enter a session (`--lock`). |
| `vmr use -p sdk@version` | | Install with the pixi conda-forge backend (`--pixi`). |
| `vmr use -f FILE` | | Install every entry in a manifest (`--file`); cannot be combined with a spec, `-s`, `-l`, or `-p`. |
| `vmr freeze [FILE]` | `fz` | Export installed versions; defaults to `vmr_<YYYYMMDD>.txt`. |
| `vmr migrate [MAP] --asdf` | | Export asdf versions to `migration_from_asdf.txt`. |
| `vmr migrate [MAP] --vfox` | | Export vfox versions to `migration_from_vfox.txt`. |
| `vmr migrate [MAP] --system` | | Probe system commands and export `migration_from_system.txt`. |
| `vmr uninstall sdk@version` | `uni`, `r` | Remove a version; use `sdk@all` to remove all versions of an SDK. |
| `vmr session` | `se` | Report whether the current shell is in a vmr session. |
| `vmr nested` | `ns` | Toggle permission for nested sessions. |
| `vmr cdhook` | `cd-hook` | Internal directory hook: apply project pins and offer to install missing versions. |
| `vmr doctor` | | Diagnose integration, environment, `PATH`, and version links. |
| `vmr mcp` | | Serve the Model Context Protocol over stdio. |

`--conda` is also accepted as an alias for `--pixi` on `search` and `use`. Migration only writes manifests; it does not install versions. An optional mapping file specifies one `<source> <vmr-sdk>` pair per line and replaces the default mappings. See the [tutorial](/guides/tutorial) for examples.

`vmr doctor` accepts `--json`, `--quiet`, `--strict` (exit 1 on warnings), `--no-emoji`, and `--shell` followed by `bash`, `zsh`, `fish`, or `powershell`. Learn about plugin authoring under [plugins](/plugins/).
