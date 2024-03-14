package installer

import (
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/versions"
)

func IsCoursierInstalled() bool {
	_, err := gutils.ExecuteSysCommand(true, "", "cs", "--help")
	return err == nil
}

/*
Use coursier as installer for Scala.

https://get-coursier.io/docs/cli-install
*/
type CoursierInstaller struct {
	AppName   string
	Version   string
	Searcher  *Searcher
	V         *versions.VersionItem
	Install   func(appName, version, zipFilePath string)
	UnInstall func(appName, version string)
	HomePage  string
}

// TODO: scala installer.
