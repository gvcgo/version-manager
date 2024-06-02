package download

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/tui/table"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	SDKListFileName string = "sdk_list.json"
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
	dUrl := cnf.GetSDKListFileUrl()

	fPath := filepath.Join(cnf.GetCacheDir(), SDKListFileName)
	lastModifiedTime := utils.GetFileLastModifiedTime(fPath)
	timelag := time.Now().Unix() - lastModifiedTime

	content, _ := os.ReadFile(fPath)
	if timelag > 1800 || len(content) < 20 {
		// over half an hour, then download again.
		fetcher := cnf.GetFetcher(dUrl)
		fetcher.Timeout = 10 * time.Second
		resp, _ := fetcher.GetString()
		os.WriteFile(fPath, []byte(resp), os.ModePerm)
		json.Unmarshal([]byte(resp), &ss)
	} else {
		// otherwise, read from the cached file.
		json.Unmarshal(content, &ss)
	}
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
