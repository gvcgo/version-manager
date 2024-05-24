package cmds

import (
	"encoding/json"
	"time"

	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

/*
Show the SDK list supported by vmr.
*/

type SDKSha struct {
	Sha      string `json:"sha256"`
	HomePage string `json:"homepage"`
}

type SDKNameList map[string]SDKSha

type VMRSDKList struct {
	SdkList SDKNameList
	Fetcher *request.Fetcher
}

func NewVMRSDKList() *VMRSDKList {
	return &VMRSDKList{
		SdkList: make(SDKNameList),
		Fetcher: request.NewFetcher(),
	}
}

func (v *VMRSDKList) ShowSDKList() {
	dUrl := cnf.GetSDKListFileUrl()
	v.Fetcher.SetUrl(dUrl)
	v.Fetcher.Timeout = 10 * time.Second

	resp, _ := v.Fetcher.GetString()
	sdkList := make(SDKNameList)
	json.Unmarshal([]byte(resp), &sdkList)

	ll := table.NewList()
	_, w, _ := terminal.GetTerminalSize()
	ll.SetHeader([]table.Column{
		{Title: "sdkname", Width: 20},
		{Title: "homepage", Width: w - 30},
	})
	rows := []table.Row{}
	for k, v := range sdkList {
		rows = append(rows, table.Row{
			k,
			v.HomePage,
		})
	}
	SortVersionAscend(rows)
	ll.SetRows(rows)
	ll.Run()
}
