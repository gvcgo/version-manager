# Quick Start

Get vmr installed, connect it to your shell, and switch your first SDK. vmr supports Linux, macOS, and Windows on x86_64 and aarch64.

## Install vmr

Choose one installation method:

### Homebrew (macOS and Linux)

```sh
brew install gvcgo/tap/vmr
```

### Scoop (Windows)

```powershell
scoop bucket add gvcgo https://github.com/gvcgo/scoop-bucket
scoop install gvcgo/vmr
```

### APT (Debian and Ubuntu)

```sh
curl -fsSL https://gvcgo.github.io/vmr-apt/vmr.gpg | sudo gpg --dearmor -o /usr/share/keyrings/vmr.gpg
echo "deb [signed-by=/usr/share/keyrings/vmr.gpg] https://gvcgo.github.io/vmr-apt stable main" | sudo tee /etc/apt/sources.list.d/vmr.list
sudo apt update
sudo apt install vmr
```

### pacman (Arch Linux)

```sh
curl -fsSLO https://gvcgo.github.io/vmr-pacman/vmr.gpg
sudo pacman-key --add vmr.gpg
sudo pacman-key --lsign-key C387E9AE5C936248D446868C44E5A0323545E858
```

Append this repository to `/etc/pacman.conf`:

```ini
[vmr]
SigLevel = Required DatabaseOptional
Server = https://gvcgo.github.io/vmr-pacman/$arch
```

Then run `sudo pacman -Syu vmr`.

### MacPorts (macOS)

```sh
sudo mkdir -p /opt/local/var/macports/sources/github.com/gvcgo
sudo git clone https://github.com/gvcgo/vmr-macports /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports
echo "file:///opt/local/var/macports/sources/github.com/gvcgo/vmr-macports [nosync]" | sudo tee -a /opt/local/etc/macports/sources.conf
sudo portindex /opt/local/var/macports/sources/github.com/gvcgo/vmr-macports
sudo port install vmr
```

Replace `/opt/local` if your MacPorts prefix differs.

### Cargo (from source)

Install Rust 1.93 or later and a C toolchain first: Xcode Command Line Tools on macOS, MSVC Build Tools on Windows, or `build-essential` and `pkg-config` on Linux.

```sh
git clone https://github.com/gvcgo/version-manager
cd version-manager
cargo install --path vmr/crates/vmr-cli --locked
```

Put `~/.cargo/bin` on `PATH`. The crates are not published to crates.io; `cargo install vmr` is not available. To upgrade after pulling changes, repeat the install command with `--force`.

### Release archive

Download the archive for your OS and architecture from [GitHub Releases](https://github.com/gvcgo/version-manager/releases). Each package has a `.sha256` file, and the release also includes `SHA256SUMS`. For example, on Intel macOS:

```sh
shasum -a 256 -c vmr-x86_64-apple-darwin.tar.xz.sha256
tar -xJf vmr-x86_64-apple-darwin.tar.xz
mkdir -p ~/.local/bin
install -m 755 vmr-x86_64-apple-darwin/vmr ~/.local/bin/vmr
```

Ensure `~/.local/bin` is on `PATH`. On Windows, extract the zip and place `vmr.exe` in a directory on `PATH`.

Confirm the executable is available:

```sh
vmr version
```

## Connect your shell

The first interactive command can offer to add shell integration automatically. Alternatively, add the appropriate line to your shell configuration, then reopen the terminal:

```sh
# ~/.bashrc
eval "$(vmr init bash)"
# or ~/.zshrc
eval "$(vmr init zsh)"
```

```fish
# ~/.config/fish/config.fish
vmr init fish | source
```

```powershell
# PowerShell: add to $PROFILE
Invoke-Expression (& vmr init powershell)
```

Integration refreshes the current shell after a switch and enables project-directory hooks. Without it, commands still work, but switches require a new terminal to take effect.

## Install and switch

```sh
vmr plugins             # supported SDKs
vmr search go           # available versions
vmr use go@1.22.0       # install and switch
vmr local go            # installed versions; <current> is active
```

Version availability depends on the plugin and platform. Use `vmr search go -n` to refresh a cached list. Continue with [core concepts](/starts/concepts) or the [tutorial](/guides/tutorial).
