/*
 @@    Copyright (c) 2024 moqsien@hotmail.com
 @@
 @@    Permission is hereby granted, free of charge, to any person obtaining a copy of
 @@    this software and associated documentation files (the "Software"), to deal in
 @@    the Software without restriction, including without limitation the rights to
 @@    use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 @@    the Software, and to permit persons to whom the Software is furnished to do so,
 @@    subject to the following conditions:
 @@
 @@    The above copyright notice and this permission notice shall be included in all
 @@    copies or substantial portions of the Software.
 @@
 @@    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 @@    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 @@    FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 @@    COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 @@    IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 @@    CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

package installer

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/envs"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/pkgs/conf"
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
			c.GetVersion()
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

		installDir := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.Version)
		_, err := gutils.ExecuteSysCommand(
			false, "",
			"cs", "install",
			"-P",
			fmt.Sprintf("--install-dir=%s", installDir),
			fmt.Sprintf("scala:%s", c.Version),
		)
		if err == nil {
			c.NewPty(installDir) // for session scope only.

			symbolicPath := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.AppName)
			os.RemoveAll(symbolicPath)
			utils.SymbolicLink(installDir, symbolicPath)
			em := envs.NewEnvManager()
			defer em.CloseKey()
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

func (c *CoursierInstaller) FixAppName() {}

func (c *CoursierInstaller) GetVersion() {
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
	// Handle "LTS"
	if strings.Contains(c.Version, "LTS") {
		c.Version = strings.ReplaceAll(c.Version, "-LTS", "")
		c.Version = strings.ReplaceAll(c.Version, " LTS", "")
	}
}

// Uses a version only in current session.
func (c *CoursierInstaller) NewPty(installDir string) {
	if gconv.Bool(os.Getenv(conf.VMOnlyInCurrentSessionEnvName)) {
		t := terminal.NewPtyTerminal(c.AppName)
		t.AddEnv("PATH", installDir)
		t.Run()
	}
}

func (c *CoursierInstaller) Download() (zipFilePath string) {
	c.GetVersion()
	if c.V != nil {
		symbolicPath := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.AppName)
		installDir := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.Version)
		if ok, _ := gutils.PathIsExist(installDir); ok {
			c.NewPty(installDir) // for session scope only.

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
		defer em.CloseKey()
		em.DeleteFromPath(symbolicPath)
	}
	os.RemoveAll(vDir)
}

func (c *CoursierInstaller) ClearCache() {}

func (c *CoursierInstaller) GetHomepage() string {
	return c.HomePage
}

func (c *CoursierInstaller) SearchVersions() {
	if c.Searcher == nil {
		c.Searcher = NewSearcher()
	}
	c.Searcher.Search(c.AppName)
}
