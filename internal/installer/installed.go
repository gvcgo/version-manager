package installer

import (
	"encoding/json"
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
	OriginSDKName     string
	InstalledVersions []string
	CurrentVersion    string
	Installer         *Installer
}

func NewIVFinder(sdkName string) (i *InstalledVersionFinder) {
	versionFilePath := download.GetVersionFilePath(sdkName)
	content, _ := os.ReadFile(versionFilePath)
	rawVersionList := make(download.VersionList)
	json.Unmarshal(content, &rawVersionList)
	installerType := "unarchiver"
	for _, vl := range rawVersionList {
		if len(vl) > 0 {
			installerType = vl[0].Installer
			break
		}
	}
	i = &InstalledVersionFinder{
		OriginSDKName: sdkName,
		Installer:     NewInstaller(sdkName, "", "", download.Item{Installer: installerType}),
	}
	return
}

func (i *InstalledVersionFinder) findCurrentVersion(symbolPath string) {
	if ok, _ := gutils.PathIsExist(symbolPath); !ok {
		return
	}

	slink, _ := os.Readlink(symbolPath)
	fName := filepath.Base(slink)
	namePrefix := fmt.Sprintf("%s-", i.OriginSDKName)
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
	versionDir := filepath.Dir(symbolPath)
	if ok, _ := gutils.PathIsExist(versionDir); !ok {
		return
	}
	i.findCurrentVersion(symbolPath)

	namePrefix := fmt.Sprintf("%s-", i.OriginSDKName)
	dList, _ := os.ReadDir(versionDir)
	for _, d := range dList {
		if d.IsDir() && strings.HasPrefix(d.Name(), namePrefix) {
			i.InstalledVersions = append(i.InstalledVersions, strings.TrimPrefix(d.Name(), namePrefix))
		}
	}
	return i.InstalledVersions, i.CurrentVersion
}

func (i *InstalledVersionFinder) UninstallAllVersions() {
	sdkInstaller := i.Installer.GetSDKInstaller()
	if sdkInstaller == nil {
		return
	}
	versionDir := filepath.Dir(sdkInstaller.GetSymbolLinkPath())
	if ok, _ := gutils.PathIsExist(versionDir); !ok {
		return
	}
	os.RemoveAll(versionDir)
	i.Installer.UnsetEnv()
}
