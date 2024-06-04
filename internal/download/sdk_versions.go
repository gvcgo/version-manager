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

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/tui/table"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	Conda      string = "conda"
	CondaForge string = "conda-forge"
	Coursier   string = "coursier"
	Unarchiver string = "unarchiver"
	Executable string = "executable"
	Dpkg       string = "dpkg"
	Rpm        string = "rpm"
)

type Item struct {
	Url       string `json:"url"`       // download url
	Arch      string `json:"arch"`      // amd64 | arm64
	Os        string `json:"os"`        // linux | darwin | windows
	Sum       string `json:"sum"`       // Checksum
	SumType   string `json:"sum_type"`  // sha1 | sha256 | sha512 | md5
	Size      int64  `json:"size"`      // Size in bytes
	Installer string `json:"installer"` // conda | conda-forge | coursier | unarchiver | executable | dpkg | rpm
	LTS       string `json:"lts"`       // Long Term Support
	Extra     string `json:"extra"`     // Extra Info
}

type SDKVersion []Item

type VersionList map[string]SDKVersion

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

func GetVersionList(sdkName, newSha256 string) (filteredVersions map[string]Item) {
	dUrl := cnf.GetVersionFileUrlBySDKName(sdkName)
	fetcher := cnf.GetFetcher(dUrl)
	fetcher.Timeout = time.Minute

	var content []byte
	if ok, localFile := CheckSumForVersionFile(sdkName, newSha256); ok {
		content, _ = os.ReadFile(localFile)
	} else {
		resp, _ := fetcher.GetString()
		content = []byte(resp)
		// cache version files.
		os.WriteFile(localFile, content, os.ModePerm)
	}

	rawVersionList := make(VersionList)
	filteredVersions = make(map[string]Item)
	json.Unmarshal(content, &rawVersionList)
	for vName, vList := range rawVersionList {
	INNER:
		for _, item := range vList {
			if item.Os == "unix" && (runtime.GOOS != gutils.Windows) {
				item.Os = runtime.GOOS
			}
			if (item.Os == runtime.GOOS || item.Os == "any") && (item.Arch == runtime.GOARCH || item.Arch == "any") {
				// save filtered version.
				item.Os = runtime.GOOS
				item.Arch = runtime.GOARCH

				if sdkName == "vscode" && item.Os == gutils.Darwin {
					item.Installer = Executable
				}
				if sdkName == "kubectl" {
					item.Installer = Executable
				}

				// filter: php from github
				if sdkName == "php" && item.Url != "" && item.Os != gutils.Windows {
					continue INNER
				}
				if FilterVersionItem(item) {
					filteredVersions[vName] = item
				}
			}
		}
	}
	return
}

func FilterVersionItem(item Item) (ok bool) {
	if item.Os == gutils.Linux && (item.Installer == Dpkg || item.Installer == Rpm) {
		switch utils.DNForAPTonLinux() {
		case utils.LinuxInstallerApt:
			return strings.HasSuffix(item.Url, ".deb")
		case utils.LinuxInstallerYum, utils.LinuxInstallerDnf:
			return strings.HasSuffix(item.Url, ".rpm")
		default:
			return false
		}
	}
	return true
}

func GetVersionsSortedRows(filteredVersions map[string]Item) (rows []table.Row) {
	for vName, vItem := range filteredVersions {
		if vItem.LTS != "" {
			vName += "-lts"
		}
		rows = append(rows, table.Row{
			vName,
			vItem.Installer,
		})
	}
	SortVersions(rows)
	return
}

func getLatestVersion(sdkName, newSha256 string) (vName string, version Item, ok bool) {
	fvs := GetVersionList(sdkName, newSha256)
	if len(fvs) == 0 {
		return
	}
	rows := GetVersionsSortedRows(fvs)
	vName = strings.TrimSuffix(rows[0][0], "-lts")
	version = fvs[vName]
	ok = true
	return
}

func GetLatestVersionBySDKName(sdkName string) (vName string, vItem Item) {
	sdkList := GetSDKList()
	if sdkInfo, ok := sdkList[sdkName]; ok {
		newSha256 := sdkInfo.Sha256
		if versionName, versionItem, ok1 := getLatestVersion(sdkName, newSha256); ok1 {
			vName = versionName
			vItem = versionItem
		}
	}
	return
}
