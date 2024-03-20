<p style="" align="center">
  <!-- <img src="https://github.com/moqsien/img_repo/raw/main/vm_header_photo_2.png" alt="Logo" width="720" height="240"> -->
  <img src="https://github.com/moqsien/img_repo/raw/main/vm_profile_image.png" alt="Logo" width="240" height="240">
</p>

[中文](https://github.com/gvcgo/version-manager/blob/main/docs/readmeCN.md) | [En](https://github.com/gvcgo/version-manager)

* [version-manager(vm)](#1)
* [What's supported?](#2)
* [Installation/Update](#3)
* [How to set a proxy?](#4)
* [Subcommands](#5)
* [Related dirs](#6)
* [For Windows](#7)

------
<p id="1"></p>  

### version-manager(vm)

**vm** is a simple, cross-platform, and well-tested version manager for programming **languages** and **tools**. It is totally created for general purpose. You don't need any plugins, but just vm. Then everything can be managed.

Maybe you've already heard of **sdkman**, **gvm**, **pyenv**, **phpenv**, etc. However, never of these can manage multiple programming languages. **gvc** does the job, but it adopts a lot of other features. Most importantly, **gvc** provides free VPNs/Proxies, which makes its promotion become impossible in China. So, **vfox** is born. Indeed, **vfox** implies some promising features by introducing lua runtime as plugins just like neovim. Actually, **vfox** is not as perfect as what it promises. Therefore, **vm** comes.

------

<p id="2"></p>

### What's supported?

- **programming languages**
  - **java**(jdk, maven, gradle)
  - **kotlin**
  - **scala**(coursier, scala)
  - **go**
  - **python**(miniconda, python)
  - **php**(php8.0+ only)
  - **javascript/typescript**(node, bun, deno)
  - **dart**(flutter, dart)
  - **julia**
  - **.net**(dotnet-sdk, c#)
  - **c/c++**(cygwin-installer, msys2-installer)
  - **rust**(rustup-init, rust)
  - **vlang**(v, v-analyzer)
  - **zig**(zig, zls)
  - **typst**(typst, typst-lsp, typst-preview)
  - **gleam**
- **tools**
  - **commandline-tools**(for android, latest version only)
  - **git**(for windows only)
  - **lazygit**(depends on git)
  - **protoc**(protobuf)
  - **gsudo**(for windows only)
  - **vscode**(latest version only)
  - **neovim**
  - **agg**
  - **fd**
  - **fzf**
  - **ripgrep**
  - **tree-sitter**
  - **vhs**
  - **glow**

------

<p id="3"></p>  

### Installation/Update
- for **MacOS/Linux**(run the command below in terminal)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://gvc.1710717.xyz/proxy/https://raw.githubusercontent.com/gvcgo/version-manager/main/scripts/install.sh | sh
```

- for **Windows**(run the command below in powershell)
```powershell
powershell -nop -c "iex(New-Object Net.WebClient).DownloadString('https://gvc.1710717.xyz/proxy/https://raw.githubusercontent.com/gvcgo/version-manager/main/scripts/install.ps1')"
```

------

<p id="4"></p> 

### How to set a proxy?

**Choose either proxy or reverse-proxy.**

- **proxy**
```bash
vm set-proxy <http://localhost:port or socks5://localhost:port>
```

- **reverse-proxy**

```bash
# reverse proxy <https://gvc.1710717.xyz/proxy/> is available for free.
vm set-reverse-proxy https://gvc.1710717.xyz/proxy/
```

- **enable downloading from mirror sites in China**.
```bash
vm use -mirror-in-china go@1.22.1
```

------

<p id="5"></p> 

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
| **install-self** | - | Installs vm. |
| **version** | - | Shows version info of vm. |
------

**demo for MacOS**

<!-- <a href="https://asciinema.org/a/647462" target="_blank"><img src="https://asciinema.org/a/647462.svg" /></a> -->
![demo](https://github.com/moqsien/img_repo/raw/main/vm.gif)

**demo for Windows**

![demo](https://github.com/moqsien/img_repo/raw/main/vm_win.gif)

**demo for linux**

![demo](https://github.com/moqsien/img_repo/raw/main/vm_linux.gif)

------

<p id="6"></p> 

### Related dirs

- **vm installation dir**
```bash
$HOME/.vm/
```

- **application installation dir**

Specified during installation of **vm**.
```bash
~ % ./vm install-self
Enter App Installation Dir["$Home/.vm/" by default]:
/Users/moqsien/.vm
```

------

<p id="7"></p> 

### For Windows

**Note**: If you are using vm on Windows11, you need to enable the **Developer Mode** as vm requires to create symbolic links. If you're on Windows10, and any creating-symbolic-links-failure occurs, you can try vm with **Admin Privilege**. To get **envs** take effect for windows, you may need to close the current powershell terminal and open a new one.
