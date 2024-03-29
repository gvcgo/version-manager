<p style="" align="center">
  <!-- <img src="https://github.com/moqsien/img_repo/raw/main/vm_header_photo_2.png" alt="Logo" width="720" height="240"> -->
  <img src="https://github.com/moqsien/img_repo/raw/main/vm_profile_image.png" alt="Logo" width="240" height="240">
</p>

[![Go Report Card](https://img.shields.io/badge/go%20report-A+-brightgreen.svg?style=for-the-badge)](https://goreportcard.com/report/github.com/gvcgo/version-manager)
[![GitHub License](https://img.shields.io/github/license/gvcgo/version-manager?style=for-the-badge)](LICENSE)
[![GitHub Release](https://img.shields.io/github/v/release/gvcgo/version-manager?display_name=tag&style=for-the-badge)](https://github.com/gvcgo/version-manager/releases)
[![Discord](https://img.shields.io/discord/1191981003204477019?style=for-the-badge&logo=discord)](https://discord.gg/85c8ptYgb7)

[中文](https://github.com/gvcgo/version-manager/blob/main/docs/readmeCN.md) | [En](https://github.com/gvcgo/version-manager)

- [vm简介](#vm简介)
- [功能特点](#功能特点)
- [vm和vfox支持列表对比](#vm和vfox支持列表对比)
- [一键安装/更新vm](#一键安装更新vm)
- [如何设置代理?](#如何设置代理)
- [子命令介绍](#子命令介绍)
- [相关目录](#相关目录)
- [Windows用户须知](#windows用户须知)
- [贡献者](#贡献者)
- [说明](#说明)
- [Todo-List](#todo-list)

------
<p id="1"></p>  

### vm简介

**vm** 是一个简单，跨平台，并且经过良好测试的版本管理工具。它完全是为了通用目的而创建的。无需插件，开箱即用。

可能你已经听说过 **sdkman**, **gvm**, **nvm**, **pyenv**, **phpenv** 等工具。然而，这些工具都不能管理多种编程语言。最近，有一款专注于编程语言版本管理的管理器，叫做**vfox**，从它的主页来看，它暗示了一些非常有吸引力的功能，比如类似neovim的lua插件功能。然而，**vfox**并没有描述的那样完美。它引入lua运行时，然后lua脚本需要调用go代码中的爬虫相关功能才能实现各种版本的下载和管理，对于各种官网页面来说，复杂度不降反升。此外，直接在版本管理器中使用页面爬虫，存在着相当的不稳定因素，因为页面上的接口可能会随时改版，从而导致用户端体验不够好。所以，**vfox**支持的编程语言和工具有限，并且稳定性不够高。基于这些原因，**vm**诞生了。**vm**支持了国内程序员常用的几乎所有编程语言，并且支持了vlang、zig、typst等新兴的有一定潜力的语言，它隔离并缓存了爬虫部分，让用户体验更流畅和稳定。还支持反代和本地代理设置，多线程下载等，大大提高国内用户的下载体验。因此，不管你是老鸟还是菜鸟，**vm**都能给你带来相当的便利。你不用再手动去找任何资源，就能轻松安装管理各种sdk版本，尝试新的语言，新的特性。最后，**vm**将这些sdk或工具集中管理，对于有**洁癖**的人来说，也是福音。

------

<p id="2"></p>

### 功能特点

- 安装或卸载某个版本的sdk；
- 在不同版本的sdk之间切换；
- 一键管理环境变量；
- 对neovim和vscode用户友好，可以一键安装neovim和vscode。同时，neovim中一些明星插件的安装也可以一键完成，例如fd，ripgrep，tree-sitter等；
- 相比其他版本管理器来说要更稳定；
- 支持多线程下载，速度飞快🚀🚀🚀，可使用**vm use -h**命令查看用法；
- 无需任何插件，开箱即用；

------
<p id="3"></p> 

### vm和vfox支持列表对比

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
| **kubectl** | ✅︎ | ❌︎ |

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
| **env** | --remove=false/true | 手动设置环境变量，比编辑shell配置文件或者打开windows环境变量管理更方便 |
| **install-self** | - | 安装vm到$HOME/.vm，用户一般无需关心 |
| **version** | - | 显示vm的版本信息 |
| **completion** | - | 生成关于不同shell的自动补全(支持bash、zsh、fish、powershell) |

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

------
<p id="9"></p>  

### 贡献者
> 感谢以下贡献者对本项目的贡献。
<a href="https://github.com/gvcgo/version-manager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=gvcgo/version-manager" />
</a>

------
<p id="10"></p> 

### 说明
**vm**是一个跨平台的命令行工具。它不会像asdf-vm(基于shell脚本，仅支持unix-like系统)那样做到什么都试图包含，因为那样会大大增加复杂性，也降低跨了平台的可能。而且大多数情况下，我们常用的语言和工具基本已经包含在**vm**中了。**vm**不会试图去包含那些在某个平台下需要自行编译的sdk，因为每个开发者的开发环境都不一样，无法保证编译一定能通过。所以**vm**仅仅会使用pre-compiled binary来安装应用。如果你有什么需要进行版本管理的sdk或者工具推荐，请在[Issues](https://github.com/gvcgo/version-manager/issues)中提出，**vm**开发团队会评估后决定是否加入。

------
<p id="11"></p> 

### Todo-List
- [ ] 各种语言的包安装管理器国内加速资源一键配置
