package cmds

import (
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/installer"
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
	nextEvent, sdkName := v.SList.Show()

	switch nextEvent {
	case KeyEventSeachVersionList:
		// search version list for selected sdkname.
		v.SearchVersions(sdkName, v.SList.GetSDKItemByName(sdkName))
	case KeyEventShowLocalInstalled:
		// show local installed versions for selected sdkname.
		v.ShowLocalInstalled(sdkName)
	case KeyEventClearLocalCached:
		// clear all cached files for selected sdkname.
		v.ClearLocalCachedFiles(sdkName, "")
	case KeyEventRemoveLocalInstalled:
		// remove all installed versions for selected sdkname.
		v.RemoveInstalledVersions(sdkName)
	default:
	}
}

func (v *VmrTUI) SearchVersions(sdkName string, sdkItem download.SDK) {
	if v.VList == nil {
		v.VList = NewVersionSearcher()
	}
	lastPressedKy, versionName := v.VList.Search(sdkName, sdkItem.Sha256)

	switch lastPressedKy {
	case KeyEventBacktoPreviousPage:
		v.ListSDKName()
	case KeyEventUseVersionGlobally:
		vItem := v.VList.GetVersionByVersionName(versionName)
		ins := installer.NewInstaller(sdkName, versionName, sdkItem.InstallConfSha256, vItem)
		ins.SetInvokeMode(installer.ModeGlobally)
		ins.Install()
	case KeyEventUseVersionSessionly:
		vItem := v.VList.GetVersionByVersionName(versionName)
		ins := installer.NewInstaller(sdkName, versionName, sdkItem.InstallConfSha256, vItem)
		ins.SetInvokeMode(installer.ModeSessionly)
		ins.Install()
	case KeyEventLockVersion:
		vItem := v.VList.GetVersionByVersionName(versionName)
		ins := installer.NewInstaller(sdkName, versionName, sdkItem.InstallConfSha256, vItem)
		ins.SetInvokeMode(installer.ModeToLock)
		ins.Install()
	}
}

func (v *VmrTUI) ShowLocalInstalled(sdkName string) {
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
	ins := installer.NewInstaller(sdkName, versionName, "", download.Item{})
	ins.Uninstall()
}
