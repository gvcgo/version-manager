package download

import (
	"encoding/json"
	"time"

	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
)

/*
Download SDK List file.
*/
type SDK struct {
	Sha256   string `json:"sha256"`
	HomePage string `json:"homepage"`
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
