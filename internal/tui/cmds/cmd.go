package cmds

var TUIContinueToNext bool

type VmrTUI struct {
	SList *VMRSDKList
	VList *SDKVersionList
}

func NewTUI() *VmrTUI {
	return &VmrTUI{}
}

func (v *VmrTUI) ListSDKName() {
	TUIContinueToNext = false
	if v.SList == nil {
		v.SList = NewVMRSDKList()
	}
	v.SList.ShowSDKList()

	if TUIContinueToNext {
		v.SearchVersions(v.SList.GetSelected())
	}
}

func (v *VmrTUI) SearchVersions(sdkName string) {
	TUIContinueToNext = false
	if v.VList == nil {
		v.VList = NewSDKVersionList()
	}
	v.VList.Search(sdkName)
}
