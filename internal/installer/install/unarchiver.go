package install

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"

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
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf(VerisonDirPattern, a.SDKName))
	os.MkdirAll(d, os.ModePerm)
	return filepath.Join(d, fmt.Sprintf(VersionInstallDirPattern, a.OriginSDKName, a.VersionName))
}

func (a *ArchiverInstaller) GetSymbolLinkPath() string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf(VerisonDirPattern, a.SDKName))
	os.MkdirAll(d, os.ModePerm)
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

	// Decompress archived file to temp dir.
	tempDir := cnf.GetTempDir()
	if err := utils.Extract(fPath, tempDir); err != nil {
		gprint.PrintError("Extract file failed: %+v.", err)
		os.RemoveAll(fPath)
		return
	}

	// find dir to copy
	a.prepareDirFinder()
	a.dirFinder.Find(tempDir)
	dirToCopy := a.dirFinder.GetDirName()
	if dirToCopy == "" {
		gprint.PrintError("Can't find dir to copy.")
		return
	}

	// Copy extracted files to install directory.
	installDir := a.GetInstallDir()
	defer func() {
		os.RemoveAll(tempDir)
	}()
	if err := gutils.CopyDirectory(dirToCopy, installDir, true); err != nil {
		gprint.PrintError("Copy directory failed: %+v.", err)
		return
	}
}
