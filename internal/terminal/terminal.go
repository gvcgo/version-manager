package terminal

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/installer/install"
	"github.com/gvcgo/version-manager/internal/shell/sh"
	"github.com/gvcgo/version-manager/internal/terminal/term"
	"github.com/shirou/gopsutil/process"
)

/*
Prepares envs for pty/conpty.
*/
func addToPath(value string) {
	pathStr := os.Getenv("PATH")
	if runtime.GOOS == gutils.Windows {
		// Windows: Insert new path as second position (after first semicolon)
		paths := strings.Split(pathStr, ";")
		if len(paths) == 0 || pathStr == "" {
			// If PATH is empty, just set the new value
			pathStr = value
		} else if len(paths) == 1 {
			// If only one path exists, add new value as second
			pathStr = fmt.Sprintf("%s;%s", paths[0], value)
		} else {
			// Insert new value after the first path (second position)
			newPaths := make([]string, 0, len(paths)+1)
			newPaths = append(newPaths, paths[0])     // First path
			newPaths = append(newPaths, value)        // New path as second
			newPaths = append(newPaths, paths[1:]...) // Rest of paths
			pathStr = strings.Join(newPaths, ";")
		}
	} else {
		// Unix/Linux: Simple prepend (original behavior)
		pathStr = fmt.Sprintf("%s:%s", value, pathStr)
	}
	os.Setenv("PATH", pathStr)
}

func addEnv(key, value string) {
	if strings.ToLower(key) == "path" {
		addToPath(value)
	} else {
		os.Setenv(key, value)
	}
}

/*
Remove default global envs for an SDK.
*/
func ModifyPathForPty(appName string) {
	pathStr := os.Getenv("PATH")
	symbolicPath := filepath.Join(install.GetSDKVersionDir(appName), appName)
	sep := ":"
	if runtime.GOOS == gutils.Windows {
		sep = ";"
	}
	eList := []string{}
	for _, pStr := range strings.Split(pathStr, sep) {
		if pStr == symbolicPath {
			continue
		}
		eList = append(eList, pStr)
	}
	os.Setenv("PATH", strings.Join(eList, sep))
}

/*
pty for Unix.
conpty for Windows.
*/
type PtyTerminal struct {
	Terminal term.Terminal
}

func NewPtyTerminal() (p *PtyTerminal) {
	p = &PtyTerminal{
		Terminal: term.NewTerminal(),
	}
	return
}

func (p *PtyTerminal) AddEnv(key, value string) {
	addEnv(key, value)
}

func (p *PtyTerminal) FindShellCommand() (command string) {
	if runtime.GOOS == gutils.Windows {
		pp, _ := process.NewProcess(int32(os.Getppid()))
		command, _ = pp.Name()
		if command == "" {
			command = "powershell.exe"
		}
	} else {
		command = "/bin/sh"
		if shell := os.Getenv("SHELL"); shell != "" {
			command = shell
		}
	}
	return
}

func (p *PtyTerminal) Run() {
	command := p.FindShellCommand()
	// Disable reading vmr.sh/vmr.fish for the new pseudo-shell.
	if runtime.GOOS != gutils.Windows {
		os.Setenv(sh.VMDisableEnvName, "111")
	}

	if p.Terminal != nil {
		if err := p.Terminal.Record(command, os.Environ()...); err != nil {
			gprint.PrintError("open pty failed: %+v", err)
			return
		}
	} else {
		gprint.PrintError("no pty found")
	}
	os.Exit(0)
}

func GetTerminalSize() (height, width int, err error) {
	t := term.NewTerminal()
	return t.Size()
}

func RunTerminal() {
	// already run a terminal. Allow/Disallow nested sessions.
	if os.Getenv(sh.VMDisableEnvName) != "" && !cnf.DefaultConfig.AllowNestedSessions {
		return
	}

	var command string
	if runtime.GOOS == gutils.Windows {
		pp, _ := process.NewProcess(int32(os.Getppid()))
		command, _ = pp.Name()
		if command == "" {
			command = "powershell.exe"
		}
	} else {
		command = "/bin/sh"
		if shell := os.Getenv("SHELL"); shell != "" {
			command = shell
		}
	}

	// if runtime.GOOS != gutils.Windows {
	// 	os.Setenv(sh.VMDisableEnvName, "111")
	// }
	os.Setenv(sh.VMDisableEnvName, "111")

	if command != "" {
		cmd := exec.Command(command)
		cmd.Env = os.Environ()
		cmd.Stdout = os.Stdout
		cmd.Stdin = os.Stdin
		cmd.Stderr = os.Stderr
		cmd.Start()
		cmd.Wait()
	}
}
