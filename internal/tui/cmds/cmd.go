package cmds

import (
	"github.com/gvcgo/version-manager/internal/installer"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
)

type VmrTUI struct {
	SList *SDKSearcher
	VList *VersionSearcher
}

func NewTUI() *VmrTUI {
	return &VmrTUI{}
}

func (v *VmrTUI) ListSDKName() {
	v.SList = NewSDKSearcher()
	nextEvent, pluginName := v.SList.Show()

	if nextEvent == KeyEventWhatsInstalled {
		// show SDKs already installed by vmr.
		nextEvent, pluginName = v.SList.ShowInstalledOnly()
	}

	switch nextEvent {
	case KeyEventSeachVersionList:
		// search version list for selected sdkname.
		v.SearchVersions(v.SList.GetSDKItemByName(pluginName))
	case KeyEventShowLocalInstalled:
		// show local installed versions for selected sdkname.
		v.ShowLocalInstalled(pluginName)
	case KeyEventClearLocalCached:
		// clear all cached files for selected sdkname.
		v.ClearLocalCachedFiles(pluginName, "")
	case KeyEventRemoveLocalInstalled:
		// remove all installed versions for selected sdkname.
		v.RemoveInstalledVersions(pluginName)
	default:
	}
}

func (v *VmrTUI) SearchVersions(pluginItem plugin.Plugin) {
	if v.VList == nil {
		v.VList = NewVersionSearcher()
	}
	lastPressedKy, versionName := v.VList.Search(pluginItem.PluginName)

	switch lastPressedKy {
	case KeyEventBacktoPreviousPage:
		v.ListSDKName()
	case KeyEventUseVersionGlobally:
		vItem := v.VList.GetVersionByVersionName(versionName)
		ins := installer.NewInstaller(pluginItem.SDKName, pluginItem.PluginName, versionName, vItem)
		ins.SetInvokeMode(installer.ModeGlobally)
		ins.Install()
	case KeyEventUseVersionSessionly:
		vItem := v.VList.GetVersionByVersionName(versionName)
		ins := installer.NewInstaller(pluginItem.SDKName, pluginItem.PluginName, versionName, vItem)
		ins.SetInvokeMode(installer.ModeSessionly)
		ins.Install()
	case KeyEventLockVersion:
		vItem := v.VList.GetVersionByVersionName(versionName)
		ins := installer.NewInstaller(pluginItem.SDKName, pluginItem.PluginName, versionName, vItem)
		ins.SetInvokeMode(installer.ModeToLock)
		ins.Install()
	}
}

func (v *VmrTUI) ShowLocalInstalled(sdkName string) {
	if v.VList == nil {
		v.VList = NewVersionSearcher()
	}
	li := NewLocalInstalled()
	li.Search(sdkName)
	nextEvent, selectedVersion := li.Show()

	switch nextEvent {
	case KeyEventBacktoPreviousPage:
		v.ListSDKName()
	case KeyEventClearCachedFileForAVersion:
		// clear the cached files for selected version.
		v.ClearLocalCachedFiles(sdkName, selectedVersion)
	case KeyEventRemoveAnInstalledVersion:
		// remove the selected version.
		v.RemoveSelectedVersion(sdkName, selectedVersion)
	case KeyEventLockVersion:
		if v.VList == nil {
			v.VList = NewVersionSearcher()
		}
		vItem := v.VList.GetVersionByVersionName(selectedVersion)
		ins := installer.NewInstaller(sdkName, selectedVersion, "", vItem)
		ins.SetInvokeMode(installer.ModeToLock)
		ins.Install()
	case KeyEventUseVersionGlobally:
		vItem := v.VList.GetVersionByVersionName(selectedVersion)
		ins := installer.NewInstaller(sdkName, selectedVersion, "", vItem)
		ins.SetInvokeMode(installer.ModeGlobally)
		ins.Install()
	case KeyEventUseVersionSessionly:
		vItem := v.VList.GetVersionByVersionName(selectedVersion)
		ins := installer.NewInstaller(sdkName, selectedVersion, "", vItem)
		ins.SetInvokeMode(installer.ModeSessionly)
		ins.Install()
	}
}

func (v *VmrTUI) ClearLocalCachedFiles(sdkName, versionName string) {
	cf := installer.NewCachedFileFinder(sdkName, versionName)
	cf.Delete()
}

func (v *VmrTUI) RemoveInstalledVersions(sdkName string) {
	lif := installer.NewIVFinder(sdkName)
	lif.UninstallAllVersions()
}

func (v *VmrTUI) RemoveSelectedVersion(sdkName, versionName string) {
	pls := plugin.NewPlugins()
	pls.LoadAll()
	p := pls.GetPluginBySDKName(sdkName)
	versions := plugin.NewVersions(p.PluginName)
	if versions == nil {
		return
	}
	vItem := versions.GetVersionByName(versionName)

	ins := installer.NewInstaller(sdkName, p.PluginName, versionName, vItem)
	ins.Uninstall()
}
