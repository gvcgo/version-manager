package install

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/spinner"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	MinicondaSDKName string = "miniconda"
	ErlangSDKName    string = "erlang"
	ElixirSDKName    string = "elixir"
	VSCodeSDKName    string = "vscode"
)

/*
Install miniconda.
*/
func InstallMiniconda(exePath, installDir string) (task *utils.SysCommandRunner) {
	var commands []string
	homeDir, _ := os.UserHomeDir()
	if runtime.GOOS == "windows" {
		/*
			start /wait "" Miniconda3-latest-Windows-x86_64.exe /InstallationType=JustMe /RegisterPython=0 /S /D=%UserProfile%\Miniconda3
		*/
		commands = []string{
			"start",
			"/wait",
			"",
			exePath,
			"/InstallationType=JustMe",
			"/RegisterPython=0",
			"/S",
			fmt.Sprintf("/D=%s", installDir),
		}
	} else {
		gutils.ExecuteSysCommand(true, homeDir, "chmod", "+x", exePath)
		/*
			bash ~/miniconda.sh -b -p $HOME/miniconda
		*/
		commands = []string{
			"bash",
			exePath,
			"-b",
			"-p",
			installDir,
		}
	}
	// _, err = gutils.ExecuteSysCommand(true, homeDir, commands...)
	task = utils.NewSysCommandRunner(true, homeDir, commands...)
	return
}

// vscode
func InstallVSCode(pkgFilePath, installDir string) (task *utils.SysCommandRunner) {
	homeDir, _ := os.UserHomeDir()
	switch runtime.GOOS {
	case gutils.Windows:
		if strings.HasSuffix(pkgFilePath, ".exe") {
			// _, err = gutils.ExecuteSysCommand(true, homeDir,
			// 	pkgFilePath, "/VERYSILENT", "/MERGETASKS=!runcode")
			task = utils.NewSysCommandRunner(
				true,
				homeDir,
				pkgFilePath,
				"/VERYSILENT",
				"/MERGETASKS=!runcode",
			)
		}
	case gutils.Linux:
		if strings.HasSuffix(pkgFilePath, ".deb") {
			// _, err = gutils.ExecuteSysCommand(true, homeDir,
			// 	"sudo", "dpkg", "-i", pkgFilePath)
			task = utils.NewSysCommandRunner(
				true,
				homeDir,
				"sudo",
				"dpkg",
				"-i",
				pkgFilePath,
			)
		} else if strings.HasSuffix(pkgFilePath, ".rpm") {
			// _, err = gutils.ExecuteSysCommand(true, homeDir,
			// 	"sudo", "rpm", "-ivh", pkgFilePath)
			task = utils.NewSysCommandRunner(
				true,
				homeDir,
				"sudo",
				"rpm",
				"-ivh",
				pkgFilePath,
			)
		}
	case gutils.Darwin:
		err := utils.Extract(pkgFilePath, cnf.GetTempDir())
		if err != nil {
			gprint.PrintError("extract vscode failed: %v", err)
			os.RemoveAll(pkgFilePath)
			return
		}
		appName := "Visual Studio Code.app"
		ff := utils.NewFinder(appName)
		ff.Find(cnf.GetTempDir())
		appPath := filepath.Join(ff.GetDirName(), appName)
		if ok, _ := gutils.PathIsExist(appPath); ok {
			utils.MoveFileOnUnixSudo(appPath, "/Applications")
		}
		os.RemoveAll(cnf.GetTempDir())
	default:
	}
	return
}

/*
install *.exe for windows
1. erlang
2. elixir
*/
func InstallExeForWindows(exePath, installDir string) (task *utils.SysCommandRunner) {
	homeDir, _ := os.UserHomeDir()
	// _, err = gutils.ExecuteSysCommand(true, homeDir,
	// 	"start", "/wait", exePath, "/S", fmt.Sprintf("/D=%s", installDir))
	task = utils.NewSysCommandRunner(
		true,
		homeDir,
		"start",
		"/wait",
		exePath,
		"/S",
		fmt.Sprintf("/D=%s", installDir),
	)
	return
}

// Other standalone executables.
func InstallStandAloneExecutables(exePath, installDir string) (task *utils.SysCommandRunner) {
	fName := filepath.Base(exePath)
	os.MkdirAll(installDir, os.ModePerm)
	destPath := filepath.Join(installDir, fName)
	var err error
	if ok, _ := gutils.PathIsExist(exePath); ok {
		err = gutils.CopyAFile(exePath, destPath)
	}

	if err != nil {
		gprint.PrintError("%+v", err)
		os.RemoveAll(filepath.Dir(exePath))
	}

	if err == nil && runtime.GOOS != gutils.Windows {
		gutils.ExecuteSysCommand(true, installDir, "chmod", "+x", destPath)
	}
	return
}

/*
1. *.exe
2. *.deb
3. *.rpm
4. *.sh (miniconda)
5. unix-like executable
*/
type ExeInstaller struct {
	PluginName  string
	SDKName     string
	VersionName string
	Version     lua_global.Item
	Fetcher     *request.Fetcher
	spinner     *spinner.Spinner
	downloader  *download.Downloader
	installConf *lua_global.InstallerConfig
	signal      chan struct{}
}

func NewExeInstaller() (ei *ExeInstaller) {
	ei = &ExeInstaller{
		spinner:    spinner.NewSpinner(),
		downloader: download.NewDownloader(),
		signal:     make(chan struct{}),
	}
	return
}

func (ei *ExeInstaller) SetInstallConf(iconf *lua_global.InstallerConfig) {
	ei.installConf = iconf
}

func (ei *ExeInstaller) Initiate(pluginName, sdkName, versionName string, version lua_global.Item) {
	ei.PluginName = pluginName
	ei.SDKName = sdkName
	ei.VersionName = versionName
	ei.Version = version
}

func (ei *ExeInstaller) GetInstallDir() string {
	d := GetSDKVersionDir(ei.SDKName)
	return filepath.Join(d, fmt.Sprintf(VersionInstallDirPattern, ei.PluginName, ei.VersionName))
}

func (ei *ExeInstaller) GetSymbolLinkPath() string {
	d := GetSDKVersionDir(ei.SDKName)
	return filepath.Join(d, ei.SDKName)
}

func (ei *ExeInstaller) RenameFile() {
	if ei.installConf.BinaryRename != nil && ei.installConf.BinaryRename.NameFlag != "" {
		installDir := ei.GetInstallDir()
		dList, _ := os.ReadDir(installDir)
		for _, dd := range dList {
			if !dd.IsDir() && strings.Contains(dd.Name(), ei.installConf.BinaryRename.NameFlag) {
				newName := filepath.Join(installDir, ei.installConf.BinaryRename.RenameTo)
				if runtime.GOOS == "windows" {
					newName += ".exe"
				}
				os.Rename(filepath.Join(installDir, dd.Name()), newName)
			}
		}
	}
}

func (ei *ExeInstaller) Install() {
	localPath := ei.downloader.Download(ei.PluginName, ei.VersionName, ei.Version)
	if localPath == "" {
		return
	}
	var task *utils.SysCommandRunner

	switch ei.PluginName {
	case MinicondaSDKName:
		task = InstallMiniconda(localPath, ei.GetInstallDir())
	case ErlangSDKName, ElixirSDKName:
		task = InstallExeForWindows(localPath, ei.GetInstallDir())
	case VSCodeSDKName:
		task = InstallVSCode(localPath, ei.GetInstallDir())
	default:
		task = InstallStandAloneExecutables(localPath, ei.GetInstallDir())
		ei.RenameFile()
	}

	ei.spinner.SetTitle(fmt.Sprintf("Installing %s", ei.PluginName))
	ei.spinner.SetSweepFunc(func() {
		if task != nil {
			task.Cancel()
		}
		ei.signal <- struct{}{}
		os.RemoveAll(ei.GetInstallDir())
	})
	go ei.spinner.Run()

	go func() {
		var err error

		// switch ei.OriginSDKName {
		// case MinicondaSDKName:
		// 	err = InstallMiniconda(localPath, ei.GetInstallDir())
		// case ErlangSDKName, ElixirSDKName:
		// 	err = InstallExeForWindows(localPath, ei.GetInstallDir())
		// case VSCodeSDKName:
		// 	err = InstallVSCode(localPath, ei.GetInstallDir())
		// default:
		// 	err = InstallStandAloneExecutables(localPath, ei.GetInstallDir())
		// 	ei.RenameFile()
		// }

		if task != nil {
			err = task.Run()
		}
		if err != nil {
			gprint.PrintError("%+v", err)
			os.RemoveAll(filepath.Dir(localPath))
		}

		ei.signal <- struct{}{}
		ei.spinner.Quit()
	}()

	<-ei.signal
	// time.Sleep(time.Second * 2) // cursor fix.
}
