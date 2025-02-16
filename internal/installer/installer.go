package installer

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/installer/install"
	"github.com/gvcgo/version-manager/internal/installer/post"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
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
	Initiate(originSDKName, versionName string, version lua_global.Item)
	SetInstallConf(iconf *lua_global.InstallerConfig)
	FormatSDKName()
	GetInstallDir() string
	GetSymbolLinkPath() string
	Install()
}

/*
SDK Installer.
*/
type Installer struct {
	PluginName    string
	SDKName       string
	VersionName   string
	Version       lua_global.Item
	sdkInstaller  SDKInstaller
	installerConf *lua_global.InstallerConfig
	Shell         shell.Sheller
	Mode          InvokeMode
	NoEnvs        bool
}

func NewInstaller(sdkName, pluginName, versionName string, version lua_global.Item) (i *Installer) {
	i = &Installer{
		PluginName:  pluginName,
		SDKName:     sdkName,
		VersionName: versionName,
		Version:     version,
		Shell:       shell.NewShell(),
		Mode:        ModeGlobally,
		NoEnvs:      false,
	}
	switch version.Installer {
	case lua_global.Conda, lua_global.CondaForge:
		i.sdkInstaller = install.NewCondaInstaller()
	case lua_global.Coursier:
		i.sdkInstaller = install.NewCoursierInstaller()
	case lua_global.Executable, lua_global.Dpkg, lua_global.Rpm:
		i.sdkInstaller = install.NewExeInstaller()
	default:
		i.sdkInstaller = install.NewArchiverInstaller()
	}
	i.sdkInstaller.Initiate(sdkName, versionName, version)

	vv := plugin.NewVersions(pluginName)
	i.installerConf = vv.GetInstallerConfig()
	i.sdkInstaller.SetInstallConf(i.installerConf)
	return
}

func (i *Installer) DisableEnvs() {
	i.NoEnvs = true
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
	if i.NoEnvs {
		return result
	}
	if ok, _ := gutils.PathIsExist(basePath); ok {
		binDirList := []lua_global.DirPath{}
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
			binDirList = append(binDirList, lua_global.DirPath{})
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
				addEnv.Value = append(addEnv.Value, lua_global.DirPath{})
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
	if i.NoEnvs {
		return
	}
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
	if i.NoEnvs {
		return
	}
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
	case lua_global.Conda, lua_global.CondaForge:
		CheckAndInstallMiniconda()
	case lua_global.Coursier:
		CheckAndInstallCoursier()
	default:
	}

	if !i.IsInstalled() {
		i.sdkInstaller.Install()
		// post-install handler.
		if handler, ok := post.PostInstallHandlers[i.PluginName]; ok {
			handler(i.VersionName, i.Version)
		}
	} else if i.Mode == ModeGlobally {
		gprint.PrintInfo("%s %s is already installed.", i.PluginName, i.VersionName)
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
		RemoveGlobalSDKPathTemporarily(i.PluginName)
		// Enable temporary envs.
		os.Setenv(AddToPathTemporarillyEnvName, "1")
		i.AddEnvsTemporarilly()

		// t := terminal.NewPtyTerminal()
		// t.Run()
		terminal.RunTerminal()
	}
}

func (i *Installer) writeLockFile() {
	l := NewVLocker()
	l.Save(i.PluginName, i.VersionName)
}

func (i *Installer) Uninstall() {
	ivFinder := NewIVFinder(i.PluginName)
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
