package install

import (
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/spinner"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/download"
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
}

func NewCoursierInstaller() (c *CoursierInstaller) {
	c = &CoursierInstaller{
		spinner: spinner.NewSpinner(),
	}
	return
}

func (c *CoursierInstaller) FormatSDKName() {
	c.SDKName = c.OriginSDKName
}

func (c *CoursierInstaller) GetInstallDir() string {
	versionDir := cnf.GetVersionsDir()
	return filepath.Join(versionDir, c.SDKName, fmt.Sprintf("%s%s", c.OriginSDKName, c.VersionName))
}

func (c *CoursierInstaller) Install(originSDKName, versionName string, version download.Item) {
	c.OriginSDKName = originSDKName
	c.VersionName = versionName
	c.Version = version
	c.FormatSDKName()

	homeDir, _ := os.UserHomeDir()
	c.spinner.SetTitle(fmt.Sprintf("Coursier installing %s", c.OriginSDKName))
	go c.spinner.Run()
	/*
		https://get-coursier.io/docs/cli-install
	*/
	_, err := gutils.ExecuteSysCommand(
		false, homeDir,
		"cs", "install",
		"-q",
		fmt.Sprintf("--install-dir=%s", c.GetInstallDir()),
		fmt.Sprintf("%s:%s", c.OriginSDKName, c.VersionName),
	)
	c.spinner.Quit()
	time.Sleep(time.Duration(2) * time.Second)
	if err != nil {
		gprint.PrintError("%+v", err)
	}
}
