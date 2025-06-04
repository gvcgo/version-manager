package installer

import (
	"os"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	MinicondaSDKName string = "miniconda"
	CoursierSDKName  string = "coursier"
)

/*
Automatically detects and installs prerequisites for the installer.
1. miniconda latest
2. coursier latest
*/

func installPrequisite(pluginName string) {
	// add envs temporarily, so the following command will easilly find prequisites.
	os.Setenv(AddToPathTemporarillyEnvName, "1")
	versions := plugin.NewVersions(pluginName)
	sdkName := versions.GetSDKName()
	vName, vItem := versions.GetLatestVersion()
	ins := NewInstaller(sdkName, pluginName, vName, vItem)
	ins.Install()
}

func CheckAndInstallMiniconda() {
	if !utils.IsMinicondaInstalled() {
		gprint.PrintInfo("Installing miniconda first: ")
		installPrequisite(MinicondaSDKName)
	}
}

func CheckAndInstallCoursier() {
	if !utils.IsCoursierInstalled() {
		gprint.PrintInfo("Installing coursier first: ")
		installPrequisite(CoursierSDKName)
	}
}
