package download

import (
	"encoding/json"
	"strings"
	"time"

	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

/*
Download SDK List file.
*/
type SDK struct {
	Sha256            string `json:"sha256"`
	HomePage          string `json:"homepage"`
	InstallConfSha256 string `json:"install_conf_sha256"`
}

type SDKList map[string]SDK

func GetSDKList() (ss SDKList) {
	ss = make(SDKList)
	ff := request.NewFetcher()

	dUrl := cnf.GetSDKListFileUrl()
	ff.SetUrl(dUrl)
	ff.Timeout = 10 * time.Second

	resp, _ := ff.GetString()
	json.Unmarshal([]byte(resp), &ss)
	return
}

func GetSDKSortedRows(ss SDKList) (rows []table.Row) {
	for k, v := range ss {
		if strings.Contains(k, "conda-forge-pkgs") {
			continue
		}
		rows = append(rows, table.Row{
			k,
			v.HomePage,
		})
	}
	SortVersionAscend(rows)
	return
}
