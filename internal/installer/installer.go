package installer

import (
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/installer/install"
)

func IsMinicondaInstalled() bool {
	_, err := gutils.ExecuteSysCommand(true, "", "conda", "--help")
	return err == nil
}

func IsCoursierInstalled() bool {
	_, err := gutils.ExecuteSysCommand(true, "", "cs", "--help")
	return err == nil
}

type SDKInstaller interface {
	Initiate(originSDKName, versionName string, version download.Item)
	SetInstallConf(iconf download.InstallerConfig)
	FormatSDKName()
	GetInstallDir() string
	GetSymbolLinkPath() string
	Install()
}

/*
SDK Installer.
*/
type Installer struct {
	OriginSDKName string
	VersionName   string
	Version       download.Item
	sdkInstaller  SDKInstaller
	installerConf download.InstallerConfig
}

func NewInstaller(originSDKName, versionName, intallSha256 string, version download.Item) (i *Installer) {
	i = &Installer{
		OriginSDKName: originSDKName,
		VersionName:   versionName,
		Version:       version,
	}
	switch version.Installer {
	case download.Conda, download.CondaForge:
		i.sdkInstaller = install.NewCondaInstaller()
	case download.Coursier:
		i.sdkInstaller = install.NewCoursierInstaller()
	case download.Executable, download.Dpkg, download.Rpm:
		i.sdkInstaller = install.NewExeInstaller()
	default:
		i.sdkInstaller = install.NewArchiverInstaller()
	}
	i.sdkInstaller.Initiate(originSDKName, versionName, version)
	i.installerConf = download.GetSDKInstallationConfig(originSDKName, intallSha256)
	i.sdkInstaller.SetInstallConf(i.installerConf)
	return
}

func (i *Installer) Install() {
	i.sdkInstaller.Install()
}

func (i *Installer) Uninstall() {}

func (i *Installer) SetEnv() {}

func (i *Installer) UnsetEnv() {}
