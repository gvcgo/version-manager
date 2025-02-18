package installer

import (
	"os"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
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

func IsMinicondaInstalled() bool {
	_, err := gutils.ExecuteSysCommand(true, "", "conda", "--help")
	return err == nil
}

func IsCoursierInstalled() bool {
	_, err := gutils.ExecuteSysCommand(true, "", "cs", "--help")
	return err == nil
}

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
	if !IsMinicondaInstalled() {
		gprint.PrintInfo("Installing miniconda first: ")
		installPrequisite(MinicondaSDKName)
	}
}

func CheckAndInstallCoursier() {
	if !IsCoursierInstalled() {
		gprint.PrintInfo("Installing coursier first: ")
		installPrequisite(CoursierSDKName)
	}
}
