# Core Concepts

## SDKs, plugins, and versions

A plugin supplies version and installation information for an SDK. Run `vmr plugins` to see available SDKs and `vmr search go` to discover versions for your platform. A specification such as `go@1.22.0` names both the SDK and version. See [plugins](/plugins/) for more.

Installed versions coexist under vmr's installation directory. A global `vmr use go@1.22.0` installs the version if needed and switches the active link; it does not replace system directories. `vmr local` lists installed versions.

## Global, session, and project scope

A normal `vmr use` changes the global selection. For a temporary shell, use `vmr use -s python@3.12.2`; exit the session to restore the previous environment. Nested sessions are disabled by default and can be toggled with `vmr nested`.

For a project, run `vmr use -l node@20.11.0` in its root. This writes `.vmr.lock` and enters a session. With shell integration, entering the directory later activates its pins and leaving restores the previous environment. Missing versions prompt for confirmation before installation. Commit the lock file to share the pins with your team.

The directory hook caches the resolved project in `VMR_CD_CACHE`, avoiding repeated lookups as you move within the project. Shell integration is needed for automatic switching and immediate updates in the current shell; see [Quick Start](/starts/quickstart).

## Storage and environment

By default, `~/.vmr` holds configuration, plugins, caches, versions, and generated shell environment files. `VMR_SDK_INSTALLATION_DIR` changes the versions and cache root. On Windows, vmr uses shims in `%USERPROFILE%\.vmr\bin` instead of adding every SDK directory to `PATH`; other global variables are stored under `HKCU\Environment`.

## conda-forge packages

The built-in pixi backend can install packages beyond the plugin bundle: use `vmr search -p lua` and `vmr use -p lua@5.4.6`. No separate conda installation is required. Executables are exposed on `PATH` (or through Windows shims), but package `etc/conda/activate.d` variables are not imported.

## Manifests and migration

`vmr freeze` exports installed versions as a plain-text manifest. `vmr use -f FILE` installs its entries on another machine. Each SDK's active version is listed last so replay selects it. Migration from asdf, vfox, or system installations generates such a manifest without installing anything or converting project version files. See the [tutorial](/guides/tutorial).
