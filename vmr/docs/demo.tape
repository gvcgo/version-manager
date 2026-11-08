# vmr feature demo.
#
# Run from this directory:
#     vhs demo.tape
#
# Produces demo.gif next to this file. Requires vmr, Go, Node.js and the
# JetBrainsMono Nerd Font Mono font to be installed locally.

Output demo.gif

Set Shell "bash"
Set FontFamily "JetBrainsMono Nerd Font Mono"
Set FontSize 22
Set LetterSpacing 0.5
Set LineHeight 1.25
Set Width 1280
Set Height 760
Set Padding 24
Set Theme "Catppuccin Mocha"
Set TypingSpeed 25ms
Set Framerate 20
Set PlaybackSpeed 1.0

Hide
Type "exec bash --norc -i"
Enter
Sleep 1s
Type "set +H"
Enter
Type 'export PS1="\[\e[38;5;114m\]vmr\[\e[0m\] \[\e[38;5;117m\]\W\[\e[0m\] $ "'
Enter
Type 'printf "%s\n" "#!/bin/sh" "PS1=\"(session) vmr \\W \$ \"" "export PS1" "exec /bin/bash --norc -i" > /tmp/vmr-session-shell; chmod +x /tmp/vmr-session-shell; export SHELL=/tmp/vmr-session-shell'
Enter
Type "cd /tmp && rm -rf vmr-demo && mkdir vmr-demo && cd vmr-demo"
Enter
Type "source <(vmr init bash)"
Enter
Type "vmr use -p lua@5.4.6 >/dev/null 2>&1"
Enter
Sleep 25s
Type 'rm -rf "$HOME/.vmr/cache/agg/1.9.0"'
Enter
Type "clear"
Enter
Sleep 1s
Show

# 1. Plugin ecosystem
Type "# One binary manages 100+ SDKs; no admin rights, no system pollution."
Sleep 4s
Ctrl+U
Sleep 400ms
Type "vmr version"
Enter
Sleep 1.5s
Type "vmr plugins | head -n 12"
Enter
Sleep 3s

# 2. Search, coexist, switch
Ctrl+L
Sleep 300ms
Type "# Search live versions; switch between them by repointing a symlink."
Sleep 4s
Ctrl+U
Sleep 400ms
Type "vmr search go | head -n 6"
Enter
Sleep 3s
Type "vmr local go"
Enter
Sleep 2.5s
Type "vmr use go@1.26.10"
Enter
Sleep 2s
Type "go version"
Enter
Sleep 2s
Type "vmr use go@1.27.3"
Enter
Sleep 2s
Type "go version"
Enter
Sleep 2.5s

# 3. Download progress bar
Ctrl+L
Sleep 300ms
Type "# Downloads show live bytes, speed and ETA while chunks run in parallel."
Sleep 4s
Ctrl+U
Sleep 400ms
Type@5ms "vmr use agg@1.9.0; vmr local agg | tail -n 2; vmr uninstall agg@1.9.0"
Enter
Sleep 18s

# 4. Project-level pinning
Ctrl+L
Sleep 300ms
Type "# Commit .vmr.lock; every cd into the project switches versions."
Sleep 4s
Ctrl+U
Sleep 400ms
Type "mkdir demo-app && cd demo-app"
Enter
Sleep 1s
Type "vmr use -l go@1.26.10"
Enter
Sleep 2s
Type "cat .vmr.lock; echo"
Enter
Sleep 2s
Type "go version"
Enter
Sleep 2s
Type "exit"
Enter
Sleep 1.5s
Type "cd .."
Enter
Sleep 300ms
Type "cd demo-app"
Enter
Sleep 2.5s
Type "go version"
Enter
Sleep 2s
Type "exit"
Enter
Sleep 1.5s
Type "cd .."
Enter
Sleep 1s

# 5. Session isolation
Ctrl+L
Sleep 300ms
Type "# Session mode gives one shell a temporary version, restored on exit."
Sleep 4s
Ctrl+U
Sleep 400ms
Type "vmr session"
Enter
Sleep 1s
Type "vmr use -s go@1.26.5"
Enter
Sleep 2.5s
Type "go version"
Enter
Sleep 2s
Type "exit"
Enter
Sleep 2s
Type "go version"
Enter
Sleep 2.5s

# 6. Freeze and replay
Ctrl+L
Sleep 300ms
Type "# Freeze versions to text; replay the manifest on a new machine."
Sleep 4s
Ctrl+U
Sleep 400ms
Type "vmr freeze demo-freeze.txt"
Enter
Sleep 1.5s
Type "head -n 9 demo-freeze.txt"
Enter
Sleep 2.5s
Type "grep -E '^(go|node)@' demo-freeze.txt | tail -n 2 > stack.txt"
Enter
Sleep 1s
Type "cat stack.txt"
Enter
Sleep 1.5s
Type "vmr use -f stack.txt"
Enter
Sleep 2s
Type "go version && node -v"
Enter
Sleep 2.5s

# 7. Built-in pixi / conda-forge backend
Ctrl+L
Sleep 300ms
Type "# Built-in pixi installs any conda-forge package, then cleans up."
Sleep 4s
Ctrl+U
Sleep 400ms
Type@5ms "vmr search -p lua 2>/dev/null | head -n 6; vmr use -p lua@5.4.6; ~/.vmr/versions/lua_versions/lua/bin/lua -v; vmr uninstall lua@5.4.6"
Enter
Sleep 14s

# 8. Lua plugin templates
Ctrl+L
Sleep 300ms
Type "# Add SDKs with Lua plugin templates bundled in the binary."
Sleep 4s
Ctrl+U
Sleep 400ms
Type "vmr template json | head -n 14"
Enter
Sleep 3s

# 9. MCP for AI clients
Ctrl+L
Sleep 300ms
Type "# AI clients can drive vmr over MCP on stdio."
Sleep 4s
Ctrl+U
Sleep 400ms
Type@3ms 'printf "%s\n" "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\"}}" "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"vmr_list_installed\",\"arguments\":{\"sdk\":\"go\"}}}" | vmr mcp'
Enter
Sleep 6s

Hide
Type "cd /tmp && rm -rf vmr-demo"
Enter
Type "rm -f /tmp/vmr-session-shell"
Enter
