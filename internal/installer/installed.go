package installer

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
)

/*
1. Find current version.
2. Find installed versions.
*/

type InstalledVersionFinder struct {
	PluginName        string
	SDKName           string
	InstalledVersions []string
	CurrentVersion    string
	Installer         *Installer
}

func NewIVFinder(sdkName string) (i *InstalledVersionFinder) {
	pls := plugin.NewPlugins()
	pls.LoadAll()
	p := pls.GetPluginBySDKName(sdkName)

	versions := plugin.NewVersions(p.PluginName)
	if versions == nil {
		return nil
	}

	_, vItem := versions.GetLatestVersion()

	i = &InstalledVersionFinder{
		PluginName: p.PluginName,
		SDKName:    sdkName,
		Installer:  NewInstaller(sdkName, p.PluginName, "", lua_global.Item{Installer: vItem.Installer}),
	}
	return
}

func (i *InstalledVersionFinder) findCurrentVersion(symbolPath string) {
	if ok, _ := gutils.PathIsExist(symbolPath); !ok {
		return
	}

	slink, _ := os.Readlink(symbolPath)
	fName := filepath.Base(slink)
	namePrefix := fmt.Sprintf("%s-", i.PluginName)
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

	namePrefix := fmt.Sprintf("%s-", i.PluginName)
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
