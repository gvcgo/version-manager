package cmds

import (
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

/*
Search version list for SDK.
*/
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

type VersionSearcher struct {
	V       VersionList
	SDKName string
	Fetcher *request.Fetcher
}

func NewVersionSearcher() (sv *VersionSearcher) {
	sv = &VersionSearcher{
		V:       make(VersionList),
		Fetcher: request.NewFetcher(),
	}
	return
}

func (s *VersionSearcher) checkSum(newSha256 string) (ok bool, fPath string) {
	versionFileCacheDir := filepath.Join(cnf.GetCacheDir(), s.SDKName)
	os.MkdirAll(versionFileCacheDir, os.ModePerm)
	fPath = filepath.Join(versionFileCacheDir, strings.Trim(fmt.Sprintf(cnf.VersionFileUrlPattern, s.SDKName), "/"))
	content, _ := os.ReadFile(fPath)

	h := sha256.New()
	h.Write(content)
	oldSha256 := fmt.Sprintf("%x", h.Sum(nil))
	// fmt.Println("oldSha256:", oldSha256)
	// fmt.Println("newSha256:", newSha256)
	return oldSha256 == newSha256, fPath
}

func (s *VersionSearcher) Search(sdkName, newSha256 string) {
	s.SDKName = sdkName
	dUrl := cnf.GetVersionFileUrlBySDKName(s.SDKName)
	s.Fetcher.SetUrl(dUrl)
	s.Fetcher.Timeout = time.Minute

	// compare sha256.
	var content []byte
	if ok, localFile := s.checkSum(newSha256); ok {
		content, _ = os.ReadFile(localFile)
	} else {
		resp, _ := s.Fetcher.GetString()
		content = []byte(resp)
		// cache version files.
		os.WriteFile(localFile, content, os.ModePerm)
	}

	json.Unmarshal(content, &s.V)
	s.Show()
}

func (s *VersionSearcher) Show() (nextEvent, selectedItem string) {
	if len(s.V) == 0 {
		gprint.PrintInfo("No versions found for current platform.")
		return
	}
	ll := table.NewList()
	ll.SetListType(table.SDKList)
	// s.RegisterKeyEvents(ll)

	_, w, _ := terminal.GetTerminalSize()
	if w > 30 {
		w -= 30
	} else {
		w = 120
	}
	ll.SetHeader([]table.Column{
		{Title: s.SDKName, Width: 20},
		{Title: "installer", Width: w},
	})
	rows := []table.Row{}
	for k, v := range s.V {
		for _, item := range v {
			if (item.Os == runtime.GOOS || item.Os == "any") && (item.Arch == runtime.GOARCH || item.Arch == "any") {
				rows = append(rows, table.Row{
					k,
					item.Installer,
				})
			}
		}
	}
	SortVersions(rows)
	ll.SetRows(rows)
	ll.Run()

	selectedItem = ll.GetSelected()
	nextEvent = ll.NextEvent
	return
}

// TODO: install, switch-to, session-only, lock-version
func (s *VersionSearcher) RegisterKeyEvents(ll *table.List) {
}
