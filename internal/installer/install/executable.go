package install

import (
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/utils"
)

/*
1. *.exe
2. *.deb
3. *.rpm
4. *.sh (miniconda)
5. unix-like executable
*/
type ExeInstaller struct {
	OriginSDKName string
	SDKName       string
	VersionName   string
	Version       utils.Item
	Fetcher       *request.Fetcher
}
