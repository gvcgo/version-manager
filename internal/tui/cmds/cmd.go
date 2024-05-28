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
	if v.SList == nil {
		v.SList = NewSDKSearcher()
	}
	lastPressedKey, sdkName := v.SList.Show()

	// search version list for selected sdkname.
	if lastPressedKey == KeyEventSeachVersionList {
		v.SearchVersions(sdkName, v.SList.GetSDKItemByName(sdkName))
	}
}

func (v *VmrTUI) SearchVersions(sdkName string, sdkItem download.SDK) {
	if v.VList == nil {
		v.VList = NewVersionSearcher()
	}
	lastPressedKy, versionName := v.VList.Search(sdkName, sdkItem.Sha256)

	if lastPressedKy == KeyEventInstallGlobally {
		vItem := v.VList.GetVersionByVersionName(versionName)
		ins := installer.NewInstaller(sdkName, versionName, sdkItem.InstallConfSha256, vItem)
		ins.Install()
	}
}
