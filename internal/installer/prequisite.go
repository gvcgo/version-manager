package installer

import (
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/download"
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

func installPrequisite(sdkName string) {
	vName, vItem := download.GetLatestVersionBySDKName(sdkName)

	sdkList := download.GetSDKList()
	sdkItem := sdkList[sdkName]
	ins := NewInstaller(sdkName, vName, sdkItem.InstallConfSha256, vItem)
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
