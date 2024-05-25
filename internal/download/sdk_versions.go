package download

import (
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/utils"
)

/*
Download version list file.
*/
func CheckSumForVersionFile(sdkName, newSha256 string) (ok bool, fPath string) {
	versionFileCacheDir := filepath.Join(cnf.GetCacheDir(), sdkName)
	os.MkdirAll(versionFileCacheDir, os.ModePerm)
	fPath = filepath.Join(versionFileCacheDir, strings.Trim(fmt.Sprintf(cnf.VersionFileUrlPattern, sdkName), "/"))
	content, _ := os.ReadFile(fPath)

	h := sha256.New()
	h.Write(content)
	oldSha256 := fmt.Sprintf("%x", h.Sum(nil))
	return oldSha256 == newSha256, fPath
}

func GetVersionList(sdkName, newSha256 string) (filteredVersions map[string]utils.Item) {
	dUrl := cnf.GetVersionFileUrlBySDKName(sdkName)

	ff := request.NewFetcher()
	ff.SetUrl(dUrl)
	ff.Timeout = time.Minute

	var content []byte
	if ok, localFile := CheckSumForVersionFile(sdkName, newSha256); ok {
		content, _ = os.ReadFile(localFile)
	} else {
		resp, _ := ff.GetString()
		content = []byte(resp)
		// cache version files.
		os.WriteFile(localFile, content, os.ModePerm)
	}

	rawVersionList := make(utils.VersionList)
	filteredVersions = make(map[string]utils.Item)
	json.Unmarshal(content, &rawVersionList)
	for vName, vList := range rawVersionList {
		for _, item := range vList {
			if (item.Os == runtime.GOOS || item.Os == "any") && (item.Arch == runtime.GOARCH || item.Arch == "any") {
				// save filtered version.
				item.Os = runtime.GOOS
				item.Arch = runtime.GOARCH
				filteredVersions[vName] = item
			}
		}
	}
	return
}
