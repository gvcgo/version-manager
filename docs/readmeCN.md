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

- [vmr简介](#vmr简介)
- [功能特点](#功能特点)
- [贡献者](#贡献者)

------
🔥🔥🔥 **v0.6.1 Preview** 已发布!

请前往[release](https://github.com/gvcgo/version-manager/releases/tag/v0.6.1)查看惊喜！

------

<!-- ![demo](https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr.gif) -->
<div align=center><img src="https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr_new.gif"></div>

------
<p id="1"></p>  

### vmr简介

**vmr** 是一个简单，跨平台，并且经过良好测试的版本管理工具。它完全是为了通用目的而创建的。无需插件，开箱即用。

可能你已经听说过**fnm**, **gvm**, **nvm**, **pyenv**, **phpenv** 等工具。然而，这些工具都不能管理多种编程语言，甚至有些看起来会比较复杂。而**vmr**支持了国内程序员常用的几乎所有编程语言，并且支持了vlang、zig、typst等新兴的有一定潜力的语言，它隔离并缓存了爬虫部分的结果，而不是让爬虫变成lua插件，所以**vmr**能让用户体验更流畅和稳定。此外，**vmr**还支持了反向代理或者本地代理设置，多线程下载等，大大提高国内用户的下载体验。因此，不管你是老鸟还是菜鸟，**vmr**都能给你带来相当的便利。你不用再手动去找任何资源，就能轻松安装管理各种sdk版本，尝试新的语言，新的特性。最后，**vmr**将这些sdk或工具集中管理，对于有**洁癖**的人来说，也是福音。

[b站演示视频(不包含project锁定版本)](https://www.bilibili.com/video/BV1bZ421v7sD/)

[查看详细文档](https://gvcgo.github.io/vmrdocs/#/zh-cn/)

------

### 功能特点

- 跨平台，支持Windows，Linux，MacOS
- 支持多种语言和工具，省心
- 更友好的TUI交互，尽量减少用户输入，同时不失灵活性
- 支持针对项目锁定SDK版本
- 支持反向代理设置和多线程下载，提高国内用户下载体验
- 版本爬虫与主项目分离，响应更快，稳定性更高
- 无需插件，开箱即用
- 无需docker，纯本地安装
- 简单易用，用较少的命令，实现了常见SDK版本管理器的所有功能(用户只需关注VMR的大约6个子命令即可)。

------
<p id="9"></p>  

### 贡献者
> 感谢以下贡献者对本项目的贡献。
<a href="https://github.com/gvcgo/version-manager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=gvcgo/version-manager" />
</a>

一些后面需要优化的问题放在讨论区了，感兴趣的同学可以到[discussions](https://github.com/gvcgo/version-manager/discussions)查看。注意，大家在提出问题之前，可以先阅读**VMR**的官方文档，避提问重复或者与**VMR**的总体设计理念相违背。同时，**VMR**也十分期待有时间和精力的同学，参与到**VMR**项目的优化和改进中来。
