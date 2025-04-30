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

func (v *VmrTUI) ListPluginName() {
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
		v.ListPluginName()
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

func (v *VmrTUI) ShowLocalInstalled(pluginName string) {
	if v.VList == nil {
		v.VList = NewVersionSearcher()
	}
	li := NewLocalInstalled()
	li.Search(pluginName)
	nextEvent, selectedVersion := li.Show()

	switch nextEvent {
	case KeyEventBacktoPreviousPage:
		v.ListPluginName()
	case KeyEventClearCachedFileForAVersion:
		// clear the cached files for selected version.
		v.ClearLocalCachedFiles(pluginName, selectedVersion)
	case KeyEventRemoveAnInstalledVersion:
		// remove the selected version.
		v.RemoveSelectedVersion(pluginName, selectedVersion)
	case KeyEventLockVersion:
		if v.VList == nil {
			v.VList = NewVersionSearcher()
		}
		vItem := v.VList.GetVersionByVersionName(selectedVersion)
		sdkName := v.VList.GetSDKName(pluginName)
		ins := installer.NewInstaller(sdkName, pluginName, selectedVersion, vItem)
		ins.SetInvokeMode(installer.ModeToLock)
		ins.Install()
	case KeyEventUseVersionGlobally:
		vItem := v.VList.GetVersionByVersionName(selectedVersion)
		sdkName := v.VList.GetSDKName(pluginName)
		ins := installer.NewInstaller(sdkName, pluginName, selectedVersion, vItem)
		ins.SetInvokeMode(installer.ModeGlobally)
		ins.Install()
	case KeyEventUseVersionSessionly:
		vItem := v.VList.GetVersionByVersionName(selectedVersion)
		sdkName := v.VList.GetSDKName(pluginName)
		ins := installer.NewInstaller(sdkName, pluginName, selectedVersion, vItem)
		ins.SetInvokeMode(installer.ModeSessionly)
		ins.Install()
	}
}

func (v *VmrTUI) ClearLocalCachedFiles(pluginName, versionName string) {
	cf := installer.NewCachedFileFinder(pluginName, versionName)
	cf.Delete()
}

func (v *VmrTUI) RemoveInstalledVersions(pluginName string) {
	lif := installer.NewIVFinder(pluginName)
	lif.UninstallAllVersions()
}

func (v *VmrTUI) RemoveSelectedVersion(pluginName, versionName string) {
	versions := plugin.NewVersions(pluginName)
	if versions == nil {
		return
	}
	sdkName := versions.GetSDKName()
	vItem := versions.GetVersionByName(versionName)

	ins := installer.NewInstaller(sdkName, pluginName, versionName, vItem)
	ins.Uninstall()
}
