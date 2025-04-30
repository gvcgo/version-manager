package installer

import (
	"encoding/json"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/installer/install"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/shell/sh"
	"github.com/gvcgo/version-manager/internal/terminal"
)

/*
Lock the version of an SDK for a project.
*/
const (
	LockerFileName = ".vmr.lock"
)

/*
Lock the SDK version for a project.
*/
type VersionLocker struct {
	VersionOfSDKs map[string]string
}

func NewVLocker() (v *VersionLocker) {
	return &VersionLocker{VersionOfSDKs: make(map[string]string)}
}

func (v *VersionLocker) FindLockerFile(dirPath ...string) string {
	var dPath string
	if len(dirPath) == 0 {
		dPath, _ = os.Getwd()
	} else {
		dPath = dirPath[0]
	}
	if dPath == filepath.Dir(dPath) {
		return ""
	}
	p := filepath.Join(dPath, LockerFileName)
	if ok, _ := gutils.PathIsExist(p); ok {
		return p
	} else {
		return v.FindLockerFile(filepath.Dir(dPath))
	}
}

func (v *VersionLocker) Load() {
	fPath := v.FindLockerFile()
	if fPath == "" {
		return
	}
	if ok, _ := gutils.PathIsExist(fPath); ok {
		data, _ := os.ReadFile(fPath)
		content := strings.TrimSpace(string(data))
		if content != "" && !strings.Contains(content, "{") {
			sList := strings.Split(content, "@")
			if len(sList) == 2 {
				v.VersionOfSDKs[sList[0]] = sList[1]
			}
		} else {
			json.Unmarshal([]byte(content), &v.VersionOfSDKs)
		}
	}

	// for old .vmr.lock file.
	for key, value := range v.VersionOfSDKs {
		if key == "nodejs" || key == "node.js" {
			v.VersionOfSDKs["node"] = value
		}
	}
}

/*
save lock info.
*/
func (v *VersionLocker) Save(sdkName, versionName string) {
	lockFilePath := v.FindLockerFile()

	var content string

	if lockFilePath != "" {
		// for old .vmr.lock file.
		data, _ := os.ReadFile(lockFilePath)
		content = strings.TrimSpace(string(data))
		if content != "" && !strings.Contains(content, "{") {
			sList := strings.Split(content, "@")
			if len(sList) == 2 {
				v.VersionOfSDKs[sList[0]] = sList[1]
			}
		} else {
			json.Unmarshal([]byte(content), &v.VersionOfSDKs)
		}
	} else {
		cwd, _ := os.Getwd()
		lockFilePath = filepath.Join(cwd, LockerFileName)
	}

	if sdkName != "" && versionName != "" {
		v.VersionOfSDKs[sdkName] = versionName
	}

	data, _ := json.MarshalIndent(&v.VersionOfSDKs, "", "    ")
	_ = os.WriteFile(lockFilePath, data, sh.ModePerm)
}

/*
Hook for cd command.
*/
func (v *VersionLocker) HookForCdCommand() {
	v.Load()
	if len(v.VersionOfSDKs) == 0 {
		os.Exit(0)
	}
	// t := terminal.NewPtyTerminal()
	os.Setenv(AddToPathTemporarillyEnvName, "1")
	for sdkName, versionName := range v.VersionOfSDKs {
		RemoveGlobalSDKPathTemporarily(sdkName)
		ins := NewInstaller(sdkName, versionName, "", lua_global.Item{})
		// follow the order.
		ins.AddEnvsTemporarilly()
	}
	// t.Run()
	terminal.RunTerminal()
}

/*
Remove the global SDK path from envs.
Otherwise, it will shadow the hooked ones.
*/
func RemoveGlobalSDKPathTemporarily(sdkName string) {
	pathStr := os.Getenv("PATH")
	symbolicPath := filepath.Join(install.GetSDKVersionDir(sdkName), sdkName)
	sep := ":"
	if runtime.GOOS == gutils.Windows {
		sep = ";"
	}
	eList := []string{}
	for _, pStr := range strings.Split(pathStr, sep) {
		if strings.HasPrefix(pStr, symbolicPath) {
			continue
		}
		eList = append(eList, pStr)
	}
	os.Setenv("PATH", strings.Join(eList, sep))
}
