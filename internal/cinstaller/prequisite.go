package cinstaller

import (
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
)

/*
Check and install Miniconda, Coursior, Rustup, etc.
*/
func (i *Installer) CheckPrequisite() *Installer {
	latest := i.SearchVersion("latest")
	if latest == nil {
		return nil
	}

	plugins := plugin.NewPlugins()
	switch latest.Installer {
	case lua_global.InstallerConda:
		p := plugins.GetPlugin("miniconda")
		return New(&p)
	case lua_global.InstallerCoursier:
		p := plugins.GetPlugin("coursier")
		return New(&p)
	case lua_global.InstallerRustup:
		p := plugins.GetPlugin("rustup")
		return New(&p)
	default:
	}

	return nil
}
