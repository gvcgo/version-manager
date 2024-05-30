package installer

import (
	"os"
	"path/filepath"

	"github.com/gvcgo/version-manager/internal/cnf"
)

/*
Handle cached files.
*/
type CachedFileFinder struct {
	SDKName     string
	VersionName string
}

func NewCachedFileFinder(sdkName string, versionName ...string) *CachedFileFinder {
	vName := ""
	if len(versionName) > 0 {
		vName = versionName[0]
	}
	return &CachedFileFinder{
		SDKName:     sdkName,
		VersionName: vName,
	}
}

func (cf *CachedFileFinder) Delete() {
	cacheDir := cnf.GetCacheDir()
	if cf.VersionName == "" {
		cachedSDKName := filepath.Join(cacheDir, cf.SDKName)
		dList, _ := os.ReadDir(cachedSDKName)
		for _, d := range dList {
			if d.IsDir() {
				os.RemoveAll(filepath.Join(cachedSDKName, d.Name()))
			}
		}

	} else {
		dd := filepath.Join(cacheDir, cf.SDKName, cf.VersionName)
		os.RemoveAll(dd)
	}
}
