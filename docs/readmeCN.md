<p style="" align="center">
  <!-- <img src="https://github.com/moqsien/img_repo/raw/main/vm_header_photo_2.png" alt="logo" width="720" height="240"> -->
  <img src="https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr-logo.png" alt="logo" width="360" height="120">
</p>

[![go report card](https://img.shields.io/badge/go%20report-a+-brightgreen.svg?style=for-the-badge)](https://goreportcard.com/report/github.com/gvcgo/version-manager)
[![github license](https://img.shields.io/github/license/gvcgo/version-manager?style=for-the-badge)](license)
[![github release](https://img.shields.io/github/v/release/gvcgo/version-manager?display_name=tag&style=for-the-badge)](https://github.com/gvcgo/version-manager/releases)
[![prs card](https://img.shields.io/badge/prs-vmr-cyan.svg?style=for-the-badge)](https://github.com/gvcgo/version-manager/pulls)
[![issues card](https://img.shields.io/badge/issues-vmr-pink.svg?style=for-the-badge)](https://github.com/gvcgo/version-manager/issues)
[![versions repo card](https://img.shields.io/badge/versions-repo-blue.svg?style=for-the-badge)](https://github.com/gvcgo/resources)
[![Go Reference](https://pkg.go.dev/badge/github.com/gvcgo/version-manager.svg)](https://pkg.go.dev/github.com/gvcgo/version-manager)
[![codecov](https://codecov.io/gh/gvcgo/version-manager/graph/badge.svg?token=ITQNVHMKRH)](https://codecov.io/gh/gvcgo/version-manager)

[中文](https://github.com/gvcgo/version-manager/blob/main/docs/readmecn.md) | [en](https://github.com/gvcgo/version-manager)

- [](#)
  - [VMR简介](#vmr简介)
  - [功能特点](#功能特点)
  - [安装](#安装)
  - [支持的部分SDK](#支持的部分sdk)
  - [贡献者](#贡献者)
  - [欢迎star](#欢迎star)
  - [特别感谢](#特别感谢)

 <div align=center><img src="https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr_wordcloud.png" width="70%"></div>
------

<!-- ![demo](https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr.gif) -->
<div align=center><img src="https://image-acc.vmr.dpdns.org/vmr.gif"></div>

------
<p id="1"></p>

### VMR简介

VMR是一款**简单**，**跨平台**，且经过**良好设计**的版本管理器，用于管理多种SDK以及其他工具。它完全是为了通用目的而创建的。

你可能已经听说过fnm，gvm，nvm，pyenv，phpenv等SDK版本管理工具。然而，它们很多都不能管理多种编程语言。像asdf-vm这样的管理器支持多种语言，但只适用于类unix系统，并且看起来非常复杂。因此，VMR的出现主要就是为了解决这些问题。

[查看详细文档](https://vdocs.vmr.dpdns.org/zh-cn/)

------

### 功能特点

- 跨平台，支持**Windows**，**Linux**，**MacOS**
- 支持**多种语言和工具**，省心
- 受到lazygit的启发，拥有更友好的TUI，更符合直觉，且**无需记忆任何命令**
- 同时也**支持CLI模式**，你可以根据自己的喜好选择使用CLI模式或者TUI模式
- 支持针**对项目锁定SDK版本**
- 支持**反向代理**/**本地代理**设置，提高国内用户下载体验
- 相比于其他SDK管理器，拥有**更优秀的架构设计**，**响应更快**，**稳定性更高**
- **无需麻烦的插件**，开箱即用
- **无需docker**，纯本地安装，效率更高
- 更高的**可扩展性**，甚至可以通过使用**conda**来支持数以千计的应用
- 支持多种Shell，包括**bash**，**zsh**，**fish**, **powershell**, **git-bash**

------

### 安装

- MacOS/Linux
```bash
curl --proto '=https' --tlsv1.2 -sSf https://scripts.vmr.dpdns.org | sh
```
- Windows
```bash
powershell -c "irm https://scripts.vmr.dpdns.org/windows | iex"
```

**注意**：安装之后，请记得阅读[文档](https://vdocs.vmr.dpdns.org//zh-cn/)，尤其是国内用户存在访问github受限的情况，你遇到的问题应该都在文档中了。

如果你仍然无法安装vmr, 请转到[release](https://github.com/gvcgo/version-manager/releases)页面下载zip文件, 解压之后在终端运行如下命令即可安装:

```bash
./vmr install-self
```

------

### 支持的部分SDK

[bun](https://bun.sh/), [clang](https://clang.llvm.org/), [clojure](https://clojure.org/), [codon](https://github.com/exaloop/codon), [crystal](https://crystal-lang.org/), [deno](https://deno.com/), [dlang](https://dlang.org/), [dotnet](https://dotnet.microsoft.com/), [elixir](https://elixir-lang.org/), [erlang](https://www.erlang.org/), [flutter](https://flutter.dev/), [gcc](https://gcc.gnu.org/), [gleam](https://gleam.run/), [go](https://go.dev/), [groovy](http://www.groovy-lang.org/), [jdk](https://bell-sw.com/pages/downloads/), [julia](https://julialang.org/), [kotlin](https://kotlinlang.org/), [lfortran](https://lfortran.org/), [lua](https://www.lua.org/), [nim](https://nim-lang.org/), [node](https://nodejs.org/en), [odin](http://odin-lang.org/), [perl](https://www.perl.org/), [php](https://www.php.net/), [pypy](https://www.pypy.org/), [python](https://www.python.org/), [r](https://www.r-project.org/), [ruby](https://www.ruby-lang.org/en/), [rust](https://www.rust-lang.org/), [scala](https://www.scala-lang.org/), [typst](https://typst.app/), [v](https://vlang.io/), [zig](https://ziglang.org/), [以及更多...](https://vdocs.vmr.dpdns.org/zh-cn/starts/sdklist/#supported-lsp)

------
<p id="9"></p>

### 贡献者
> 感谢以下贡献者对本项目的贡献。

<a href="https://github.com/gvcgo/version-manager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=gvcgo/version-manager" />
</a>

一些后面需要优化的问题放在讨论区了，感兴趣的同学可以到[discussions](https://github.com/gvcgo/version-manager/discussions)查看。注意，大家在提出问题之前，可以先阅读**VMR**的官方文档，避提问重复或者与**VMR**的总体设计理念相违背。同时，**VMR**也十分期待有时间和精力的同学，参与到**VMR**项目的优化和改进中来。

------

### 欢迎star

**如果本项目对您的工作和学习有所帮助，欢迎🌟🌟🌟**。

------

### 特别感谢

<div></a><a href="https://conda-forge.org/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/anaconda/anaconda-original-wordmark.svg" align="middle" height="128" /></a><a href="https://servicecomb.apache.org/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/apache/apache-original-wordmark.svg" align="middle" height="128"/></a><a href="https://code.visualstudio.com/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/vscode/vscode-original-wordmark.svg" align="middle" width="64"/></a><a href="https://dash.domain.digitalplat.org/"><img src="https://github.com/gvcgo/version-manager/blob/main/docs/freedomain_logo.jpg" align="middle" width="64"/><a href="https://www.cloudflare.com/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/cloudflare/cloudflare-original-wordmark.svg" align="middle" width="64" /></a></div>
<!-- <a href="https://evolution-host.com/"><img src="https://evolution-host.com/images/branding/newLogoBlack.png" align="middle" width="64"/> -->

------

### Star History

![Star History Chart](https://api.star-history.com/svg?repos=gvcgo/version-manager&type=Date)
