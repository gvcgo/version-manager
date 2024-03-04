package versions

import (
	"encoding/json"
	"fmt"
	"runtime"
	"time"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/request"
)

const (
	RemoteVersionFilePattern string = "https://raw.githubusercontent.com/gvcgo/resources/main/%s.version.json"
	ReverseProxy             string = "https://gvc.1710717.xyz/proxy/"
)

/*
Apps supported by version manager
*/
var AppList []string = []string{
	"bun", "cygwin", "deno", "fd", "flutter", "fzf", "git",
	"go", "gradle", "gsudo", "jdk", "julia", "kotlin",
	"maven", "miniconda", "msys2", "neovim", "nodejs",
	"php", "python", "ripgrep", "rust", "sdkmanager",
	"typst-lsp", "typst", "v-analyzer", "v", "vscode",
	"zig", "zls",
}

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
	List    map[string]VersionList
	AppName string
	fetcher *request.Fetcher
}

func NewVInfo(appName string) (vi *VersionInfo) {
	vi = &VersionInfo{
		List:    map[string]VersionList{},
		AppName: appName,
		fetcher: request.NewFetcher(),
	}
	return
}

func (v *VersionInfo) Parse() {
	if v.AppName == "" {
		return
	}
	v.fetcher.Timeout = 120 * time.Second
	u := ReverseProxy + fmt.Sprintf(RemoteVersionFilePattern, v.AppName)
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
	r := make(map[string]VersionList)

	if len(v.List) == 0 {
		return r
	}
	for vName, vList := range v.List {
		if _, ok := r[vName]; !ok {
			r[vName] = VersionList{}
		}
		for _, ver := range vList {
			if ver.Arch == runtime.GOARCH && ver.Os == runtime.GOOS {
				r[vName] = append(r[vName], ver)
			}
		}
	}
	return r
}
