package installer

import (
	"os"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/installer/install"
	"github.com/gvcgo/version-manager/internal/shell"
)

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
	Shell         shell.Sheller
}

func NewInstaller(originSDKName, versionName, intallSha256 string, version download.Item) (i *Installer) {
	i = &Installer{
		OriginSDKName: originSDKName,
		VersionName:   versionName,
		Version:       version,
		Shell:         shell.NewShell(),
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

func (i *Installer) CreateSymlink() {
	symbolPath := i.sdkInstaller.GetSymbolLinkPath()
	ok, _ := gutils.PathIsExist(symbolPath)
	installDir := i.sdkInstaller.GetInstallDir()
	ok1, _ := gutils.PathIsExist(installDir)

	if ok && ok1 {
		os.RemoveAll(symbolPath)
	}
	if ok1 {
		os.Symlink(installDir, symbolPath)
	}
}

func (i *Installer) Install() {
	// check prequisite.
	switch i.Version.Installer {
	case download.Conda, download.CondaForge:
		CheckAndInstallMiniconda()
	case download.Coursier:
		CheckAndInstallCoursier()
	default:
	}
	i.sdkInstaller.Install()
	i.CreateSymlink()
}

func (i *Installer) Uninstall() {}

func (i *Installer) SetEnv() {}

func (i *Installer) UnsetEnv() {}
