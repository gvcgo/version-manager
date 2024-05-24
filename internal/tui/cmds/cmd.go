package cmds

type VmrTUI struct {
	SList *VMRSDKList
	VList *SDKVersionList
}

func NewTUI() *VmrTUI {
	return &VmrTUI{}
}

func (v *VmrTUI) ListSDKName() {
	if v.SList == nil {
		v.SList = NewVMRSDKList()
	}
	lastPressedKey, sdkName := v.SList.ShowSDKList()

	// search version list for selected sdkname.
	if lastPressedKey == "s" {
		v.SearchVersions(sdkName)
	}
}

func (v *VmrTUI) SearchVersions(sdkName string) {
	if v.VList == nil {
		v.VList = NewSDKVersionList()
	}
	v.VList.Search(sdkName)
}
