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

const (
	VersionInstallDirPattern string = "%s-%s"
)

/*
Install using conda.
*/
type CondaInstaller struct {
	OriginSDKName string
	SDKName       string
	VersionName   string
	Version       download.Item
	spinner       *spinner.Spinner
}

func NewCondaInstaller() (c *CondaInstaller) {
	c = &CondaInstaller{
		spinner: spinner.NewSpinner(),
	}
	return
}

func (c *CondaInstaller) FormatSDKName() {
	if c.OriginSDKName == "pypy" {
		c.SDKName = "python"
	} else {
		c.SDKName = c.OriginSDKName
	}
}

func (c *CondaInstaller) GetInstallDir() string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, c.SDKName)
	os.MkdirAll(d, os.ModePerm)
	return filepath.Join(d, fmt.Sprintf(VersionInstallDirPattern, c.OriginSDKName, c.VersionName))
}

func (c *CondaInstaller) GetSymbolLinkPath() string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, c.SDKName)
	os.MkdirAll(d, os.ModePerm)
	return filepath.Join(d, c.SDKName)
}

func (c *CondaInstaller) Install(originSDKName, versionName string, version download.Item) {
	c.OriginSDKName = originSDKName
	c.VersionName = versionName
	c.Version = version
	c.FormatSDKName()

	homeDir, _ := os.UserHomeDir()
	/*
		https://docs.conda.io/projects/conda/en/latest/commands/create.html
		Example: conda create -q -y --prefix=~/.vm/versions/pypy_versions -c conda-forge pypy python=3.8
	*/
	c.spinner.SetTitle(fmt.Sprintf("Conda installing %s", c.OriginSDKName))
	go c.spinner.Run()
	_, err := gutils.ExecuteSysCommand(
		true, homeDir,
		"conda", "create",
		"-q", "-y",
		fmt.Sprintf("--prefix=%s", c.GetInstallDir()),
		"-c", "conda-forge", c.OriginSDKName,
		fmt.Sprintf("%s=%s", c.OriginSDKName, c.VersionName),
	)
	c.spinner.Quit()
	time.Sleep(time.Duration(2) * time.Second)

	if err != nil {
		gprint.PrintError("%+v", err)
	}
}
