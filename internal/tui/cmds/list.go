package cmds

import (
	"encoding/json"
	"net/url"
	"time"

	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

type SDKNameList map[string]string

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
	l.SetHeader([]table.Column{
		{Title: "sdkname", Width: 30},
		{Title: "homepage", Width: 180},
	})
	rows := []table.Row{}
	for k, v := range sList {
		rows = append(rows, table.Row{
			k,
			v,
		})
	}
	l.SetRows(rows)
	l.Run()
}
