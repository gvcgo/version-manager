package locker

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
)

const (
	LockerFileName = ".vmr.lock"
)

/*
TODO: lock versions for multi SDKs in one project.
Lock the sdk version for a project.
*/
type VersionLocker struct {
	versionInfo string
}

func NewVLocker() (v *VersionLocker) {
	return &VersionLocker{}
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
		v.versionInfo = string(data)
	}
}

func (v *VersionLocker) Save(vInfo string) {
	if strings.Contains(vInfo, "@") {
		v.versionInfo = vInfo
		fPath := v.FindLockerFile()
		if fPath == "" {
			cwd, _ := os.Getwd()
			fPath = filepath.Join(cwd, LockerFileName)
		}
		os.WriteFile(fPath, []byte(v.versionInfo), os.ModePerm)
	}
}

func (v *VersionLocker) Get() (vInfo string) {
	v.Load()
	return v.versionInfo
}
