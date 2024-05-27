package install

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/download"
)

/*
Install use unarchiver.
*/
type ArchiverInstaller struct {
	OriginSDKName string
	SDKName       string
	VersionName   string
	Version       download.Item
}

func NewArchiverInstaller() (a *ArchiverInstaller) {
	a = &ArchiverInstaller{}
	return
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
	d := filepath.Join(versionDir, a.SDKName)
	os.MkdirAll(d, os.ModePerm)
	return filepath.Join(d, a.SDKName)
}

func (a *ArchiverInstaller) Install() {
	// in installer.go
}
