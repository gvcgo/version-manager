<p style="" align="center">
  <!-- <img src="https://github.com/moqsien/img_repo/raw/main/vm_header_photo_2.png" alt="Logo" width="720" height="240"> -->
  <img src="https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr_logo_trans.png" alt="Logo" width="360" height="120">
</p>

[![Go Report Card](https://img.shields.io/badge/go%20report-A+-brightgreen.svg?style=for-the-badge)](https://goreportcard.com/report/github.com/gvcgo/version-manager)
[![GitHub License](https://img.shields.io/github/license/gvcgo/version-manager?style=for-the-badge)](LICENSE)
[![GitHub Release](https://img.shields.io/github/v/release/gvcgo/version-manager?display_name=tag&style=for-the-badge)](https://github.com/gvcgo/version-manager/releases)
[![PRs Card](https://img.shields.io/badge/PRs-vm-cyan.svg?style=for-the-badge)](https://github.com/gvcgo/version-manager/pulls)
[![Issues Card](https://img.shields.io/badge/Issues-vm-pink.svg?style=for-the-badge)](https://github.com/gvcgo/version-manager/issues)
[![Versions Repo Card](https://img.shields.io/badge/Versions-repo-blue.svg?style=for-the-badge)](https://github.com/gvcgo/resources)

[中文](https://github.com/gvcgo/version-manager/blob/main/docs/readmeCN.md) | [En](https://github.com/gvcgo/version-manager)

- [version-manager(vmr)](#version-managervmr)
- [Features](#features)
- [Installation](#installation)
- [What's supported?](#whats-supported)
- [Contributors](#contributors)
- [Thanks to](#thanks-to)

 <div align=center><img src="https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr_wordcloud.png" width="70%"></div>

------

<div align=center><img src="https://cdn.jsdelivr.net/gh/moqsien/img_repo@main/vmr_preview.gif"></div>

------
<p id="1"></p>  

### version-manager(vmr)

🔥🔥🔥**VMR** is a **simple**, **cross-platform**, and **well-designed** version manager for multiple sdks and tools. It is totally created for general purpose.

Maybe you've already heard of fnm, gvm, nvm, pyenv, phpenv, etc. However, none of them can manage multiple programming languages. Managers like asdf-vm support multiple languages, but only works on unix-like systems, and annoyingly makes things look complicated. Therefore, **VMR** comes.

[See docs for details](https://docs.vmr.us.kg/) 

[FAQs](https://docs.vmr.us.kg/#/faq)

------

### Features

- Cross-platform, supports **Windows**, **Linux**, **MacOS**.
- Supports **multiple languages and tools**.
- Nicer TUI, inpsired by lazygit, more intuitive, **no need to remember any commands**.
- Supports **locking SDK version for each project**.
- Supports **Reverse Proxy**/**Local Proxy**, improves your download experience.
- Well-designed, **faster** response and **higher** stability.
- **No plugins** needed, just out of the box.
- Installs SDKs **in local disk** instead of docker containers.
- **High extendability**, even for thousands of applications(through **conda**).

------

### Installation

- MacOS/Linux
```bash
curl --proto '=https' --tlsv1.2 -sSf https://scripts.vmr.us.kg | sh
```
- Windows
```bash
powershell -c "irm https://scripts.vmr.us.kg/windows | iex"
```

**Note**: Please remember to read the [docs](https://docs.vmr.us.kg/), as the problems you encounter may be caused by your improper usage.

------

### What's supported?

[bun](https://bun.sh/), [clang](https://clang.llvm.org/), [clojure](https://clojure.org/), [codon](https://github.com/exaloop/codon), [deno](https://deno.com/), [dlang](https://dlang.org/), [dotnet](https://dotnet.microsoft.com/), [elixir](https://elixir-lang.org/), [erlang](https://www.erlang.org/), [flutter](https://flutter.dev/), [gcc](https://gcc.gnu.org/), [gleam](https://gleam.run/), [go](https://go.dev/), [groovy](http://www.groovy-lang.org/), [jdk](https://bell-sw.com/pages/downloads/), [julia](https://julialang.org/), [kotlin](https://kotlinlang.org/), [lfortran](https://lfortran.org/), [lua](https://www.lua.org/), [nim](https://nim-lang.org/), [node](https://nodejs.org/en), [odin](http://odin-lang.org/), [perl](https://www.perl.org/), [php](https://www.php.net/), [pypy](https://www.pypy.org/), [python](https://www.python.org/), [r](https://www.r-project.org/), [ruby](https://www.ruby-lang.org/en/), [rust](https://www.rust-lang.org/), [scala](https://www.scala-lang.org/), [typst](https://typst.app/), [v](https://vlang.io/), [zig](https://ziglang.org/)

------

### Contributors
> Thanks to the following people who have contributed to this project.

<a href="https://github.com/gvcgo/version-manager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=gvcgo/version-manager" />
</a>

------

### Thanks to

<div><a href="https://evolution-host.com/"><img src="https://evolution-host.com/images/branding/newLogoBlack.png" align="middle" width="64"/></a><a href="https://conda-forge.org/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/anaconda/anaconda-original-wordmark.svg" align="middle" height="128" /></a><a href="https://servicecomb.apache.org/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/apache/apache-original-wordmark.svg" align="middle" height="128"/></a><a href="https://code.visualstudio.com/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/vscode/vscode-original-wordmark.svg" align="middle" width="64"/></a><a href="https://register.us.kg/"><img src="https://register.us.kg/static/img/logo.jpg" align="middle" width="64"/><a href="https://www.cloudflare.com/"><img src="https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/cloudflare/cloudflare-original-wordmark.svg" align="middle" width="64" /></a></div>