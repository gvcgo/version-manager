# Tutorial

These examples assume [shell integration](/starts/quickstart) is enabled. Pick versions shown by `vmr search` for your platform.

## Switch an SDK

```sh
vmr plugins
vmr search node
vmr use node@20.11.0
vmr local node
```

`vmr use` installs a missing version and makes it active globally. To remove a version, run `vmr uninstall node@20.11.0`; `vmr uninstall node@all` removes all installed Node versions.

## Pin a project

```sh
cd my-project
vmr use -l node@20.11.0
```

The command writes `.vmr.lock` and enters a session. Commit the file so teammates can share the pin. When you later enter the project, the shell hook activates its versions, asking before installing any that are missing; leaving restores the previous environment.

## Try a temporary version

```sh
vmr use -s python@3.12.2
python --version
exit
```

The session ends at `exit`, restoring the prior environment. If you need a session inside another session, toggle the setting first with `vmr nested`.

## Install from conda-forge

```sh
vmr search -p lua
vmr use -p lua@5.4.6
```

Use `VMR_PIXI_CHANNEL` to select a mirror. Package activation scripts' environment variables are not imported.

## Move to another machine

```sh
vmr freeze my-versions.txt
# Copy my-versions.txt to the new machine, then:
vmr use -f my-versions.txt
```

The manifest contains one `sdk@version` per line; `#` begins a comment. The active version is exported last for each SDK, so replay selects it. Without a filename, `freeze` writes `vmr_<YYYYMMDD>.txt`.

## Migrate an existing setup

```sh
vmr migrate --asdf
vmr migrate --vfox
vmr migrate --system
# Review the generated file, then install its entries:
vmr use -f migration_from_asdf.txt
```

Migration reads existing state and writes `migration_from_asdf.txt`, `migration_from_vfox.txt`, or `migration_from_system.txt`; it does not install anything or modify the other manager. It skips unsupported names and incomplete installations, and does not convert project-level version files. System probing skips executables owned by common version managers.

To override name mapping, supply a positional map file:

```text
nodejs node
golang go
unwanted -
```

```sh
vmr migrate my-map.txt --asdf
```

The map is authoritative: unlisted names are skipped and reported; a bare source name or `-` target skips that source.

## Configure downloads

vmr reads `~/.vmr/conf.toml`, falling back to `~/.config/vmr/conf.toml` only if the first file does not exist. It never merges them and writes to the file it read. If neither exists, it creates the first. Keys are PascalCase:

```toml
ProxyUri = "http://127.0.0.1:7890"
ThreadNum = 4
```

Other keys include `ReverseProxy`, `SDKIntallationDir` (spelling intentional), `VersionHostUrl`, `AllowNestedSessions`, `GithubToken`, `CacheRetentionTime`, and `DisableCache`. Environment variables take precedence: `VMR_SDK_INSTALLATION_DIR`, `VMR_HOST`, `VMR_LOCAL_PROXY`, `VMR_REVERSE_PROXY`, `VMR_DOWNLOAD_THREADS`, `VMR_ALLOW_NESTED_SESSIONS`, and `VMR_GITHUB_TOKEN`. The GitHub token also falls back to `GITHUB_TOKEN`; standard `HTTP_PROXY`, `HTTPS_PROXY`, and `NO_PROXY` are honored.

See the [command reference](/reference/commands) and [FAQ](/guides/faq) for more.
