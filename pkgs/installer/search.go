package installer

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/atotto/clipboard"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/gtable"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/utils"
	"github.com/gvcgo/version-manager/pkgs/versions"
)

func PrintVersions(appName string, versionList []string) {
	if IsAppNameSupportedBySDKManager(appName) {
		appName = fmt.Sprintf("android-%s", appName)
	}

	columns := []gtable.Column{
		{Title: gprint.CyanStr(fmt.Sprintf("available %v versions", appName)), Width: 150},
	}

	rows := []gtable.Row{}

	for _, verName := range versionList {
		rows = append(rows, gtable.Row{
			verName,
		})
	}

	t := gtable.NewTable(
		gtable.WithColumns(columns),
		gtable.WithRows(rows),
		gtable.WithFocused(true),
		gtable.WithHeight(25),
		gtable.WithWidth(100),
	)
	t.CopySelectedRow(true)
	t.Run()

	if version, err := clipboard.ReadAll(); err == nil && version != "" {
		// generate use command to clipboard.
		binPath, _ := os.Executable()
		binName := filepath.Base(binPath)
		if binName != "" {
			cmdStr := fmt.Sprintf(`%s use "%s@%s"`, binName, appName, version)
			clipboard.WriteAll(cmdStr)
			fmt.Println("")
			gprint.PrintInfo("Now you can use 'ctrl+v' or 'cmd+v' to install the selected version.")
		}
	}
}

type ISearcher interface {
	GetVersions(appName string) map[string]versions.VersionList
	Search(appName string)
}

type Searcher struct {
	VersionInfo *versions.VersionInfo
}

func NewSearcher() (s *Searcher) {
	s = &Searcher{}
	return
}

func (s *Searcher) init(appName string) {
	s.VersionInfo = versions.NewVInfo(appName)
	s.VersionInfo.RegisterArchHandler(versions.ArchHandlerList[appName])
	s.VersionInfo.RegisterOsHandler(versions.OsHandlerList[appName])
}

// Gets version list.
func (s *Searcher) GetVersions(appName string) map[string]versions.VersionList {
	if s.VersionInfo == nil {
		switch appName {
		case "cmdline-tools":
			s.init("sdkmanager")
		default:
			s.init(appName)
		}
	}
	return s.VersionInfo.GetVersions()
}

// Shows version list.
func (s *Searcher) Search(appName string) {
	if appName == "cmdline-tools" {
		s.init("sdkmanager")
	} else {
		s.init(appName)
	}

	vl := s.VersionInfo.GetSortedVersionList()
	if len(vl) == 0 {
		gprint.PrintWarning("No versions found!")
		return
	}

	PrintVersions(appName, vl)
}

// Checks if sdkmanager has been installed correctly.
func IsAndroidSDKManagerInstalled() (ok bool) {
	uHome, _ := os.UserHomeDir()
	_, err := gutils.ExecuteSysCommand(true, uHome, "sdkmanager", "--list")
	if err != nil {
		gprint.PrintError("%+v", err)
	}
	return err == nil
}

func IsAppNameSupportedBySDKManager(appName string) bool {
	r := false
	switch appName {
	case "cmdline-tools":
		r = true
	case "build-tools":
		r = true
	case "platform-tools":
		r = true
	case "platforms":
		r = true
	case "system-images":
		r = true
	case "ndk":
		r = true
	case "add-ons":
		r = true
	case "extras":
		r = true
	case "sources":
		r = true
	default:
	}
	return r
}

func FixAndroidSDKNames(appName string) (sdkName string) {
	sdkName = appName
	if strings.HasPrefix(appName, "android-") {
		sdkName = strings.TrimPrefix(appName, "android-")
	}
	return
}

/*
Search versions via Android SDKManager.
*/
type SDKManagerSearcher struct {
	currentList map[string]versions.VersionList
}

func NewSDKManagerSearcher() *SDKManagerSearcher {
	if !IsAndroidSDKManagerInstalled() {
		gprint.PrintWarning("please install android commandline-tools first.")
		os.Exit(1)
	}
	return &SDKManagerSearcher{
		currentList: make(map[string]versions.VersionList),
	}
}

func (s *SDKManagerSearcher) getVersionString(installed bool) (r string) {
	sep := "Available Packages:"
	rootDir := GetAndroidHomeDir()
	buff, err := gutils.ExecuteSysCommand(true, rootDir, "sdkmanager", fmt.Sprintf("--sdk_root=%s", rootDir), "--list")
	if err != nil {
		gprint.PrintError("get versions failed: %+v", err)
		os.Exit(1)
	}
	sList := strings.Split(buff.String(), sep)
	if installed {
		return sList[0]
	}
	return sList[1]
}

func (s *SDKManagerSearcher) getVersion(ss []string, appName string) (v string) {
	for _, vv := range ss {
		if strings.Contains(vv, appName) {
			return strings.TrimSpace(vv)
		}
	}
	return
}

func (s *SDKManagerSearcher) GetVersions(appName string) map[string]versions.VersionList {
	if !IsAppNameSupportedBySDKManager(appName) {
		gprint.PrintWarning("unspported sdk name: %s", appName)
		os.Exit(1)
	}
	for _, line := range strings.Split(s.getVersionString(false), "\n") {
		ss := strings.Split(line, " ")
		if strings.Contains(line, appName) {
			vName := s.getVersion(ss, appName)
			if vName == "" {
				continue
			}
			s.currentList[vName] = versions.VersionList{
				{
					Arch: runtime.GOARCH,
					Os:   runtime.GOOS,
				},
			}
		}
	}
	return s.currentList
}

func (s *SDKManagerSearcher) ShowInstalledPackages() {
	r := s.getVersionString(true)
	fmt.Println(r)
}

// Shows version list.
func (s *SDKManagerSearcher) Search(appName string) {
	s.GetVersions(appName)

	if len(s.currentList) == 0 {
		gprint.PrintWarning("no versions found.")
		return
	}

	vl := []string{}
	for vName := range s.currentList {
		vl = append(vl, vName)
	}
	utils.SortVersions(vl)

	PrintVersions(appName, vl)
}
