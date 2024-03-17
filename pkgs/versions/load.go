package versions

import (
	"encoding/json"
	"fmt"
	"runtime"
	"strings"
	"time"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/pkgs/conf"
)

const (
	RemoteVersionFilePattern string = "https://raw.githubusercontent.com/gvcgo/resources/main/%s.version.json"
)

type VersionItem struct {
	Url     string `json:"Url"`
	Arch    string `json:"Arch"`
	Os      string `json:"Os"`
	Sum     string `json:"Sum"`
	SumType string `json:"SumType"`
	Extra   string `json:"Extra"`
}

type VersionList []VersionItem

type VersionInfo struct {
	List        map[string]VersionList // full version list
	CurrentList map[string]VersionList // version list for current Arch and Os.
	AppName     string                 // name in AppList
	fetcher     *request.Fetcher
	ArchHandler func(archType, osType string) string
	OsHandler   func(archType, osType string) string
}

func NewVInfo(appName string) (vi *VersionInfo) {
	vi = &VersionInfo{
		List:        map[string]VersionList{},
		CurrentList: map[string]VersionList{},
		AppName:     appName,
		fetcher:     conf.GetFetcher(),
	}
	return
}

func (v *VersionInfo) RegisterArchHandler(f func(archType, osType string) string) {
	v.ArchHandler = f
}

func (v *VersionInfo) RegisterOsHandler(f func(archType, osType string) string) {
	v.OsHandler = f
}

func (v *VersionInfo) Parse() {
	if v.AppName == "" {
		return
	}
	v.fetcher.Timeout = 120 * time.Second
	rawUrl := fmt.Sprintf(RemoteVersionFilePattern, v.AppName)
	u := conf.DecorateUrl(rawUrl)
	/*
		Speedup.
		Example: https://cdn.jsdelivr.net/gh/moqsien/neobox_resources@main/conf.txt
	*/
	if rawUrl == u && conf.UseMirrorSiteInChina() {
		u = fmt.Sprintf("https://cdn.jsdelivr.net/gh/gvcgo/resources@main/%s.version.json", v.AppName)
	}

	v.fetcher.SetUrl(u)
	if s, rCode := v.fetcher.GetString(); rCode == 200 {
		if err := json.Unmarshal([]byte(s), &v.List); err != nil {
			gprint.PrintError("Parse version list failed: %s", err)
		}
	} else {
		gprint.PrintError("Download version list failed: %d", rCode)
	}
}

func (v *VersionInfo) GetVersions() map[string]VersionList {
	if len(v.List) == 0 {
		v.Parse()
	}
	v.CurrentList = map[string]VersionList{}
	if len(v.List) == 0 {
		return v.CurrentList
	}
	for vName, vList := range v.List {
		for _, ver := range vList {
			if v.ArchHandler != nil {
				ver.Arch = v.ArchHandler(ver.Arch, ver.Os)
			}
			if v.OsHandler != nil {
				ver.Os = v.OsHandler(ver.Arch, ver.Os)
			}
			if ver.Arch == runtime.GOARCH && ver.Os == runtime.GOOS {
				if _, ok := v.CurrentList[vName]; !ok {
					v.CurrentList[vName] = VersionList{}
				}
				v.CurrentList[vName] = append(v.CurrentList[vName], ver)
			}
		}
	}
	return v.CurrentList
}

func (v *VersionInfo) sortByVersion() bool {
	l := []string{
		"go",
	}
	return strings.Contains(strings.Join(l, ""), v.AppName)
}

func (v *VersionInfo) GetSortedVersionList() (r []string) {
	v.GetVersions()
	for vName := range v.CurrentList {
		r = append(r, vName)
	}
	if len(r) > 1 {
		if !v.sortByVersion() {
			r = SortStringListDesc(r)
		} else {
			r = SortVersion(r)
		}
	}
	return
}
