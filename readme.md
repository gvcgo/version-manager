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
- [For Windows](#for-windows)
- [Contributors](#contributors)
- [Supplementary](#supplementary)
- [Todo-List](#todo-list)

------
<p id="1"></p>  

### version-manager(vmr)

**vmr** is a simple, cross-platform, and well-tested version manager for programming **languages** and **tools**. It is totally created for general purpose. You don't need any plugins, but just vm. Then everything can be managed.

Maybe you've already heard of **fnm**, **sdkman**, **gvm**, **nvm**, **pyenv**, **phpenv**, etc. However, none of them can manage multiple programming languages. Managers like **asdf-vm** support multiple languages, but only works on unix-like systems, and makes things look complicated. Therefore, **vmr** comes.

------

<p id="2"></p>

### features

- Installs or uninstalls versions of sdk.
- Swithes between versions of sdk.
- Using a version only in current terminal session is supported. See with command **vmr use -h**.
- Lock sdk version for a project with command like **vmr use -l go@1.22.2**, and autoswithes sdk to the locked version while using command **cdr <path-to-your-project>**.
- Handles envs.
- Friendly to VSCoders or Neovimers.
- Downloads files blazingly fast🚀🚀🚀 with multi-threads. See with command **vmr use -h**.
- Auto-completions for shells. See with command **vmr completion -h**.
- Generates command **"vmr use sdk-name@version"** automatically using selected item from version list, and add the command to clipboard for later usage.
- Android development with Flutter and VSCode. No Android Studio is needed.
- No plugins needed.
- More stable.

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

- **Manual** installation
```text
1. Download zip file from release.
2. Unzip it, run command "your/full/path/to/vmr install-self".
```

- **If you're a gopher, you can also complie the project by yourself. The main func is in ./cmd/vmr**

- **update**

```bash
vmr-update
```

**Note**: If you are installing **vmr** for the first time, please use **source .zshrc** or **source .bashrc** to refresh your envs. For windows users, just open a new powershell terminal, then **vmr** will be available.

------

<p id="5"></p> 

### How to set a proxy?

**Choose either proxy or reverse-proxy.**

- **proxy**
```bash
vmr set-proxy <http://localhost:port or socks5://localhost:port>
```

- **reverse-proxy**

```bash
# reverse proxy <https://gvc.1710717.xyz/proxy/> is available for free.
vmr set-reverse-proxy https://gvc.1710717.xyz/proxy/
```

- **enable downloading from mirror sites in China**.
```bash
vmr use -mirror-in-china go@1.22.1
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

<p id="8"></p> 

### For Windows

**Note**: If you are using **vmr** on Windows11, you need to enable the **Developer Mode** as **vmr** requires to create symbolic links. If you're on Windows10, and any creating-symbolic-links-failure occurrs, you can try **vmr** with **Admin Privilege**. To get **envs** take effect for windows, you may need to close the current powershell terminal and open a new one. Note that extFAT and FAT32 are not supported. 

**sudo command on Windows**: **gsudo**. You can use **vmr search gsudo** to see what's available.

🛎️🚨 **Virus Positive?**: It's definitely a false positive. See [here](https://forum.golangbridge.org/t/my-compiled-exe-file-is-declared-as-a-virus/34038). If this occurrs, you can install **vmr** manually, and add **vmr** to be trusted.

- **How can I add vmr to windows 'Virus & threat protection Exclusions?'**
```text
Windows Security>> Virus & threat protection>> Virus & threat protection settings>> Exclusions
```

------
<p id="9"></p>  

### Contributors
> Thanks to the following people who have contributed to this project.
<a href="https://github.com/gvcgo/version-manager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=gvcgo/version-manager" />
</a>

------
<p id="10"></p> 

### Supplementary
**vmr** is created to be a cross-platform command line tool. **We will not try to include everything just like asdf-vm or its imitator vfox did**, as that will greatly increase the complexity and also reduce the possibility of cross-platform. And most of the time, frequently used SDKs and tools have already been covered by **vmr**. **vmr** will not try to include SDKs that need to be compiled under a certain platform. Because each developer's development environment is different, it is impossible to ensure the completion of a compilation. So **vmr** will only use pre-built binaries for installations. If you have any SDKs or tools to recommand for version management, please raise an issue in [Issues](https://github.com/gvcgo/version-manager/issues).

So, **vmr** is going to keep as **lightweight, stable, and user-friendly** as possible.

------
<p id="11"></p> 

### Todo-List
- [ ] To manage package repo mirror sites in China.
