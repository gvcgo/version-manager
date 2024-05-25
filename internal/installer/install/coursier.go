package install

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/utils"
)

/*
Install use coursier.
*/
type CoursierInstaller struct {
	OriginSDKName string
	SDKName       string
	VersionName   string
	Version       utils.Item
}

func NewCoursierInstaller() (c *CoursierInstaller) {
	c = &CoursierInstaller{}
	return
}

func (c *CoursierInstaller) FormatSDKName() {
	c.SDKName = c.OriginSDKName
}

func (c *CoursierInstaller) GetInstallDir() string {
	versionDir := cnf.GetVersionsDir()
	return filepath.Join(versionDir, c.SDKName, fmt.Sprintf("%s%s", c.OriginSDKName, c.VersionName))
}

func (c *CoursierInstaller) Install(originSDKName, versionName string, version utils.Item) {
	c.OriginSDKName = originSDKName
	c.VersionName = versionName
	c.Version = version
	c.FormatSDKName()

	homeDir, _ := os.UserHomeDir()
	_, err := gutils.ExecuteSysCommand(
		false, homeDir,
		"cs", "install",
		"-P",
		fmt.Sprintf("--install-dir=%s", c.GetInstallDir()),
		fmt.Sprintf("%s:%s", c.OriginSDKName, c.VersionName),
	)
	if err != nil {
		gprint.PrintError("%+v", err)
	}
}
