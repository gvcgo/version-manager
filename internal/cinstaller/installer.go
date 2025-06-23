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
		if err := i.Install(); err != nil {
			return err
		}
	}
	return nil
}

func (i *Installer) preInstall() error {
	preInstallHandler := i.plugin.GetPreInstallHandler()
	if preInstallHandler != nil {
		return preInstallHandler()
	}
	return nil
}

func (i *Installer) install() error {
	return nil
}

func (i *Installer) postInstall() error {
	return nil
}

func (i *Installer) Install() error {
	if err := i.installPrequisites(); err != nil {
		return err
	}
	if err := i.preInstall(); err != nil {
		return err
	}
	if err := i.install(); err != nil {
		return err
	}
	if err := i.postInstall(); err != nil {
		return err
	}
	return nil
}

func (i *Installer) Uninstall() error {
	return nil
}
