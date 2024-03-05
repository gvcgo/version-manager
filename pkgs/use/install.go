package use

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/search"
	"github.com/gvcgo/version-manager/pkgs/versions"
)

type Installer struct {
	AppName  string
	Version  string
	fetcher  *request.Fetcher
	searcher *search.Searcher
	v        *versions.VersionItem
}

func NewInstaller(appName, version string) *Installer {
	return &Installer{
		AppName:  appName,
		Version:  version,
		fetcher:  conf.GetFetcher(),
		searcher: search.NewSearcher(),
	}
}

func (i *Installer) searchVersion() {
	vf := i.searcher.GetVersions(i.AppName)

	vs := make([]string, 0)
	for key := range vf {
		if strings.Contains(key, i.Version) {
			vs = append(vs, key)
		}
	}

	if len(vs) == 0 {
		gprint.PrintError("Cannot find version: %s", i.Version)
	} else if len(vs) == 1 {
		i.v = &vf[vs[0]][0]
	} else {
		gprint.PrintError("Found multiple versions: \n%v", strings.Join(vs, "\n"))
	}
}

// Download zip file.
func (i *Installer) Download() (zipFilePath string) {
	i.searchVersion()
	if i.v == nil {
		return
	}
	zipDir := conf.GetZipFileDir()
	if ok, _ := gutils.PathIsExist(zipDir); !ok {
		if err := os.MkdirAll(zipDir, os.ModePerm); err != nil {
			gprint.PrintError("Failed to create directory: %s", zipDir)
			return
		}
	}
	f := conf.GetFetcher()
	zipFilePath = filepath.Join(zipDir, filepath.Base(i.v.Url))
	f.GetAndSaveFile(zipFilePath)
	return
}
