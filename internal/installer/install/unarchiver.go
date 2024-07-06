package install

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/utils"
)

/*
Install use unarchiver.
*/
type ArchiverInstaller struct {
	OriginSDKName string
	SDKName       string
	VersionName   string
	Version       download.Item
	installConf   download.InstallerConfig
	dirFinder     *utils.HomeDirFinder
}

func NewArchiverInstaller() (a *ArchiverInstaller) {
	a = &ArchiverInstaller{
		dirFinder: utils.NewFinder(),
	}
	return
}

func (a *ArchiverInstaller) SetInstallConf(iconf download.InstallerConfig) {
	a.installConf = iconf
}

func (a *ArchiverInstaller) Initiate(originSDKName, versionName string, version download.Item) {
	a.OriginSDKName = originSDKName
	a.VersionName = versionName
	a.Version = version
	a.FormatSDKName()
}

func (a *ArchiverInstaller) FormatSDKName() {
	a.SDKName = a.OriginSDKName
}

func (a *ArchiverInstaller) GetInstallDir() string {
	d := GetSDKVersionDir(a.SDKName)
	return filepath.Join(d, fmt.Sprintf(VersionInstallDirPattern, a.OriginSDKName, a.VersionName))
}

func (a *ArchiverInstaller) GetSymbolLinkPath() string {
	d := GetSDKVersionDir(a.SDKName)
	return filepath.Join(d, a.SDKName)
}

func (a *ArchiverInstaller) prepareDirFinder() {
	a.dirFinder.Clear()
	ff := a.installConf.FlagFiles
	if ff == nil {
		return
	}
	flags := []string{}
	switch runtime.GOOS {
	case gutils.Darwin:
		flags = ff.MacOS
	case gutils.Linux:
		flags = ff.Linux
	case gutils.Windows:
		flags = ff.Windows
	default:
	}
	a.dirFinder.SetFlags(flags...)
	a.dirFinder.SetFlagDirExcepted(a.installConf.FlagDirExcepted)
}

func (a *ArchiverInstaller) handleArchivedFile(fPath string) (newPath string) {
	if strings.Contains(fPath, "git") && strings.HasSuffix(fPath, ".7z.exe") {
		newPath = strings.TrimSuffix(fPath, ".exe")
		os.Rename(fPath, newPath)
		return newPath
	}
	return fPath
}

func (a *ArchiverInstaller) Install() {
	if a.Version.Url == "" {
		return
	}

	// Download archived files.
	dd := download.NewDownloader()
	fPath := dd.Download(a.OriginSDKName, a.VersionName, a.Version)
	if ok, _ := gutils.PathIsExist(fPath); !ok || fPath == "" {
		return
	}

	fPath = a.handleArchivedFile(fPath)

	// Decompress archived file to temp dir.
	tempDir := cnf.GetTempDir()
	if err := utils.Extract(fPath, tempDir); err != nil {
		gprint.PrintError("Extract file failed: %+v.", err)
		os.RemoveAll(filepath.Dir(fPath))
		return
	}

	a.patchFileName()
	// find dir to copy
	a.prepareDirFinder()
	a.dirFinder.Find(tempDir)
	dirToCopy := a.dirFinder.GetDirName()
	if dirToCopy == "" {
		gprint.PrintError("Can't find dir to copy.")
		os.RemoveAll(filepath.Dir(fPath))
		return
	}

	// Copy extracted files to install directory.
	installDir := a.GetInstallDir()
	defer func() {
		os.RemoveAll(tempDir)
	}()
	if err := gutils.CopyDirectory(dirToCopy, installDir, true); err != nil {
		gprint.PrintError("Copy directory failed: %+v.", err)
		os.RemoveAll(filepath.Dir(fPath))
		return
	}
}

// patches file name for some tools.
func (a *ArchiverInstaller) patchFileName() {
	tempDir := cnf.GetTempDir()
	dList, _ := os.ReadDir(tempDir)
	newName := a.SDKName
	if runtime.GOOS == gutils.Windows {
		newName += ".exe"
	}
	if len(dList) == 1 {
		dd := dList[0]
		if !dd.IsDir() && strings.Contains(dd.Name(), a.SDKName) && dd.Name() != newName {
			oldPath := filepath.Join(tempDir, dd.Name())
			newPath := filepath.Join(tempDir, newName)
			os.Rename(oldPath, newPath)
		}
		if runtime.GOOS != gutils.Windows {
			gutils.ExecuteSysCommand(true, tempDir, "chmod", "+x", newName)
		}
	}
}
