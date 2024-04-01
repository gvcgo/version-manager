package terminal

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/asciinema/terminal"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
)

/*
Prepares envs for pty/conpty.
*/
func addToPath(value string) {
	pathStr := os.Getenv("PATH")
	if runtime.GOOS == gutils.Windows {
		pathStr = fmt.Sprintf("%s;%s", value, pathStr)
	} else {
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
pty for Unix.
conpty for Windows.
*/
type PtyTerminal struct {
	AppName  string
	Terminal terminal.Terminal
}

func NewPtyTerminal(appName string) (p *PtyTerminal) {
	p = &PtyTerminal{
		AppName:  appName,
		Terminal: terminal.NewTerminal(),
	}
	return
}

func (p *PtyTerminal) AddEnv(key, value string) {
	addEnv(key, value)
}

func (p *PtyTerminal) modifyPath() {
	pathStr := os.Getenv("PATH")
	symbolicPath := filepath.Join(conf.GetVMVersionsDir(p.AppName), p.AppName)
	binPath := conf.GetAppBinDir()
	sep := ":"
	if runtime.GOOS == gutils.Windows {
		sep = ";"
	}
	eList := []string{}
	for _, pStr := range strings.Split(pathStr, sep) {
		if pStr == binPath || pStr == symbolicPath {
			continue
		}
		eList = append(eList, pStr)
	}
	os.Setenv("PATH", strings.Join(eList, sep))
}

func (p *PtyTerminal) Run() {
	command := "C:\\WINDOWS\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
	if ok, _ := gutils.PathIsExist(command); !ok {
		command = "powershell.exe"
	}
	if runtime.GOOS != "windows" {
		command = "/bin/sh"
		if shell := os.Getenv("SHELL"); shell != "" {
			command = shell
		}
	}
	p.modifyPath()

	// Disable reading vm_env.sh for the new pseudo-shell.
	if runtime.GOOS != gutils.Windows {
		os.Setenv(conf.VMDiableEnvName, "111")
	}

	if p.Terminal != nil {
		if err := p.Terminal.Record(command, &NilWriter{}, os.Environ()...); err != nil {
			gprint.PrintError("open pty failed: %+v", err)
			return
		}
	} else {
		gprint.PrintError("no pty found")
	}
	os.Exit(0)
}

type NilWriter struct{}

func (nw *NilWriter) Write(p []byte) (n int, err error) {
	// doing nothing.
	return len(p), nil
}
