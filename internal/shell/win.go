//go:build windows

package shell

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"syscall"
	"unsafe"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/shell/sh"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"golang.org/x/sys/windows/registry"
)

const (
	EnvironmentName string = "Environment"
	PathEnvName     string = "path"
)

const oldPwsHook string = `function cdhook {
    $TRUE_FALSE=(Test-Path $args[0])
    if ( $TRUE_FALSE -eq "True" )
    {
        chdir $args[0]
        vmr use -E
    }
}

function vmrsource {
	$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}

Set-Alias -Name cd -Option AllScope -Value cdhook
Set-Alias -Name source -Value vmrsource`

// PowershellHook for Powershell
const PowershellHook string = `# cd hook start
function cdhook {
    $TRUE_FALSE=(Test-Path $args[0])
    if ( $TRUE_FALSE -eq "True" )
    {
        chdir $args[0]
        vmr use -E
    }
}

function vmrsource {
	$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}

Set-Alias -Name cd -Option AllScope -Value cdhook
Set-Alias -Name source -Value vmrsource

if ( "" -eq "$env:VMR_CD_INIT" )
{
    $env:VMR_CD_INIT="vmr_cd_init"
    cd "$(-split $(pwd))"
}
# cd hook end`

var _ Sheller = (*Shell)(nil)

const (
	VMR_VERSIONS_ENV    = "VMR_VERSIONS"
	VMR_VERSIONS_PREFIX = `%VMR_VERSIONS%`
)

func TidyWindowsPathEnv(pathStr string) (newPath string) {
	var VersionsDir = cnf.GetVersionsDir()

	if os.Getenv(VMR_VERSIONS_ENV) != VersionsDir {
		shell := NewShell()
		shell.SetEnv(VMR_VERSIONS_ENV, VersionsDir)
	}

	newPath = pathStr
	if strings.Contains(pathStr, VersionsDir) {
		newPath = strings.ReplaceAll(pathStr, VersionsDir, VMR_VERSIONS_PREFIX)
	}
	return
}

type Shell struct {
	sh.Sheller
	Key     registry.Key
	KeyInfo *registry.KeyInfo
}

func NewShell() *Shell {
	s := &Shell{}
	s.getKeyInfo()
	return s
}

func (s *Shell) getKeyInfo() {
	if s.KeyInfo == nil {
		var err error
		s.Key, err = registry.OpenKey(registry.CURRENT_USER, "Environment", registry.ALL_ACCESS)
		if err != nil {
			gprint.PrintError("Get windows registry key failed: %+v", err)
			return
		}

		s.KeyInfo, err = s.Key.Stat()
		if err != nil {
			gprint.PrintError("Get windows registry key info failed: %+v", err)
			s.Key.Close()
			return
		}
	}
}

func (s *Shell) broadcast() {
	ee, _ := syscall.UTF16PtrFromString(EnvironmentName)
	r, _, err := syscall.NewLazyDLL("user32.dll").NewProc("SendMessageTimeoutW").Call(
		0xffff, // HWND_BROADCAST
		0x1a,   // WM_SETTINGCHANGE
		0,
		uintptr(unsafe.Pointer(ee)),
		0x02, // SMTO_ABORTIFHUNG
		5000, // 5 seconds
		0,
	)
	if r == 0 {
		gprint.PrintError("Broadcast env changes failed: %+v", err)
	}
}

func (s *Shell) cdHook() {
	/*
		powershell config file path:
		https://learn.microsoft.com/zh-cn/powershell/module/microsoft.powershell.core/about/about_profiles?view=powershell-7.4
		https://www.jb51.net/article/53412.htm

		set-alias:
		https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/set-alias?view=powershell-7.4
	*/
	psConfPath := s.ConfPath()

	var content string
	if ok, _ := gutils.PathIsExist(psConfPath); ok {
		data, _ := os.ReadFile(psConfPath)
		content = strings.TrimSpace(string(data))
	}

	if strings.Contains(content, PowershellHook) {
		return
	}

	if strings.Contains(content, oldPwsHook) {
		content = strings.ReplaceAll(content, oldPwsHook, PowershellHook)
	} else if content != "" {
		content = fmt.Sprintf("%s\n%s", PowershellHook, content)
	} else {
		content = PowershellHook
	}

	err := os.WriteFile(psConfPath, []byte(content), os.ModePerm)
	if err != nil {
		_ = os.WriteFile("vmr_error.txt", []byte(err.Error()), os.ModePerm)
	}
}

func (s *Shell) ConfPath() string {
	homeDir, _ := os.UserHomeDir()
	psConfDir := filepath.Join(homeDir,
		"Documents",
		"WindowsPowerShell",
	)
	psConfName := "profile.ps1"

	if ok, _ := gutils.PathIsExist(psConfDir); !ok {
		_ = os.MkdirAll(psConfDir, os.ModePerm)
	}

	psConfPath := filepath.Join(psConfDir, psConfName)
	return psConfPath
}

func (s *Shell) VMEnvConfPath() string {
	return ""
}

func (s *Shell) WriteVMEnvToShell() {
	s.cdHook()
}

func (s *Shell) SetEnv(key, value string) {
	if s.KeyInfo == nil {
		gprint.PrintError("Windows registry key is closed.")
		return
	}

	if key == PathEnvName {
		return
	}
	err := s.Key.SetStringValue(key, value)
	if err != nil {
		gprint.PrintError("Set env '%s=%s' failed: %+v", key, value, err)
		return
	}
	s.broadcast()
}

func (s *Shell) UnsetEnv(key string) {
	if s.KeyInfo == nil {
		gprint.PrintError("Windows registry key is closed.")
		return
	}

	if key == PathEnvName {
		return
	}
	err := s.Key.DeleteValue(key)
	if err != nil {
		gprint.PrintError("Unset env '%s' failed: %+v", key, err)
		return
	}
	s.broadcast()
}

func (s *Shell) SetPath(path string) {
	if s.KeyInfo == nil {
		gprint.PrintError("Windows registry key is closed.")
		return
	}
	path = TidyWindowsPathEnv(path)

	oldPathValue, _, err := s.Key.GetStringValue(PathEnvName)
	if err != nil {
		gprint.PrintError("Get env $path failed: %+v", err)
		return
	}
	if !strings.Contains(oldPathValue, path) {
		newPathValue := path + ";" + oldPathValue
		err := s.Key.SetStringValue(PathEnvName, newPathValue)
		if err != nil {
			gprint.PrintError("Set env $path failed: %s, %+v", path, err)
			return
		}
	}
	s.broadcast()
}

func (s *Shell) UnsetPath(path string) {
	if s.KeyInfo == nil {
		gprint.PrintError("Windows registry key is closed.")
		return
	}
	path = TidyWindowsPathEnv(path)

	oldPathValue, _, err := s.Key.GetStringValue(PathEnvName)
	if err != nil {
		gprint.PrintError("Get env $path failed: %+v", err)
		return
	}
	if strings.Contains(oldPathValue, path) {
		newPathValue := strings.ReplaceAll(strings.ReplaceAll(oldPathValue, path, ""), ";;", ";")
		err := s.Key.SetStringValue(PathEnvName, newPathValue)
		if err != nil {
			gprint.PrintError("Unset env $path failed: %s, %+v", path, err)
			return
		}
	}
	s.broadcast()
}

func (s *Shell) Close() {
	if s.KeyInfo != nil {
		s.KeyInfo = nil
		s.Key.Close()
	}
}
