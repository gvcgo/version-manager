//go:build windows

package envs

import (
	"strings"
	"syscall"
	"unsafe"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"golang.org/x/sys/windows/registry"
)

const (
	EnvironmentName string = "Environment"
	PathEnvName     string = "path"
)

type EnvManager struct {
	Key     registry.Key
	KeyInfo *registry.KeyInfo
}

func NewEnvManager() (em *EnvManager) {
	em = &EnvManager{}
	em.getKeyInfo()
	return
}

func (em *EnvManager) getKeyInfo() {
	if em.KeyInfo == nil {
		var err error
		em.Key, err = registry.OpenKey(registry.CURRENT_USER, "Environment", registry.ALL_ACCESS)
		if err != nil {
			gprint.PrintError("Get windows registry key failed: %+v", err)
			return
		}

		em.KeyInfo, err = em.Key.Stat()
		if err != nil {
			gprint.PrintError("Get windows registry key info failed: %+v", err)
			em.Key.Close()
			return
		}
	}
}

func (em *EnvManager) CloseKey() {
	if em.KeyInfo != nil {
		em.KeyInfo = nil
		em.Key.Close()
	}
}

func (em *EnvManager) broadcast() {
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

// Set binary path.
func (em *EnvManager) SetPath() {
	if em.KeyInfo == nil {
		gprint.PrintError("Windows registry key is closed.")
		return
	}
	binDir := conf.GetAppBinDir()
	value, _, err := em.Key.GetStringValue(PathEnvName)
	if err != nil {
		gprint.PrintError("Get env $path failed: %+v", err)
		return
	}
	if !strings.Contains(value, binDir) {
		value = binDir + ";" + value
		err := em.Key.SetStringValue(PathEnvName, value)
		if err != nil {
			gprint.PrintError("Set env $path failed: %s, %+v", binDir, err)
			return
		}
	}
	em.broadcast()
}

func (em *EnvManager) UnsetPath() {
	if em.KeyInfo == nil {
		gprint.PrintError("Windows registry key is closed.")
		return
	}
	binDir := conf.GetAppBinDir()
	value, _, err := em.Key.GetStringValue(PathEnvName)
	if err != nil {
		gprint.PrintError("Get env $path failed: %+v", err)
		return
	}
	if strings.Contains(value, binDir) {
		value = strings.ReplaceAll(strings.ReplaceAll(value, binDir, ""), ";;", ";")
		err := em.Key.SetStringValue(PathEnvName, value)
		if err != nil {
			gprint.PrintError("Unset env $path failed: %s, %+v", binDir, err)
			return
		}
	}
	em.broadcast()
}

// Set envs.
func (em *EnvManager) Set(key, value string) {
	if em.KeyInfo == nil {
		gprint.PrintError("Windows registry key is closed.")
		return
	}

	if key == PathEnvName {
		return
	}
	err := em.Key.SetStringValue(key, value)
	if err != nil {
		gprint.PrintError("Set env '%s=%s' failed: %+v", key, value, err)
		return
	}
	em.broadcast()
}

func (em *EnvManager) UnSet(key string) {
	if em.KeyInfo == nil {
		gprint.PrintError("Windows registry key is closed.")
		return
	}

	if key == PathEnvName {
		return
	}
	err := em.Key.DeleteValue(key)
	if err != nil {
		gprint.PrintError("Unset env '%s' failed: %+v", key, err)
		return
	}
	em.broadcast()
}

func (em *EnvManager) AddToPath(value string) {
	if em.KeyInfo == nil {
		gprint.PrintError("Windows registry key is closed.")
		return
	}

	oldPathValue, _, err := em.Key.GetStringValue(PathEnvName)
	if err != nil {
		gprint.PrintError("Get env $path failed: %+v", err)
		return
	}
	if !strings.Contains(oldPathValue, value) {
		newPathValue := value + ";" + oldPathValue
		err := em.Key.SetStringValue(PathEnvName, newPathValue)
		if err != nil {
			gprint.PrintError("Set env $path failed: %s, %+v", value, err)
			return
		}
	}
	em.broadcast()
}

func (em *EnvManager) DeleteFromPath(value string) {
	if em.KeyInfo == nil {
		gprint.PrintError("Windows registry key is closed.")
		return
	}

	oldPathValue, _, err := em.Key.GetStringValue(PathEnvName)
	if err != nil {
		gprint.PrintError("Get env $path failed: %+v", err)
		return
	}
	if strings.Contains(oldPathValue, value) {
		newPathValue := strings.ReplaceAll(strings.ReplaceAll(oldPathValue, value, ""), ";;", ";")
		err := em.Key.SetStringValue(PathEnvName, newPathValue)
		if err != nil {
			gprint.PrintError("Unset env $path failed: %s, %+v", value, err)
			return
		}
	}
	em.broadcast()
}
