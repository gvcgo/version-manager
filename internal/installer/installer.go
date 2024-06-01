package installer

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/installer/install"
	"github.com/gvcgo/version-manager/internal/installer/post"
	"github.com/gvcgo/version-manager/internal/shell"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/utils"
)

type InvokeMode string

const (
	AddToPathTemporarillyEnvName string     = "VMR_ADD_TO_PATH_TEMPORARILY"
	ModeGlobally                 InvokeMode = "globally"
	ModeSessionly                InvokeMode = "sessionly"
	ModeToLock                   InvokeMode = "to-lock"
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
	Mode          InvokeMode
}

func NewInstaller(originSDKName, versionName, intallSha256 string, version download.Item) (i *Installer) {
	i = &Installer{
		OriginSDKName: originSDKName,
		VersionName:   versionName,
		Version:       version,
		Shell:         shell.NewShell(),
		Mode:          ModeGlobally,
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

func (i *Installer) SetInvokeMode(m InvokeMode) {
	i.Mode = m
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
		// os.Symlink(installDir, symbolPath)
		utils.CreateSymLink(installDir, symbolPath)
	}
}

func (i *Installer) CollectEnvs(basePath string) map[string][]string {
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

func (i *Installer) AddEnvsTemporarilly() {
	if !gconv.Bool(os.Getenv(AddToPathTemporarillyEnvName)) {
		return
	}
	installDir := i.sdkInstaller.GetInstallDir()
	envList := i.CollectEnvs(installDir)
	// fmt.Printf("%s envsList: %+v\n", i.OriginSDKName, envList)
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
	envList := i.CollectEnvs(symbolPath)
	for key, value := range envList {
		if key == "PATH" {
			i.Shell.SetPath(utils.JoinPath(value...))
		} else {
			p := utils.JoinPath(value...)
			if p != "" {
				i.Shell.SetEnv(key, p)
			}
		}
	}
}

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

	if !i.IsInstalled() {
		i.sdkInstaller.Install()
		// post-install handler.
		if handler, ok := post.PostInstallHandlers[i.OriginSDKName]; ok {
			handler(i.VersionName, i.Version)
		}
	} else if i.Mode == ModeGlobally {
		gprint.PrintInfo(fmt.Sprintf("%s %s is already installed.", i.OriginSDKName, i.VersionName))
	}

	if i.Mode == ModeGlobally {
		i.CreateSymlink()
		i.SetEnvGlobally()
		i.AddEnvsTemporarilly()
	} else {
		if i.Mode == ModeToLock {
			i.writeLockFile()
		}

		// terminal.ModifyPathForPty(i.OriginSDKName)
		RemoveGlobalSDKPathTemporarily(i.OriginSDKName)
		// Enable temporary envs.
		os.Setenv(AddToPathTemporarillyEnvName, "1")
		i.AddEnvsTemporarilly()

		t := terminal.NewPtyTerminal()
		t.Run()
	}
}

func (i *Installer) writeLockFile() {
	l := NewVLocker()
	l.Save(i.OriginSDKName, i.VersionName)
}

func (i *Installer) Uninstall() {
	ivFinder := NewIVFinder(i.OriginSDKName)
	_, current := ivFinder.FindAll()
	installDir := i.sdkInstaller.GetInstallDir()
	installDir = strings.TrimSuffix(installDir, "<current>")
	os.RemoveAll(installDir)
	if current == i.VersionName {
		symbolPath := i.sdkInstaller.GetSymbolLinkPath()
		os.RemoveAll(symbolPath)
		i.UnsetEnv()
	}
}

func (i *Installer) UnsetEnv() {
	symbolPath := i.sdkInstaller.GetSymbolLinkPath()
	envList := i.CollectEnvs(symbolPath)
	for key, value := range envList {
		if key == "PATH" {
			i.Shell.UnsetPath(utils.JoinPath(value...))
		} else {
			p := utils.JoinPath(value...)
			if p != "" {
				i.Shell.UnsetEnv(key)
			}
		}
	}
}
