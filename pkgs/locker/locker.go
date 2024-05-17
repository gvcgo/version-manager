package locker

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/shell/sh"
)

const (
	LockerFileName = ".vmr.lock"
)

/*
TODO: lock versions for multi SDKs in one project.
Lock the sdk version for a project.
*/
type VersionLocker struct {
	versionInfo   string
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
		v.versionInfo = strings.TrimSpace(string(data))
		if v.versionInfo != "" && !strings.Contains(v.versionInfo, "{") {
			sList := strings.Split(v.versionInfo, "@")
			if len(sList) == 2 {
				v.VersionOfSDKs[sList[0]] = sList[1]
			}
		} else {
			json.Unmarshal([]byte(v.versionInfo), &v.VersionOfSDKs)
		}
	}
}

func (v *VersionLocker) Save(vInfo string) {
	lockFilePath := v.FindLockerFile()

	var content string
	if lockFilePath != "" {
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

	if sList := strings.Split(vInfo, "@"); len(sList) == 2 {
		v.VersionOfSDKs[sList[0]] = sList[1]
	}

	data, _ := json.MarshalIndent(&v.VersionOfSDKs, "", "    ")
	_ = os.WriteFile(lockFilePath, data, sh.ModePerm)
}

func (v *VersionLocker) Get() (vInfo string) {
	v.Load()
	return v.versionInfo
}

// TODO: Open a new session.
