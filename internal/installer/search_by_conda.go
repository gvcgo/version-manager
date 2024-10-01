package installer

import (
	"fmt"
	"os"
	"runtime"
	"strings"
	"time"

	"github.com/gvcgo/goutils/pkgs/gtea/spinner"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/download"
)

/*
	var CondaPlatformList = []string{
		"linux-64",
		"linux-aarch64",
		"win-64",
		"win-arm64",
		"osx-64",
		"osx-arm64",
	}
*/
func GetCondaPlatform() (platform string) {
	arch := runtime.GOARCH
	switch runtime.GOOS {
	case "darwin":
		if arch == "arm64" {
			platform = "osx-arm64"
		} else {
			platform = "osx-64"
		}
	case "windows":
		if arch == "amd64" {
			platform = "win-64"
		} else if arch == "arm64" {
			platform = "win-arm64"
		}
	case "linux":
		if arch == "arm64" {
			platform = "linux-aarch64"
		} else if arch == "amd64" {
			platform = "linux-64"
		}
	default:
		os.Exit(1)
	}
	return
}

/*
conda search --override-channels --channel conda-forge --skip-flexible-search --subdir osx-64 --full-name php
*/
var CondaSearchCommand = []string{
	"conda",
	"search",
	"--override-channels",
	"--channel",
	"conda-forge",
	"--skip-flexible-search",
}

/*
search versions by Conda.
*/
type CondaSearcher struct {
	VersionList map[string]download.Item
	SDKName     string
	spinner     *spinner.Spinner
}

func NewCondaSearcher(sdkName string) (c *CondaSearcher) {
	c = &CondaSearcher{
		VersionList: make(map[string]download.Item),
		SDKName:     sdkName,
		spinner:     spinner.NewSpinner(),
	}
	return
}

func (c *CondaSearcher) FindHeader(content string) (header string) {
	lines := strings.Split(content, "\n")
	for _, line := range lines {
		if strings.HasPrefix(line, "# Name") {
			return line
		}
	}
	return
}

func (c *CondaSearcher) FindVersion(llist []string) string {
	newList := []string{}
	for _, item := range llist {
		item = strings.TrimSpace(item)
		if item == "" {
			continue
		}
		newList = append(newList, item)
	}
	if len(newList) > 1 {
		return newList[1]
	}
	return ""
}

func (c *CondaSearcher) ParseSearchResult(content string) (vlist []string) {
	header := c.FindHeader(content)
	if header == "" {
		return
	}
	filter := map[string]struct{}{}
	sList := strings.Split(content, header)
	if len(sList) == 2 {
		lines := strings.Split(sList[1], "\n")
		for _, line := range lines {
			version := c.FindVersion(strings.Split(line, " "))
			if _, ok := filter[version]; !ok {
				filter[version] = struct{}{}
				vlist = append(vlist, version)
			}
		}
	}
	return
}

func (c *CondaSearcher) GetVersions() map[string]download.Item {
	// check miniconda.
	CheckAndInstallMiniconda()

	homeDir, _ := os.UserHomeDir()
	_cmd := append([]string{}, CondaSearchCommand...)
	_cmd = append(_cmd, "--subdir", GetCondaPlatform(), "--full-name", c.SDKName)

	c.spinner.SetTitle(fmt.Sprintf("Conda Searching For %s", c.SDKName))
	go c.spinner.Run()
	r, err := gutils.ExecuteSysCommand(true, homeDir, _cmd...)
	c.spinner.Quit()
	time.Sleep(time.Duration(2) * time.Second)

	if err == nil {
		vlist := c.ParseSearchResult(r.String())
		for _, verName := range vlist {
			c.VersionList[verName] = download.Item{
				Arch:      runtime.GOARCH,
				Os:        runtime.GOOS,
				Installer: download.Conda,
			}
		}
	}
	return c.VersionList
}

func TestCondaSearcher() {
	c := NewCondaSearcher("php")
	versions := c.GetVersions()
	fmt.Printf("%+v\n", versions)
}
