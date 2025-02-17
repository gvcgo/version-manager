package install

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/spinner"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	CondaPathEnvName string = "VMR_CONDA_PATH"
)

/*
Install using conda.
*/
type CondaInstaller struct {
	PluginName  string
	SDKName     string
	VersionName string
	Version     lua_global.Item
	spinner     *spinner.Spinner
	installConf *lua_global.InstallerConfig
	signal      chan struct{}
}

func NewCondaInstaller() (c *CondaInstaller) {
	c = &CondaInstaller{
		spinner: spinner.NewSpinner(),
		signal:  make(chan struct{}),
	}
	return
}

func (c *CondaInstaller) SetInstallConf(iconf *lua_global.InstallerConfig) {
	c.installConf = iconf
}

func (c *CondaInstaller) Initiate(pluginName, sdkName, versionName string, version lua_global.Item) {
	c.PluginName = pluginName
	c.SDKName = sdkName
	c.VersionName = versionName
	c.Version = version
}

func (c *CondaInstaller) GetInstallDir() string {
	d := GetSDKVersionDir(c.SDKName)
	return filepath.Join(d, fmt.Sprintf(VersionInstallDirPattern, c.PluginName, c.VersionName))
}

func (c *CondaInstaller) GetSymbolLinkPath() string {
	d := GetSDKVersionDir(c.SDKName)
	return filepath.Join(d, c.SDKName)
}

func (c *CondaInstaller) Install() {
	homeDir, _ := os.UserHomeDir()
	/*
		https://docs.conda.io/projects/conda/en/latest/commands/create.html
		Example: conda create -q -y --prefix=~/.vm/versions/pypy_versions -c conda-forge pypy python=3.8
	*/
	condaCommand := os.Getenv(CondaPathEnvName)
	if condaCommand == "" {
		condaCommand = "conda"
	}

	task := utils.NewSysCommandRunner(
		true, homeDir,
		condaCommand, "create",
		"-q", "-y",
		fmt.Sprintf("--prefix=%s", c.GetInstallDir()),
		"-c", "conda-forge", c.PluginName,
		fmt.Sprintf("%s=%s", c.PluginName, c.VersionName),
	)

	c.spinner.SetTitle(fmt.Sprintf("Conda installing %s", c.PluginName))
	c.spinner.SetSweepFunc(func() {
		task.Cancel()
		c.signal <- struct{}{}
		os.RemoveAll(c.GetInstallDir())
	})
	go c.spinner.Run()

	go func() {
		// _, err := gutils.ExecuteSysCommand(
		// 	true, homeDir,
		// 	condaCommand, "create",
		// 	"-q", "-y",
		// 	fmt.Sprintf("--prefix=%s", c.GetInstallDir()),
		// 	"-c", "conda-forge", c.OriginSDKName,
		// 	fmt.Sprintf("%s=%s", c.OriginSDKName, c.VersionName),
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
