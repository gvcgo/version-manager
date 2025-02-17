package install

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/spinner"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	CoursierPathEnvName string = "VMR_COURSIER_PATH"
)

/*
Install use coursier.
*/
type CoursierInstaller struct {
	PluginName  string
	SDKName     string
	VersionName string
	Version     lua_global.Item
	spinner     *spinner.Spinner
	installConf *lua_global.InstallerConfig
	signal      chan struct{}
}

func NewCoursierInstaller() (c *CoursierInstaller) {
	c = &CoursierInstaller{
		spinner: spinner.NewSpinner(),
		signal:  make(chan struct{}),
	}
	return
}

func (c *CoursierInstaller) SetInstallConf(iconf *lua_global.InstallerConfig) {
	c.installConf = iconf
}

func (c *CoursierInstaller) Initiate(pluginName, sdkName, versionName string, version lua_global.Item) {
	c.PluginName = pluginName
	c.SDKName = sdkName
	c.VersionName = versionName
	c.Version = version
}

func (c *CoursierInstaller) GetInstallDir() string {
	d := GetSDKVersionDir(c.SDKName)
	return filepath.Join(d, fmt.Sprintf(VersionInstallDirPattern, c.PluginName, c.VersionName))
}

func (c *CoursierInstaller) GetSymbolLinkPath() string {
	d := GetSDKVersionDir(c.SDKName)
	return filepath.Join(d, c.SDKName)
}

func (c *CoursierInstaller) Install() {
	homeDir, _ := os.UserHomeDir()

	/*
		https://get-coursier.io/docs/cli-install
	*/
	version := strings.TrimSuffix(c.VersionName, "-LTS")
	version = strings.TrimSuffix(version, "-lts")

	coursierCommand := os.Getenv(CoursierPathEnvName)
	if coursierCommand == "" {
		coursierCommand = "cs"
	}
	task := utils.NewSysCommandRunner(
		true,
		homeDir,
		coursierCommand,
		"install",
		"-q",
		fmt.Sprintf("--install-dir=%s", c.GetInstallDir()),
		fmt.Sprintf("%s:%s", c.PluginName, version),
	)

	c.spinner.SetTitle(fmt.Sprintf("Coursier installing %s", c.PluginName))
	c.spinner.SetSweepFunc(func() {
		task.Cancel()
		c.signal <- struct{}{}
		os.RemoveAll(c.GetInstallDir())
	})
	go c.spinner.Run()

	go func() {
		// _, err := gutils.ExecuteSysCommand(
		// 	true, homeDir,
		// 	coursierCommand, "install",
		// 	"-q",
		// 	fmt.Sprintf("--install-dir=%s", c.GetInstallDir()),
		// 	fmt.Sprintf("%s:%s", c.OriginSDKName, version),
		// )
		if err := task.Run(); err != nil {
			gprint.PrintError("%+v", err)
		}
		c.signal <- struct{}{}
		c.spinner.Quit()
	}()

	<-c.signal
	// time.Sleep(time.Duration(2) * time.Second)
}
