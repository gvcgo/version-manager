package installer

import (
	"os"
	"path/filepath"
	"runtime"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/installer/install"
	"github.com/gvcgo/version-manager/internal/shell"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	AddToPathTemporarillyEnvName string = "VMR_ADD_TO_PATH_TEMPORARILY"
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

func (i *Installer) GetSDKInstaller() (si SDKInstaller) {
	return i.sdkInstaller
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

func (i *Installer) collectEnvs(basePath string) map[string][]string {
	result := make(map[string][]string)
	if ok, _ := gutils.PathIsExist(basePath); ok {
		binDirList := []download.DirPath{}
		dd := i.installerConf.BinaryDirs
		if dd != nil {
			switch runtime.GOOS {
			case gutils.Darwin:
				binDirList = i.installerConf.BinaryDirs.MacOS
			case gutils.Linux:
				binDirList = i.installerConf.BinaryDirs.Linux
			case gutils.Windows:
				binDirList = i.installerConf.BinaryDirs.Windows
			default:
			}
		}
		if len(binDirList) == 0 {
			binDirList = append(binDirList, download.DirPath{})
		}
		for _, dirPath := range binDirList {
			pList := append([]string{basePath}, dirPath...)
			p := filepath.Join(pList...)
			if ok1, _ := gutils.PathIsExist(p); ok1 {
				result["PATH"] = append(result["PATH"], p)
			}
		}

		// Other envs.
		aa := i.installerConf.AdditionalEnvs
		for _, addEnv := range aa {
			if len(addEnv.Value) == 0 {
				addEnv.Value = append(addEnv.Value, download.DirPath{})
			}
			dirList := []string{}
			for _, dirPath := range addEnv.Value {
				dPath := append([]string{basePath}, dirPath...)
				p := filepath.Join(dPath...)
				if ok, _ := gutils.PathIsExist(p); ok {
					dirList = append(dirList, p)
				}
			}
			result[addEnv.Name] = dirList
		}
	}
	return result
}

func (i *Installer) addToPathTemporarilly() {
	if !gconv.Bool(os.Getenv(AddToPathTemporarillyEnvName)) {
		return
	}
	installDir := i.sdkInstaller.GetInstallDir()
	envList := i.collectEnvs(installDir)
	for key, value := range envList {
		if key == "PATH" {
			p := utils.JoinPath(value...)
			if p != "" {
				newPathEnv := utils.JoinPath(p, os.Getenv("PATH"))
				os.Setenv("PATH", newPathEnv)
			}
		} else {
			newValue := utils.JoinPath(value...)
			if newValue != "" {
				os.Setenv(key, newValue)
			}
		}
	}
}

func (i *Installer) SetEnvGlobally() {
	symbolPath := i.sdkInstaller.GetSymbolLinkPath()
	envList := i.collectEnvs(symbolPath)
	for key, value := range envList {
		if key == "PATH" {
			i.Shell.SetPath(utils.JoinPath(value...))
		} else {
			i.Shell.SetEnv(key, utils.JoinPath(value...))
		}
	}
}

func (i *Installer) CollectEnvsForSession() {}

func (i *Installer) IsInstalled() bool {
	installDir := i.sdkInstaller.GetInstallDir()
	entries, _ := os.ReadDir(installDir)
	return len(entries) > 0
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

	// TODO: sessionly, locked.
	if !i.IsInstalled() {
		i.sdkInstaller.Install()
		i.CreateSymlink()
		i.SetEnvGlobally()
		i.addToPathTemporarilly()
	}
}

func (i *Installer) Uninstall() {}

func (i *Installer) SetEnv() {}

func (i *Installer) UnsetEnv() {}
