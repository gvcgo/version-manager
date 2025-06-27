package cinstaller

import (
	"path/filepath"
	"strings"

	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
)

type VersionItem struct {
	Version string
	lua_global.Item
}

/*
Installs an SDK.
*/
type Installer struct {
	plugin *plugin.Plugin
	vl     lua_global.VersionList
}

func New(p *plugin.Plugin) *Installer {
	return &Installer{
		plugin: p,
	}
}

func (i *Installer) GetVersionList() (lua_global.VersionList, error) {
	if i.vl != nil {
		return i.vl, nil
	}
	var err error
	i.vl, err = i.plugin.GetSDKVersions()
	return i.vl, err
}

func (i *Installer) SearchVersion(version string) *VersionItem {
	vl, _ := i.GetVersionList()
	if vl == nil {
		return nil
	}

	// return the latest version by default
	if version == "" || version == "latest" {
		sortedVl := vl.SortDesc()
		if len(sortedVl) == 0 {
			return nil
		}
		return &VersionItem{
			Version: sortedVl[0],
			Item:    vl[sortedVl[0]],
		}
	}

	sortedVl := vl.SortDesc()
	for _, v := range sortedVl {
		if v == version {
			return &VersionItem{
				Version: v,
				Item:    vl[v],
			}
		} else if strings.HasPrefix(v, version) {
			return &VersionItem{
				Version: v,
				Item:    vl[v],
			}
		}
	}
	return nil
}

func (i *Installer) GetInstallationFilePath(version string) string {
	cacheDir := cnf.GetCacheDir()
	customedFileNameHandler := i.plugin.GetCustomedFileNameHandler(version)
	if customedFileNameHandler != nil {
		filename, err := customedFileNameHandler()
		if err != nil || filename == "" {
			return ""
		}
		return filepath.Join(cacheDir, i.plugin.SDKName, i.plugin.PluginName, version, filename)
	}

	item := i.SearchVersion(version)
	idx := strings.LastIndex(item.Url, "/")
	if idx == -1 {
		return ""
	}
	filename := item.Url[idx+1:]
	return filepath.Join(cacheDir, i.plugin.SDKName, i.plugin.PluginName, version, filename)
}

func (i *Installer) GetInstallationDestDir(version string) string {
	item := i.SearchVersion(version)
	return lua_global.GetInstallDir(i.plugin.SDKName, i.plugin.PluginName, item.Version)
}

func (i *Installer) PreInstall(version string) error {
	if i.plugin == nil {
		return nil
	}
	preInstallHandler := i.plugin.GetPreInstallHandler(version)
	if preInstallHandler != nil {
		_, err := preInstallHandler()
		if err != nil {
			return err
		}
	}
	return nil
}

func (i *Installer) Install(version string) error {
	if i.plugin == nil {
		return nil
	}
	customedInstaller := i.plugin.GetCustomedInstallHandler(version)
	if customedInstaller != nil {
		_, err := customedInstaller()
		if err != nil {
			return err
		}
	}
	// TODO: default install handler.
	return nil
}

func (i *Installer) CreateSymbolicLink(version string) error {
	return nil
}

func (i *Installer) SetEnv(version string) error {
	return nil
}

func (i *Installer) PostInstall(version string) error {
	if i.plugin == nil {
		return nil
	}
	postInstallHandler := i.plugin.GetPostInstallHandler(version)
	if postInstallHandler != nil {
		_, err := postInstallHandler()
		if err != nil {
			return err
		}
	}
	return nil
}

func (i *Installer) InstallVersion(version string) error {
	if i.plugin == nil {
		return nil
	}

	if err := i.PreInstall(version); err != nil {
		return err
	}
	if err := i.Install(version); err != nil {
		return err
	}
	if err := i.PostInstall(version); err != nil {
		return err
	}
	return nil
}

func (i *Installer) Uninstall(version string) error {
	if i.plugin == nil {
		return nil
	}

	uninstall := i.plugin.GetCustomedUninstallHandler(version)
	if uninstall != nil {
		_, err := uninstall()
		if err != nil {
			return err
		}
	}
	//TODO: default uninstall handler
	return nil
}

// Uninstalls all installed versions for an SDK.
func (i *Installer) Clear() error {
	return nil
}

func (i *Installer) UnsetEnv(version string) error {
	return nil
}

// Get all installed versions for an SDK.
func (i *Installer) List() error {
	return nil
}
