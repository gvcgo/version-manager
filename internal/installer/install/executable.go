package install

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/spinner"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	MinicondaSDKName string = "miniconda"
	ErlangSDKName    string = "erlang"
	ElixirSDKName    string = "elixir"
	VSCodeSDKName    string = "vscode"
)

/*
TODO: Rename executable files.

Install miniconda.
*/
func InstallMiniconda(exePath, installDir string) (err error) {
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
	_, err = gutils.ExecuteSysCommand(true, homeDir, commands...)
	return
}

// vscode
func InstallVSCode(pkgFilePath, installDir string) (err error) {
	homeDir, _ := os.UserHomeDir()
	switch runtime.GOOS {
	case gutils.Windows:
		if strings.HasSuffix(pkgFilePath, ".exe") {
			_, err = gutils.ExecuteSysCommand(true, homeDir,
				pkgFilePath, "/VERYSILENT", "/MERGETASKS=!runcode")
		}
	case gutils.Linux:
		if strings.HasSuffix(pkgFilePath, ".deb") {
			_, err = gutils.ExecuteSysCommand(true, homeDir,
				"sudo", "dpkg", "-i", pkgFilePath)
		} else if strings.HasSuffix(pkgFilePath, ".rpm") {
			_, err = gutils.ExecuteSysCommand(true, homeDir,
				"sudo", "rpm", "-ivh", pkgFilePath)
		}
	case gutils.Darwin:
		err = utils.Extract(pkgFilePath, cnf.GetTempDir())
		if err != nil {
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
func InstallExeForWindows(exePath, installDir string) (err error) {
	return
}

// Other standalone executables.
func InstallStandAloneExecutables(exePath, installDir string) (err error) {
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
	OriginSDKName string
	SDKName       string
	VersionName   string
	Version       download.Item
	Fetcher       *request.Fetcher
	spinner       *spinner.Spinner
	downloader    *download.Downloader
	installConf   download.InstallerConfig
}

func NewExeInstaller() (ei *ExeInstaller) {
	ei = &ExeInstaller{
		spinner:    spinner.NewSpinner(),
		downloader: download.NewDownloader(),
	}
	return
}

func (ei *ExeInstaller) SetInstallConf(iconf download.InstallerConfig) {
	ei.installConf = iconf
}

func (ei *ExeInstaller) Initiate(originSDKName, versionName string, version download.Item) {
	ei.OriginSDKName = originSDKName
	ei.VersionName = versionName
	ei.Version = version
	ei.FormatSDKName()
}

func (ei *ExeInstaller) FormatSDKName() {
	ei.SDKName = ei.OriginSDKName
}

func (ei *ExeInstaller) GetInstallDir() string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf(VerisonDirPattern, ei.SDKName))
	os.MkdirAll(d, os.ModePerm)
	return filepath.Join(d, fmt.Sprintf(VersionInstallDirPattern, ei.OriginSDKName, ei.VersionName))
}

func (ei *ExeInstaller) GetSymbolLinkPath() string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf(VerisonDirPattern, ei.SDKName))
	os.MkdirAll(d, os.ModePerm)
	return filepath.Join(d, ei.SDKName)
}

func (ei *ExeInstaller) Install() {
	localPath := ei.downloader.Download(ei.OriginSDKName, ei.VersionName, ei.Version)
	if localPath == "" {
		return
	}
	ei.spinner.SetTitle(fmt.Sprintf("Installing %s", ei.OriginSDKName))
	go ei.spinner.Run()
	var err error

	switch ei.OriginSDKName {
	case MinicondaSDKName:
		err = InstallMiniconda(localPath, ei.GetInstallDir())
	case ErlangSDKName, ElixirSDKName:
		err = InstallExeForWindows(localPath, ei.GetInstallDir())
	case VSCodeSDKName:
		err = InstallVSCode(localPath, ei.GetInstallDir())
	default:
		err = InstallStandAloneExecutables(localPath, ei.GetInstallDir())
	}

	ei.spinner.Quit()
	time.Sleep(time.Second * 2) // cursor fix.
	if err != nil {
		gprint.PrintError("%+v", err)
	}
}
