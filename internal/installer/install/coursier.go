package install

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/spinner"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/download"
)

const (
	CoursierPathEnvName string = "VMR_COURSIER_PATH"
)

/*
Install use coursier.
*/
type CoursierInstaller struct {
	OriginSDKName string
	SDKName       string
	VersionName   string
	Version       download.Item
	spinner       *spinner.Spinner
	installConf   download.InstallerConfig
}

func NewCoursierInstaller() (c *CoursierInstaller) {
	c = &CoursierInstaller{
		spinner: spinner.NewSpinner(),
	}
	return
}

func (c *CoursierInstaller) SetInstallConf(iconf download.InstallerConfig) {
	c.installConf = iconf
}

func (c *CoursierInstaller) Initiate(originSDKName, versionName string, version download.Item) {
	c.OriginSDKName = originSDKName
	c.VersionName = versionName
	c.Version = version
	c.FormatSDKName()
}

func (c *CoursierInstaller) FormatSDKName() {
	c.SDKName = c.OriginSDKName
}

func (c *CoursierInstaller) GetInstallDir() string {
	d := GetSDKVersionDir(c.SDKName)
	return filepath.Join(d, fmt.Sprintf(VersionInstallDirPattern, c.OriginSDKName, c.VersionName))
}

func (c *CoursierInstaller) GetSymbolLinkPath() string {
	d := GetSDKVersionDir(c.SDKName)
	return filepath.Join(d, c.SDKName)
}

func (c *CoursierInstaller) Install() {
	homeDir, _ := os.UserHomeDir()
	c.spinner.SetTitle(fmt.Sprintf("Coursier installing %s", c.OriginSDKName))
	go c.spinner.Run()
	/*
		https://get-coursier.io/docs/cli-install
	*/
	coursierCommand := os.Getenv(CoursierPathEnvName)
	if coursierCommand == "" {
		coursierCommand = "cs"
	}

	version := strings.TrimSuffix(c.VersionName, "-LTS")
	version = strings.TrimSuffix(version, "-lts")
	_, err := gutils.ExecuteSysCommand(
		true, homeDir,
		coursierCommand, "install",
		"-q",
		fmt.Sprintf("--install-dir=%s", c.GetInstallDir()),
		fmt.Sprintf("%s:%s", c.OriginSDKName, version),
	)
	c.spinner.Quit()
	time.Sleep(time.Duration(2) * time.Second)
	if err != nil {
		gprint.PrintError("%+v", err)
	}
}
