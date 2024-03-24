<p style="" align="center">
  <!-- <img src="https://github.com/moqsien/img_repo/raw/main/vm_header_photo_2.png" alt="Logo" width="720" height="240"> -->
  <img src="https://github.com/moqsien/img_repo/raw/main/vm_profile_image.png" alt="Logo" width="240" height="240">
</p>

[![Go Report Card](https://img.shields.io/badge/go%20report-A+-brightgreen.svg?style=for-the-badge)](https://goreportcard.com/report/github.com/gvcgo/version-manager)
[![GitHub License](https://img.shields.io/github/license/gvcgo/version-manager?style=for-the-badge)](LICENSE)
[![GitHub Release](https://img.shields.io/github/v/release/gvcgo/version-manager?display_name=tag&style=for-the-badge)](https://github.com/gvcgo/version-manager/releases)
[![Discord](https://img.shields.io/discord/1191981003204477019?style=for-the-badge&logo=discord)](https://discord.gg/85c8ptYgb7)

[中文](https://github.com/gvcgo/version-manager/blob/main/docs/readmeCN.md) | [En](https://github.com/gvcgo/version-manager)

* [vm简介](#1)
* [功能介绍](#2)
* [vm和vfox对比](#2)
* [一键安装或更新](#3)
* [如何设置代理?](#4)
* [子命令介绍](#5)
* [相关目录说明](#6)
* [Win用户须知](#7)

------
<p id="1"></p>  

### vm简介

**vm** 是一个简单，跨平台，并且经过良好测试的版本管理工具。它完全是为了通用目的而创建的。你不需要任何插件，只需要 vm 就可以管理所有东西。

可能你已经听说过 **sdkman**, **gvm**, **nvm**, **pyenv**, **phpenv** 等工具。然而，这些工具都不能管理多种编程语言。最早出现的**gvc** 确实可以做到管理多种语言和工具，但它集成了很多其他特性，比较臃肿。最重要的是，**gvc** 提供了免费的 VPN或者说代理，这使得在国内推广**gvc**变得不太现实。因此，后来诞生了**vfox**。确实，**vfox**专注于编程语言版本管理，从它的主页来看，它暗示了一些非常有吸引力的功能，比如类似neovim的lua插件功能。但实际上，**vfox**并没有描述的那样完美。通过引入lua运行时，lua需要调用go代码中的爬虫相关功能才能实现各种版本的下载和管理，对于复杂页面来说，并没有降低复杂度，反而使得复杂度提高。所以，**vfox**支持的编程语言和工具还是很有限。基于这些原因，在**gvc**的基础上，**vm**诞生了。**vm**支持了国内程序员常用的几乎所有编程语言，并且支持了vlang、zig、typst等新兴的有一定潜力的语言。不管你是老鸟还是菜鸟，它都能给你带来一定的便利。你不用手动去找任何资源，就能轻松安装管理各种版本，尝试新的语言，新的特性。**vm**将这些sdk或工具集中管理，对于有**洁癖**的人来说，也是福音。

------

<p id="2"></p>

### 功能介绍

- 安装和卸载某个版本的sdk
- 在不同版本的sdk之间切换
- 管理环境变量
- 对neovim和vscode用户友好，可以一键安装neovim和vscode。同时，neovim中一些明星插件的安装也可以一键完成，例如fd，ripgrep，tree-sitter等。

------
<p id="3"></p> 

### vm和vfox对比

| sdk | vm | vfox |
|-------|-------|-------|
| **java(jdk)** | ✅︎ | ✅︎ |
| **maven** | ✅︎ | ✅︎ |
| **gradle** | ✅︎ | ✅︎ |
| **kotlin** | ✅︎ | ✅︎ |
| **scala** | ✅︎ | ❌︎ |
| **python** | ✅︎ | ✅︎ |
| **miniconda** | ✅︎ | ❌︎ |
| **go** | ✅︎ | ✅︎ |
| **node** | ✅︎ | ✅︎ |
| **deno** | ✅︎ | ✅︎ |
| **bun** | ✅︎ | ❌︎ |
| **flutter(dart)** | ✅︎ | ✅︎ |
| **.net** | ✅︎ | ✅︎ |
| **zig** | ✅︎ | ✅︎ |
| **php** | ✅︎ | ❌︎ |
| **rust** | ✅︎ | ❌︎ |
| **cmdline-tool(android)** | ✅︎ | ❌︎ |
| **vlang** | ✅︎ | ❌︎ |
| **cygwin** | ✅︎ | ❌︎ |
| **msys2** | ✅︎ | ❌︎ |
| **julia** | ✅︎ | ❌︎ |
| **typst** | ✅︎ | ❌︎ |
| **gleam** | ✅︎ | ❌︎ |
| **git-for-windows** | ✅︎ | ❌︎ |
| **neovim** | ✅︎ | ❌︎ |
| **vscode** | ✅︎ | ❌︎ |
| **protobuf(protoc)** | ✅︎ | ❌︎ |
| **lazygit** | ✅︎ | ❌︎ |

------

<p id="4"></p>  

### 一键安装/更新vm
- for **MacOS/Linux**(复制下面的命令到terminal执行即可)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://gvc.1710717.xyz/proxy/https://raw.githubusercontent.com/gvcgo/version-manager/main/scripts/install.sh | sh
```

- for **Windows**(复制下面的命令到powershell中执行即可)
```powershell
powershell -nop -c "iex(New-Object Net.WebClient).DownloadString('https://gvc.1710717.xyz/proxy/https://raw.githubusercontent.com/gvcgo/version-manager/main/scripts/install.ps1')"
```

------

<p id="5"></p> 

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

- **使用国内镜像资源网站进行下载，对于部分有国内镜像的应用有效**.
```bash
vm use -mirror-in-china go@1.22.1
```

------

<p id="6"></p> 

### 子命令介绍

| 子命令 | 参数 | 功能 |
|-------|-------|-------|
| **list** | - | 显示支持的sdk列表(列表操作：j/k翻动列表，q退出) |
| **search** | sdk-name | 显示该sdk支持的版本列表 |
| **use** | sdk-name@version | 安装/切换sdk到指定版本 |
| **local** | sdk-name | 显示sdk在本地已安装的版本 |
| **uninstall** | sdk-name@version or sdk-name@all | 卸载某个版本或者卸载所有版本 |
| **clear-cache** | sdk-name | 清除本地已缓存的压缩文件 |
| **set-reverse-proxy** | https://gvc.1710717.xyz/proxy/ | 设置反向代理，用于github下载加速 |
| **set-proxy** | http or socks5( scheme://host:port ) | 设置本地代理，可用于任何网站的下载加速 |
| **install-self** | - | 安装vm到$HOME/.vm，用户一般无需关心 |
| **version** | - | 显示vm的版本信息 |
------

**MacOS演示**

<!-- <a href="https://asciinema.org/a/647462" target="_blank"><img src="https://asciinema.org/a/647462.svg" /></a> -->
![demo](https://github.com/moqsien/img_repo/raw/main/vm.gif)

**Windows演示**

![demo](https://github.com/moqsien/img_repo/raw/main/vm_win.gif)

**Linux演示**

![demo](https://github.com/moqsien/img_repo/raw/main/vm_linux.gif)

------

<p id="7"></p> 

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

<p id="8"></p> 

### Windows用户须知

**注意**: 如果你正在使用Win11，那么你需要开启**开发者模式**，因为vm在创建链接符号时需要相关权限。如果你正在使用Win10，遇到创建链接符号失败的错误时，建议使用管理员权限打开powershell后再重试。在Win下，通过**vm**安装应用成功之后，如果在当前powershell窗口中找不到该命令，可以关闭当前powershell窗口，再打开一个新的，此时环境变量就生效了，就可以找到相关命令了，这是Win的特性，暂时修正不了。
