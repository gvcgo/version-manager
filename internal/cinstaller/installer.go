package cinstaller

import (
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
)

/*
Installs an SDK.
*/
type Installer struct {
	plugin *plugin.Plugin
}

func New(p *plugin.Plugin) *Installer {
	return &Installer{
		plugin: p,
	}
}

func (i *Installer) GetVersionList() (lua_global.VersionList, error) {
	return i.plugin.GetSDKVersions()
}

func (i *Installer) installPrequisites() error {
	if i.plugin != nil && i.plugin.Prequisite != "" {
		plugins := plugin.NewPlugins()
		p := plugins.GetPlugin(i.plugin.Prequisite)
		// install prequisite
		i := New(&p)
		// TODO: multi prequisites and versions.
		if err := i.Install(""); err != nil {
			return err
		}
	}
	return nil
}

func (i *Installer) preInstall(version string) error {
	if i.plugin == nil {
		return nil
	}
	preInstallHandler := i.plugin.GetPreInstallHandler(version)
	if preInstallHandler != nil {
		return preInstallHandler()
	}
	return nil
}

func (i *Installer) install(version string) error {
	if i.plugin == nil {
		return nil
	}
	customedInstaller := i.plugin.GetCustomedInstallHandler(version)
	if customedInstaller != nil {
		return customedInstaller()
	}
	// TODO: default install handler.
	return nil
}

func (i *Installer) postInstall(version string) error {
	if i.plugin == nil {
		return nil
	}
	postInstallHandler := i.plugin.GetPostInstallHandler(version)
	if postInstallHandler != nil {
		return postInstallHandler()
	}
	return nil
}

func (i *Installer) Install(version string) error {
	if err := i.installPrequisites(); err != nil {
		return err
	}
	if err := i.preInstall(version); err != nil {
		return err
	}
	if err := i.install(version); err != nil {
		return err
	}
	if err := i.postInstall(version); err != nil {
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
		return uninstall()
	}
	//TODO: default uninstall handler
	return nil
}
