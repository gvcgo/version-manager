package installer

import (
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
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

func NewCoursierInstaller() *CoursierInstaller {
	c := &CoursierInstaller{
		AppName:  "scala",
		Version:  "3.3.3",
		Searcher: NewSearcher(),
		HomePage: "https://www.scala-lang.org/",
	}
	c.Install = func(appName, version, zipFilePath string) {
		// TODO: scala install
	}

	c.UnInstall = func(appName, version string) {
		// TODO: scala uninstall
	}
	return c
}

func (c *CoursierInstaller) SetVersion(version string) {
	c.Version = version
}

func (c *CoursierInstaller) SearchVersion() {
	if c.Searcher == nil {
		c.Searcher = NewSearcher()
	}
	vf := c.Searcher.GetVersions(c.AppName)
	vs := make([]string, 0)
	for key := range vf {
		if strings.Contains(key, c.Version) {
			vs = append(vs, key)
		}
	}

	if len(vs) == 0 {
		c.V = nil
		gprint.PrintError("Cannot find version: %s", c.Version)
	} else if len(vs) == 1 {
		c.Version = vs[0]
		c.V = &vf[c.Version][0]
	} else {
		c.V = nil
		gprint.PrintError("Found multiple versions: \n%v", strings.Join(vs, "\n"))
	}
}

func (c *CoursierInstaller) Download() (zipFilePath string) {
	c.SearchVersion()
	return ""
}

func (c *CoursierInstaller) Unzip(zipFilePath string) {}

func (c *CoursierInstaller) Copy() {}

func (c *CoursierInstaller) CreateVersionSymbol() {}

func (c *CoursierInstaller) CreateBinarySymbol() {}

func (c *CoursierInstaller) SetEnv() {}

func (c *CoursierInstaller) GetInstall() func(appName, version, zipFilePath string) {
	return c.Install
}

func (c *CoursierInstaller) InstallApp(zipFilePath string) {
	if c.Install != nil {
		c.Install(c.AppName, c.Version, zipFilePath)
	}
}

func (c *CoursierInstaller) UnInstallApp() {
	if c.AppName == "" {
		return
	}
	if c.Version == "all" {
		c.DeleteAll()
	} else {
		if c.UnInstall != nil {
			c.UnInstall(c.AppName, c.Version)
		}
	}
}

func (c *CoursierInstaller) DeleteVersion() {}

func (c *CoursierInstaller) DeleteAll() {
	if c.AppName == "" {
		return
	}
	// TODO: delete all for scala.
}

func (c *CoursierInstaller) ClearCache() {}

func (c *CoursierInstaller) GetHomepage() string {
	return c.HomePage
}
