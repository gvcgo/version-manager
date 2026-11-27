# FAQ

## Why is `vmr` not found?

The executable must be on `PATH`. Check `which vmr`, or `where vmr` on Windows, and revisit the installation instructions in [Quick Start](/starts/quickstart).

## Why does my current terminal still use the old version?

Add the correct `vmr init` line to your shell configuration and reopen the terminal. Calling `command vmr` bypasses the shell wrapper, so use `vmr` directly. Run `vmr doctor` to diagnose integration, environment, `PATH`, and version links.

## Why did a project not switch automatically?

The directory hook requires shell integration and a project `.vmr.lock`. Enter the project directory after enabling integration. If a pinned version is missing, vmr asks before installing it; declining skips the installation. See [project pinning](/guides/tutorial).

## Why does a nested session fail?

Nested sessions are disabled by default. Exit the current session or use `vmr nested` to toggle permission before opening another one.

## How do I refresh a version list?

Run `vmr search go -n` to ignore the cached list. Version lookup uses live plugin sources; availability varies by platform.

## What if GitHub rate-limits requests?

Anonymous GitHub API requests are limited to 60 per hour. Set `VMR_GITHUB_TOKEN` or configure `GithubToken` in `conf.toml`; `GITHUB_TOKEN` is also used as a fallback.

## What if downloads are slow or fail?

Set `HTTP_PROXY` or `HTTPS_PROXY`, or use `VMR_LOCAL_PROXY` for vmr's local proxy. For conda-forge packages, set `VMR_PIXI_CHANNEL` to a suitable mirror. Configuration details are in the [tutorial](/guides/tutorial).

## How do I upgrade from vmr 0.7.x?

On startup, vmr offers to remove the old self-installed binary and updates legacy shell environment files while preserving SDK lines. Remove old vmr-related shell configuration and add the appropriate `vmr init` line. In PowerShell, also remove non-SDK variables and functions the old version wrote to `$PROFILE`. Reopen the terminal.

## How do I upgrade or remove vmr?

Upgrade using your installation method; replacing the executable does not remove versions or configuration in `~/.vmr`. To remove vmr, first uninstall unwanted SDKs with `vmr uninstall <sdk>@all`, then remove `~/.vmr` and the shell initialization line. Remove the executable using your package manager or delete the downloaded binary. On Windows, also remove the executable directory and `%USERPROFILE%\.vmr\bin` from `PATH`, and delete `%USERPROFILE%\.vmr`. If you configured a separate installation root, clean it up separately.
