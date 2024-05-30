package installer

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/download"
)

/*
1. Find current version.
2. Find installed versions.
*/

type InstalledVersionFinder struct {
	SDKName           string
	InstalledVersions []string
	CurrentVersion    string
	Installer         *Installer
}

func NewIVFinder(sdkName string) (i *InstalledVersionFinder) {
	i = &InstalledVersionFinder{
		SDKName:   sdkName,
		Installer: NewInstaller(sdkName, "", "", download.Item{}),
	}
	return
}

func (i *InstalledVersionFinder) findCurrentVersion(symbolPath string) {
	if ok, _ := gutils.PathIsExist(symbolPath); !ok {
		return
	}

	slink, _ := os.Readlink(symbolPath)
	fName := filepath.Base(slink)
	namePrefix := fmt.Sprintf("%s-", i.SDKName)
	if strings.HasPrefix(fName, namePrefix) {
		i.CurrentVersion = strings.TrimPrefix(fName, namePrefix)
	}
}

func (i *InstalledVersionFinder) FindAll() (r []string, current string) {
	current = ""
	sdkInstaller := i.Installer.GetSDKInstaller()
	if sdkInstaller == nil {
		return
	}
	symbolPath := sdkInstaller.GetSymbolLinkPath()
	versionDIr := filepath.Dir(symbolPath)
	if ok, _ := gutils.PathIsExist(versionDIr); !ok {
		return
	}
	i.findCurrentVersion(symbolPath)

	namePrefix := fmt.Sprintf("%s-", i.SDKName)
	dList, _ := os.ReadDir(versionDIr)
	for _, d := range dList {
		if d.IsDir() && strings.HasPrefix(d.Name(), namePrefix) {
			i.InstalledVersions = append(i.InstalledVersions, strings.TrimPrefix(d.Name(), namePrefix))
		}
	}
	return i.InstalledVersions, i.CurrentVersion
}
