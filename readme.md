### version-manager(vm)
------------------------
**vm** is a simple version manager for programming **languages** and **tools**. It is totally created for general purpose.

### What's supported?

- programming languages
  - **java**(jdk, maven, gradle)
  - **kotlin**
  - **go**
  - **python**(miniconda, python)
  - **php**
  - **javascript/typescript**(node, bun, deno)
  - **dart**(flutter, dart)
  - **julia**
  - **c/c++**(cygwin, msys2)
  - **rust**(rustup-init)
  - **vlang**(v, v-analyzer)
  - **zig**(zig, zls)
  - **typst**(typst, typst-lsp)
- tools
  - **commandlinetools**(for android)
  - **git**(for windows only)
  - **protoc**(protobuf)
  - **gsudo**(for windows only)
  - **vscode**
  - **neovim**
  - **fd**
  - **fzf**
  - **ripgrep**
  - **tree-sitter**

### Usage
```bash
~ % vm -h

vm <Command> <SubCommand> --flags args...

Usage:
   [command]

Command list:
  clear-cache       Clear cached zip files for an app.
  install-self      Installs version manager.
  local             Shows installed versions for an app.
  search            Shows the available versions of an application.
  set-proxy         Sets proxy for version manager.
  set-reverse-proxy Sets reverse proxy for version manager.
  show              Shows the supported applications.
  uninstall         Uninstalls a version or an app.
  use               Installs and switches to specified version.

Additional Commands:
  completion        Generate the autocompletion script for the specified shell
  help              Help about any command

Flags:
  -h, --help   help for this command

Use " [command] --help" for more information about a command.
```

```bash
~ % vm use -h

Example: vm use go@1.22.1

Usage:
   use [flags]

Aliases:
  use, u

Flags:
  -h, --help              help for use
  -c, --mirror_in_china   Downlowd from mirror sites in China.
  -t, --threads int       Number of threads to use for downloading. (default 1)
```

```bash
~ % vm search go

  go available versions
 ──────────────────────────────────────────────────────────────
  1.22.1
  1.22.0
  1.22rc2
  1.22rc1
  1.21.8
  1.21.7
  1.21.6
  1.21.5
  1.21.4
  1.21.3
  1.21.2
  1.21.1
  1.21.0
  1.21rc4
  1.21rc3
  1.21rc2
  1.20.14
  1.20.13
  1.20.12
  1.20.11
  1.20.10
  1.20.9
  1.20.8
  1.20.7
  1.20.6

Press "↑/k" to move up, "↓/j" to move down, "q" to quit.
```

### Installation

Will be available soon.
