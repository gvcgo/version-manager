<p style="" align="center">
  <!-- <img src="https://github.com/moqsien/img_repo/raw/main/vm_header_photo_2.png" alt="logo" width="720" height="240"> -->
  <img src="https://github.com/moqsien/img_repo/raw/main/vmr_logo.png" alt="logo" width="240" height="240">
</p>

[![go report card](https://img.shields.io/badge/go%20report-a+-brightgreen.svg?style=for-the-badge)](https://goreportcard.com/report/github.com/gvcgo/version-manager)
[![github license](https://img.shields.io/github/license/gvcgo/version-manager?style=for-the-badge)](license)
[![github release](https://img.shields.io/github/v/release/gvcgo/version-manager?display_name=tag&style=for-the-badge)](https://github.com/gvcgo/version-manager/releases)
[![prs card](https://img.shields.io/badge/prs-vm-cyan.svg?style=for-the-badge)](https://github.com/gvcgo/version-manager/pulls)
[![issues card](https://img.shields.io/badge/issues-vm-pink.svg?style=for-the-badge)](https://github.com/gvcgo/version-manager/issues)
[![versions repo card](https://img.shields.io/badge/versions-repo-blue.svg?style=for-the-badge)](https://github.com/gvcgo/resources)

[中文](https://github.com/gvcgo/version-manager/blob/main/docs/readmecn.md) | [en](https://github.com/gvcgo/version-manager)

- [vmr简介](#vmr简介)
- [功能特点](#功能特点)
- [vmr的数据流](#vmr的数据流)
- [一键安装/更新vm](#一键安装更新vm)
- [如何设置代理?](#如何设置代理)
- [子命令介绍](#子命令介绍)
- [贡献者](#贡献者)

------
<p id="1"></p>  

### vmr简介

**vmr** 是一个简单，跨平台，并且经过良好测试的版本管理工具。它完全是为了通用目的而创建的。无需插件，开箱即用。

可能你已经听说过**fnm**, **sdkman**, **gvm**, **nvm**, **pyenv**, **phpenv** 等工具。然而，这些工具都不能管理多种编程语言，甚至有些看起来会比较复杂。而**vmr**支持了国内程序员常用的几乎所有编程语言，并且支持了vlang、zig、typst等新兴的有一定潜力的语言，它隔离并缓存了爬虫部分的结果，而不是让爬虫变成lua插件，所以**vmr**能让用户体验更流畅和稳定。此外，**vmr**还支持了反向代理或者本地代理设置，多线程下载等，大大提高国内用户的下载体验。因此，不管你是老鸟还是菜鸟，**vmr**都能给你带来相当的便利。你不用再手动去找任何资源，就能轻松安装管理各种sdk版本，尝试新的语言，新的特性。最后，**vmr**将这些sdk或工具集中管理，对于有**洁癖**的人来说，也是福音。

[b站演示视频(不包含project锁定版本)](https://www.bilibili.com/video/BV1bZ421v7sD/)

[查看详细文档](https://gvcgo.github.io/vmr/)

------

<p id="2"></p>

### 功能特点

- 跨平台，支持Windows，Linux，MacOS
- 支持多种语言和工具，省心
- 更友好的TUI交互，尽量减少用户输入，同时不失灵活性
- 支持针对项目锁定SDK版本
- 支持反向代理设置和多线程下载，提高国内用户下载体验
- 版本爬虫与主项目分离，响应更快，稳定性更高
- 无需插件，开箱即用
- 无需docker，纯本地安装
- 简单易用，用较少的命令，实现了常见SDK版本管理器的所有功能(用户只需关注vmr的大约6个子命令即可)

------

### vmr的数据流

![framwork.png](https://github.com/moqsien/img_repo/raw/main/framework.png)

- [collector](https://github.com/gvcgo/collector) 收集SDK版本信息数据，并上传到**resources**（用户对此无感知）
- [resources](https://github.com/gvcgo/resources) 存储所有SDK的版本信息（用户对此无感知）
- [vmr](https://github.com/gvcgo/version-manager) 整个项目的用户接口

collector部署在远程服务器中，会定时获取SDK的最新版，并上传到resources仓库中，用户一般无需关心这些。
**Vmr** 从**resources**仓库获取版本信息, 用于给用户展示或者下载相应版本。

这样的结构，**增加了稳定性**，**响应更快速**，**用户体验更好**。

------

<p id="4"></p>  

### 一键安装/更新vm
- for **macos/linux**(复制下面的命令到terminal执行即可)
```bash
curl --proto '=https' --tlsv1.2 -ssf https://gvc.1710717.xyz/proxy/https://raw.githubusercontent.com/gvcgo/version-manager/main/scripts/install.sh | sh
```

- for **windows**(复制下面的命令到powershell中执行即可)
```powershell
powershell -nop -c "iex(new-object net.webclient).downloadstring('https://gvc.1710717.xyz/proxy/https://raw.githubusercontent.com/gvcgo/version-manager/main/scripts/install.ps1')"
```

- **一键更新功能**
```bash
vmr-update
```

**注意事项**：首次安装之后，如果当前命令行窗口找不到vmr命令，请使用source .zshrc或source .bashrc刷新环境变量。Windows用户无法刷新环境变量的，请关闭后另开一个新的Powershell。

------

<p id="5"></p> 

### 如何设置代理?

- **设置免费的反向代理**

```bash
# reverse proxy <https://gvc.1710717.xyz/proxy/> is available for free.
vmr set-reverse-proxy https://gvc.1710717.xyz/proxy/
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
| **install-self** | - | 安装vm到$home/.vm，用户一般无需关心 |
| **version** | - | 显示vm的版本信息 |
| **completion** | - | 生成关于不同shell的自动补全(支持bash、zsh、fish、powershell) |

------

**macos演示**

<!-- <a href="https://asciinema.org/a/647462" target="_blank"><img src="https://asciinema.org/a/647462.svg" /></a> -->
![demo](https://github.com/moqsien/img_repo/raw/main/vm.gif)

**windows演示**

![demo](https://github.com/moqsien/img_repo/raw/main/vm_win.gif)

**linux演示**

![demo](https://github.com/moqsien/img_repo/raw/main/vm_linux.gif)

------
<p id="9"></p>  

### 贡献者
> 感谢以下贡献者对本项目的贡献。
<a href="https://github.com/gvcgo/version-manager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=gvcgo/version-manager" />
</a>
