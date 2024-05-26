package install

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"time"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/spinner"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/download"
)

const (
	MinicondaSDKName string = "miniconda"
)

/*
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
}

func NewExeInstaller() (ei *ExeInstaller) {
	ei = &ExeInstaller{
		spinner:    spinner.NewSpinner(),
		downloader: download.NewDownloader(),
	}
	return
}

func (ei *ExeInstaller) FormatSDKName() {
	ei.SDKName = ei.OriginSDKName
}

func (ei *ExeInstaller) GetInstallDir() string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, ei.SDKName)
	os.MkdirAll(d, os.ModePerm)
	return filepath.Join(d, fmt.Sprintf(VersionInstallDirPattern, ei.OriginSDKName, ei.VersionName))
}

func (ei *ExeInstaller) GetSymbolLinkPath() string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, ei.SDKName)
	os.MkdirAll(d, os.ModePerm)
	return filepath.Join(d, ei.SDKName)
}

func (ei *ExeInstaller) Install(originSDKName, versionName string, version download.Item) {
	ei.OriginSDKName = originSDKName
	ei.VersionName = versionName
	ei.Version = version
	ei.FormatSDKName()

	localPath := ei.downloader.Download(originSDKName, versionName, version)
	if localPath == "" {
		return
	}
	ei.spinner.SetTitle(fmt.Sprintf("Installing %s", originSDKName))
	go ei.spinner.Run()
	var err error
	if originSDKName == MinicondaSDKName {
		err = InstallMiniconda(localPath, ei.GetInstallDir())
	}
	ei.spinner.Quit()
	time.Sleep(time.Second * 2) // cursor fix.
	if err != nil {
		gprint.PrintError("%+v", err)
	}
}
