package cmds

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
		sdkSha := v.SList.GetShaBySDKName(sdkName)
		v.SearchVersions(sdkName, sdkSha.Sha)
	}
}

func (v *VmrTUI) SearchVersions(sdkName, sha256 string) {
	if v.VList == nil {
		v.VList = NewVersionSearcher()
	}
	v.VList.Search(sdkName, sha256)
}
