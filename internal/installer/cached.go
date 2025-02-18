package installer

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/version-manager/internal/cnf"
)

/*
Handle cached files.
*/
type CachedFileFinder struct {
	PluginName  string
	VersionName string
}

func NewCachedFileFinder(pluginName string, versionName ...string) *CachedFileFinder {
	vName := ""
	if len(versionName) > 0 {
		vName = versionName[0]
	}
	return &CachedFileFinder{
		PluginName:  pluginName,
		VersionName: vName,
	}
}

func (cf *CachedFileFinder) Delete() {
	cacheDir := cnf.GetCacheDir()

	if cf.VersionName == "" {
		cachedSDKName := filepath.Join(cacheDir, cf.PluginName)
		dList, _ := os.ReadDir(cachedSDKName)
		for _, d := range dList {
			if d.IsDir() {
				os.RemoveAll(filepath.Join(cachedSDKName, d.Name()))
			}
		}
	} else {
		dd := filepath.Join(cacheDir, cf.PluginName, cf.VersionName)
		dd = strings.TrimSuffix(dd, "<current>")
		os.RemoveAll(dd)
	}
}
