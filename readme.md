<p style="" align="center">
  <!-- <img src="https://github.com/moqsien/img_repo/raw/main/vm_header_photo_2.png" alt="Logo" width="720" height="240"> -->
  <img src="https://github.com/moqsien/img_repo/raw/main/vm_profile_image.png" alt="Logo" width="240" height="240">
</p>

[![Go Report Card](https://img.shields.io/badge/go%20report-A+-brightgreen.svg?style=for-the-badge)](https://goreportcard.com/report/github.com/gvcgo/version-manager)
[![GitHub License](https://img.shields.io/github/license/gvcgo/version-manager?style=for-the-badge)](LICENSE)
[![GitHub Release](https://img.shields.io/github/v/release/gvcgo/version-manager?display_name=tag&style=for-the-badge)](https://github.com/gvcgo/version-manager/releases)
[![PRs Card](https://img.shields.io/badge/PRs-vm-cyan.svg?style=for-the-badge)](https://github.com/gvcgo/version-manager/pulls)
[![Issues Card](https://img.shields.io/badge/Issues-vm-pink.svg?style=for-the-badge)](https://github.com/gvcgo/version-manager/issues)
[![Versions Repo Card](https://img.shields.io/badge/Versions-repo-blue.svg?style=for-the-badge)](https://github.com/gvcgo/resources)

[中文](https://github.com/gvcgo/version-manager/blob/main/docs/readmeCN.md) | [En](https://github.com/gvcgo/version-manager)

- [version-manager(vmr)](#version-managervmr)
- [features](#features)
- [vmr versus vfox](#vmr-versus-vfox)
- [Installation/Update](#installationupdate)
- [How to set a proxy?](#how-to-set-a-proxy)
- [Subcommands](#subcommands)
- [Related dirs](#related-dirs)
- [Contributors](#contributors)

------
<p id="1"></p>  

### version-manager(vmr)

**vmr** is a simple, cross-platform, and well-tested version manager for programming **languages** and **tools**. It is totally created for general purpose. You don't need any plugins, but just vm. Then everything can be managed.

Maybe you've already heard of **fnm**, **sdkman**, **gvm**, **nvm**, **pyenv**, **phpenv**, etc. However, none of them can manage multiple programming languages. Managers like **asdf-vm** support multiple languages, but only works on unix-like systems, and makes things look complicated. Therefore, **vmr** comes.

[youtube video demo](https://www.youtube.com/watch?v=CFIxPfBn8QY&t=626s)

[Docs](https://gvcgo.github.io/vmr/)

------

<p id="2"></p>

### features

- Cross-platform, supports Windows, Linux, MacOS.
- Supports multiple languages and tools.
- Nicer TUI, reduces user input, while maintaining the flexibility.
- Supports locking SDK version for each project.
- Supports reverse proxy settings and multi-threaded downloads, improve your download experience.
- Version crawler and main project are separated to ensure faster response and higher stability.
- No need for plugins, just out of the box.
- Installs SDKs in local disk instead of docker containers.
- Easy to use, you only need to focus on about 6 subcommands of vmr.

------
<p id="3"></p> 

### vmr versus vfox

| sdk | vmr | vfox |
|-------|-------|-------|
| **java(jdk)** | ✅︎ | ✅︎ |
| **maven** | ✅︎ | ✅︎ |
| **gradle** | ✅︎ | ✅︎ |
| **kotlin** | ✅︎ | ✅︎ |
| **scala** | ✅︎ | ✅︎ |
| **groovy** | ✅︎ | ✅︎ |
| **python** | ✅︎ | ✅︎ |
| **pypy** | ✅︎ | ❌︎ |
| **miniconda** | ✅︎ | ❌︎ |
| **go** | ✅︎ | ✅︎ |
| **node** | ✅︎ | ✅︎ |
| **deno** | ✅︎ | ✅︎ |
| **bun** | ✅︎ | ❌︎ |
| **flutter(dart)** | ✅︎ | ✅︎ |
| **.net** | ✅︎ | ✅︎ |
| **zig** | ✅︎ | ✅︎ |
| **zls** | ✅︎ | ❌︎ |
| **php** | ✅︎ | ✅︎ |
| **rust** | ✅︎ | ❌︎ |
| **cmdline-tool(android)** | ✅︎ | ❌︎ |
| **android SDKs** | ✅︎ | ❌︎ |
| **vlang** | ✅︎ | ❌︎ |
| **v-analyzer** | ✅︎ | ❌︎ |
| **cygwin-installer** | ✅︎ | ❌︎ |
| **msys2-installer** | ✅︎ | ❌︎ |
| **julia** | ✅︎ | ❌︎ |
| **dlang** | ✅︎ | ❌︎ |
| **serve-d(lsp for dlang)** | ✅︎ | ❌︎ |
| **odin** | ✅︎ | ❌︎ |
| **typst** | ✅︎ | ❌︎ |
| **typst-lsp** | ✅︎ | ❌︎ |
| **typst-preview** | ✅︎ | ❌︎ |
| **gleam** | ✅︎ | ❌︎ |
| **git-for-windows** | ✅︎ | ❌︎ |
| **neovim** | ✅︎ | ❌︎ |
| **vscode** | ✅︎ | ❌︎ |
| **protobuf(protoc)** | ✅︎ | ❌︎ |
| **lazygit** | ✅︎ | ❌︎ |
| **kubectl** | ✅︎ | ❌︎ |
| **upx** | ✅︎ | ❌︎ |
| **acast(asciinema)** | ✅︎ | ❌︎ |
| **erlang(need compilation)** | ❌︎ | ✅︎ |
| **elixir(need compilation)** | ❌︎ | ✅︎ |

------

<p id="4"></p>  

### Installation/Update
- for **MacOS/Linux**(run the command below in terminal)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://gvc.1710717.xyz/proxy/https://raw.githubusercontent.com/gvcgo/version-manager/main/scripts/install.sh | sh
```

- for **Windows**(run the command below in powershell) (See tips in **For Windows**)
```powershell
powershell -nop -c "iex(New-Object Net.WebClient).DownloadString('https://gvc.1710717.xyz/proxy/https://raw.githubusercontent.com/gvcgo/version-manager/main/scripts/install.ps1')"
```

------

<p id="5"></p> 

### How to set a proxy?

- **reverse-proxy**

```bash
# reverse proxy <https://gvc.1710717.xyz/proxy/> is available for free.
vmr set-reverse-proxy https://gvc.1710717.xyz/proxy/
```

------

<p id="6"></p> 

### Subcommands

| subcommand | args | desc |
|-------|-------|-------|
| **list** | - | Shows what's supported. |
| **search** | sdk-name | Shows available versions for a sdk. |
| **use** | sdk-name@version | Installs/Swithes to the specific version of a sdk. |
| **local** | sdk-name | Shows installed versions of a sdk. |
| **uninstall** | sdk-name@version or sdk-name@all | Uninstalls versions for a sdk. |
| **clear-cache** | sdk-name | Clears the cached files for a sdk. |
| **set-reverse-proxy** | https://gvc.1710717.xyz/proxy/ | Sets a reverse-proxy for vm. |
| **set-proxy** | http or socks5( scheme://host:port ) | Sets a local proxy for vm. |
| **env** | --remove=false/true | Sets/Removes env manually. |
| **install-self** | - | Installs vm. |
| **version** | - | Shows version info of vm. |
| **completion** | - | Generate the autocompletion script for  for the specified shell.(bash, zsh, fish, or powershell) |

------

**demo for MacOS**

<!-- <a href="https://asciinema.org/a/647462" target="_blank"><img src="https://asciinema.org/a/647462.svg" /></a> -->
![demo](https://github.com/moqsien/img_repo/raw/main/vm.gif)

**demo for Windows**

![demo](https://github.com/moqsien/img_repo/raw/main/vm_win.gif)

**demo for linux**

![demo](https://github.com/moqsien/img_repo/raw/main/vm_linux.gif)

------

<p id="7"></p> 

### Related dirs

- **vmr installation dir**
```bash
$HOME/.vm/
```

- **application installation dir**

Specified during installation of **vmr**. Use "$HOME/.vm" by default.
![installation](https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr_install.png)

------
<p id="9"></p>  

### Contributors
> Thanks to the following people who have contributed to this project.
<a href="https://github.com/gvcgo/version-manager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=gvcgo/version-manager" />
</a>

