package cmds

import (
	"encoding/json"
	"net/url"
	"time"

	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

type SDKSha struct {
	Sha      string `json:"sha256"`
	HomePage string `json:"homepage"`
}

type SDKNameList map[string]SDKSha

func ShowSDKNameList() {
	fetcher := request.NewFetcher()
	dUrl, _ := url.JoinPath(cnf.HostUrl, cnf.SDKNameListFileUrl)
	dUrl = cnf.ReverseProxy + dUrl
	fetcher.SetUrl(dUrl)
	fetcher.Timeout = 10 * time.Second
	resp, _ := fetcher.GetString()
	sList := SDKNameList{}
	json.Unmarshal([]byte(resp), &sList)
	l := table.NewList()
	_, w, _ := terminal.GetTerminalSize()
	l.SetHeader([]table.Column{
		{Title: "sdkname", Width: 20},
		{Title: "homepage", Width: w - 30},
	})
	rows := []table.Row{}
	for k, v := range sList {
		rows = append(rows, table.Row{
			k,
			v.HomePage,
		})
	}
	l.SetRows(rows)
	l.Run()
}
