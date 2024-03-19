<p style="" align="center">
  <!-- <img src="https://github.com/moqsien/img_repo/raw/main/vm_header_photo_2.png" alt="Logo" width="720" height="240"> -->
  <img src="https://github.com/moqsien/img_repo/raw/main/vm_profile_image.png" alt="Logo" width="240" height="240">
</p>

[中文](https://github.com/gvcgo/version-manager/blob/main/docs/readmeCN.md) | [En](https://github.com/gvcgo/version-manager)

* [vm简介](#1)
* [vm都支持哪些编程语言和工具?](#2)
* [一键安装或更新](#3)
* [如何设置代理?](#4)
* [子命令介绍](#5)
* [相关目录说明](#6)
* [Win用户须知](#7)

------
<p id="1"></p>  

### vm简介

**vm** 是一个简单，跨平台，并且经过良好测试的版本管理工具。它完全是为了通用目的而创建的。你不需要任何插件，只需要 vm 就可以管理所有东西。

可能你已经听说过 **sdkman**, **gvm**, **pyenv**, **phpen** 等工具。然而，这些工具都不能管理多种编程语言/工具。最早出现的**gvc** 确实可以做到管理多种语言和工具，但它集成了很多其他特性，比较臃肿。最重要的是，**gvc** 提供了免费的 VPN或者说代理，这使得在国内推广**gvc**变得不太现实。因此，后来诞生了**vfox**。确实，**vfox**专注于编程语言版本管理，从它的主页来看，它暗示了一些非常有吸引力的功能，比如类似neovim的lua插件功能。但实际上，**vfox**并没有描述的那样完美。通过引入lua运行时，lua需要调用go代码中的爬虫相关功能才能实现各种版本的下载和管理，对于复杂页面来说，无疑更增复杂度。所以，**vfox**支持的编程语言和工具还是很有限。基于这些原因，在**gvc**的基础上，**vm**诞生了。

------

<p id="2"></p>

### vm都支持些什么语言和工具?

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

### 一键安装或更新
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

### 如何设置代理?

**代理或者反向代理任选其一进行设置，reverse-proxy由vm免费提供。对于github下载较慢或者失败的情况，你应该用得到。**

- **设置代理**
```bash
vm set-proxy <http://localhost:port or socks5://localhost:port>
```

- **设置免费的反向代理**

```bash
# reverse proxy <https://gvc.1710717.xyz/proxy/> is available for free.
vm set-reverse-proxy https://gvc.1710717.xyz/proxy/
```

- **使用国内镜像资源网站进行下载，对于部分由国内镜像的应用有效**.
```bash
vm use -mirror-in-china go@1.22.1
```

------

<p id="5"></p> 

### 子命令介绍

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

**demo**

<!-- <a href="https://asciinema.org/a/647462" target="_blank"><img src="https://asciinema.org/a/647462.svg" /></a> -->
![demo](https://github.com/moqsien/img_repo/raw/main/vm.gif)

------

<p id="6"></p> 

### 相关目录

- **vm安装目录**
```bash
$HOME/.vm/
```

- **通过vm安装的应用所在的目录**

该目录在**vm**安装过程中进行指定.例如，
```bash
~ % ./vm install-self
Enter App Installation Dir["$Home/.vm/" by default]:
/Users/moqsien/.vm
```

------

<p id="7"></p> 

### Windows用户需知

**注意**: 如果你正在使用Win11，那么你需要开启**开发者模式**，因为vm在创建链接符号时需要相关权限。如果你正在使用Win10，遇到创建链接符号失败的错误时，建议使用管理员权限打开powershell后再重试。在Win下，通过**vm**安装应用成功之后，如果在当前powershell窗口中找不到该命令，可以关闭当前powershell窗口，再打开一个新的，此时环境变量就生效了，就可以找到相关命令了，这是Win的特性，暂时修正不了。
