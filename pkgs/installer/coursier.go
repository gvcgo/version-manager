package installer

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/envs"
	"github.com/gvcgo/version-manager/pkgs/utils"
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
		if c.V == nil {
			c.SearchVersion()
		}
		if c.V == nil {
			gprint.PrintError("Can't find version: %s", c.Version)
			return
		}
		if !IsCoursierInstalled() {
			gprint.PrintWarning("No coursier is installed. Please install coursier first.")
			os.Exit(1)
		}

		if conf.UseMirrorSiteInChina() {
			os.Setenv(
				"COURSIER_REPOSITORIES",
				"https://maven.aliyun.com/repository/public|https://maven.scijava.org/content/repositories/public",
			)
		}

		if strings.Contains(c.Version, "LTS") {
			c.Version = strings.ReplaceAll(c.Version, "-LTS", "")
			c.Version = strings.ReplaceAll(c.Version, " LTS", "")
		}
		installDir := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.Version)
		_, err := gutils.ExecuteSysCommand(
			false, "",
			"cs", "install",
			"-P",
			fmt.Sprintf("--install-dir=%s", installDir),
			fmt.Sprintf("scala:%s", c.Version),
		)
		if err == nil {
			symbolicPath := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.AppName)
			os.RemoveAll(symbolicPath)
			utils.SymbolicLink(installDir, symbolicPath)
			em := envs.NewEnvManager()
			em.AddToPath(symbolicPath)
		}
	}

	c.UnInstall = func(appName, version string) {
		symbolicPath := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.AppName)
		slink, _ := os.Readlink(symbolicPath)
		if filepath.Base(slink) == version {
			gprint.PrintWarning("Can not remove a version currently in use: %s", version)
			return
		}

		installDir := filepath.Join(conf.GetVMVersionsDir(c.AppName), version)
		os.RemoveAll(installDir)
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
	if c.V != nil {
		symbolicPath := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.AppName)
		installDir := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.Version)
		if ok, _ := gutils.PathIsExist(installDir); ok {
			os.RemoveAll(symbolicPath)
			utils.SymbolicLink(installDir, symbolicPath)
			gprint.PrintSuccess("Switched to %s", c.Version)
			os.Exit(0)
		}
	}
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
	vDir := conf.GetVMVersionsDir(c.AppName)
	symbolicPath := filepath.Join(vDir, c.AppName)
	if ok, _ := gutils.PathIsExist(symbolicPath); ok {
		em := envs.NewEnvManager()
		em.DeleteFromPath(symbolicPath)
	}
	os.RemoveAll(vDir)
}

func (c *CoursierInstaller) ClearCache() {}

func (c *CoursierInstaller) GetHomepage() string {
	return c.HomePage
}
