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
		v.SearchVersions(sdkName)
	}
}

func (v *VmrTUI) SearchVersions(sdkName string) {
	if v.VList == nil {
		v.VList = NewVersionSearcher()
	}
	v.VList.Search(sdkName)
}
