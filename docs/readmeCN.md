<p style="" align="center">
  <!-- <img src="https://github.com/moqsien/img_repo/raw/main/vm_header_photo_2.png" alt="logo" width="720" height="240"> -->
  <img src="https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr_logo_trans.png" alt="logo" width="360" height="120">
</p>

[![go report card](https://img.shields.io/badge/go%20report-a+-brightgreen.svg?style=for-the-badge)](https://goreportcard.com/report/github.com/gvcgo/version-manager)
[![github license](https://img.shields.io/github/license/gvcgo/version-manager?style=for-the-badge)](license)
[![github release](https://img.shields.io/github/v/release/gvcgo/version-manager?display_name=tag&style=for-the-badge)](https://github.com/gvcgo/version-manager/releases)
[![prs card](https://img.shields.io/badge/prs-vm-cyan.svg?style=for-the-badge)](https://github.com/gvcgo/version-manager/pulls)
[![issues card](https://img.shields.io/badge/issues-vm-pink.svg?style=for-the-badge)](https://github.com/gvcgo/version-manager/issues)
[![versions repo card](https://img.shields.io/badge/versions-repo-blue.svg?style=for-the-badge)](https://github.com/gvcgo/resources)

[中文](https://github.com/gvcgo/version-manager/blob/main/docs/readmecn.md) | [en](https://github.com/gvcgo/version-manager)

- [](#)
  - [VMR简介](#vmr简介)
  - [功能特点](#功能特点)
  - [支持的部分SDK](#支持的部分sdk)
  - [贡献者](#贡献者)
  - [特别感谢](#特别感谢)

 <div align=center><img src="https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr_wordcloud.png" width="70%"></div>
------

<!-- ![demo](https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr.gif) -->
<div align=center><img src="https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr_preview.gif"></div>

------
<p id="1"></p>  

### VMR简介

VMR是一款**简单**，**跨平台**，且经过**良好设计**的版本管理器，用于管理多种SDK以及其他工具。它完全是为了通用目的而创建的。

你可能已经听说过fnm，gvm，nvm，pyenv，phpenv等SDK版本管理工具。然而，它们很多都不能管理多种编程语言。像asdf-vm这样的管理器支持多种语言，但只适用于类unix系统，并且看起来非常复杂。因此，VMR的出现主要就是为了解决这些问题。

[查看详细文档](https://gvcgo.github.io/vdocs/#/zh-cn/)

**注意**： v0.6.x改版非常大，主要是为了更好的用户体验，以及更清晰的代码架构，方便用户使用的同时，也方便更多有兴趣的开发者参与进来。所以，放弃了对老版本的兼容。在安装v0.6.x的过程中，会提示**是否删除已有的老版本**，只有删除老版本(包含通过老版本安装的SDK)，才能继续安装v0.6.x。相信v0.6.x能不负众望，给同学们带来更好的使用体验。老版本的vmr的SDK版本仓库还会继续维持一段时间，也就是说老版本在一段时间内仍然可以正常使用，但强烈建议尽快升级。因为新版本不仅能得到更好的维护，覆盖面也更广，并且更简单高效。

------

### 功能特点

- 跨平台，支持**Windows**，**Linux**，**MacOS**
- 支持**多种语言和工具**，省心
- 受到lazygit的启发，拥有更友好的TUI，更符合直觉，且**无需记忆任何命令**
- 支持针**对项目锁定SDK版本**
- 支持**反向代理**/**本地代理**设置，提高国内用户下载体验
- 相比于其他SDK管理器，拥有**更优秀的架构设计**，**响应更快**，**稳定性更高**
- **无需麻烦的插件**，开箱即用
- **无需docker**，纯本地安装，效率更高
- 更高的**可扩展性**，甚至可以通过使用**conda**来支持数以千计的应用

------

### 支持的部分SDK

[bun](https://bun.sh/), [clang](https://clang.llvm.org/), [clojure](https://clojure.org/), [codon](https://github.com/exaloop/codon), [deno](https://deno.com/), [dlang](https://dlang.org/), [dotnet](https://dotnet.microsoft.com/), [elixir](https://elixir-lang.org/), [erlang](https://www.erlang.org/), [flutter](https://flutter.dev/), [gcc](https://gcc.gnu.org/), [gleam](https://gleam.run/), [go](https://go.dev/), [groovy](http://www.groovy-lang.org/), [jdk](https://bell-sw.com/pages/downloads/), [julia](https://julialang.org/), [kotlin](https://kotlinlang.org/), [lfortran](https://lfortran.org/), [lua](https://www.lua.org/), [nim](https://nim-lang.org/), [node](https://nodejs.org/en), [odin](http://odin-lang.org/), [perl](https://www.perl.org/), [php](https://www.php.net/), [pypy](https://www.pypy.org/), [python](https://www.python.org/), [r](https://www.r-project.org/), [ruby](https://www.ruby-lang.org/en/), [rust](https://www.rust-lang.org/), [scala](https://www.scala-lang.org/), [typst](https://typst.app/), [v](https://vlang.io/), [zig](https://ziglang.org/)

------
<p id="9"></p>  

### 贡献者
> 感谢以下贡献者对本项目的贡献。

<a href="https://github.com/gvcgo/version-manager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=gvcgo/version-manager" />
</a>

一些后面需要优化的问题放在讨论区了，感兴趣的同学可以到[discussions](https://github.com/gvcgo/version-manager/discussions)查看。注意，大家在提出问题之前，可以先阅读**VMR**的官方文档，避提问重复或者与**VMR**的总体设计理念相违背。同时，**VMR**也十分期待有时间和精力的同学，参与到**VMR**项目的优化和改进中来。

------

### 特别感谢

<div><a href="https://conda-forge.org/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/anaconda/anaconda-original-wordmark.svg" align="middle" height="128" /></a><a href="https://servicecomb.apache.org/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/apache/apache-original-wordmark.svg" align="middle" height="128"/></a><a href="https://code.visualstudio.com/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/vscode/vscode-original-wordmark.svg" align="middle" width="64"/></a></div>
