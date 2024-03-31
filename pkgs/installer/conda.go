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
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/envs"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/utils"
	"github.com/gvcgo/version-manager/pkgs/versions"
)

func IsMinicondaInstalled() bool {
	_, err := gutils.ExecuteSysCommand(true, "", "conda", "--help")
	return err == nil
}

/*
Use miniconda as installer.
*/
type CondaInstaller struct {
	AppName   string
	Version   string
	Searcher  *Searcher
	V         *versions.VersionItem
	Install   func(appName, version, zipFilePath string)
	UnInstall func(appName, version string)
	HomePage  string
}

func NewCondaInstaller() *CondaInstaller {
	c := &CondaInstaller{
		AppName:  "python",
		Version:  "3.12.0",
		Searcher: NewSearcher(),
		HomePage: "https://anaconda.org/conda-forge/python/files",
	}
	c.Install = func(appName, version, zipFilePath string) {
		if c.V == nil {
			c.SearchVersion()
		}
		if c.V == nil {
			gprint.PrintError("Can't find version: %s", c.Version)
			return
		}
		if !IsMinicondaInstalled() {
			gprint.PrintWarning("No conda is installed. Please install miniconda first.")
			os.Exit(1)
		}

		if conf.UseMirrorSiteInChina() {
			/*
				conda config --add channels https://mirrors.ustc.edu.cn/anaconda/pkgs/main/
				conda config --add channels https://mirrors.ustc.edu.cn/anaconda/pkgs/free/
				conda config --add channels https://mirrors.ustc.edu.cn/anaconda/cloud/conda-forge/
				conda config --add channels https://mirrors.ustc.edu.cn/anaconda/cloud/msys2/
				conda config --add channels https://mirrors.ustc.edu.cn/anaconda/cloud/bioconda/
				conda config --add channels https://mirrors.ustc.edu.cn/anaconda/cloud/menpo/
				conda config --add channels https://mirrors.ustc.edu.cn/anaconda/cloud/
			*/
			channelList := []string{
				"https://mirrors.ustc.edu.cn/anaconda/pkgs/main/",
				"https://mirrors.ustc.edu.cn/anaconda/pkgs/free/",
				"https://mirrors.ustc.edu.cn/anaconda/cloud/conda-forge/",
				"https://mirrors.ustc.edu.cn/anaconda/cloud/msys2/",
				"https://mirrors.ustc.edu.cn/anaconda/cloud/bioconda/",
				"https://mirrors.ustc.edu.cn/anaconda/cloud/menpo/",
				"https://mirrors.ustc.edu.cn/anaconda/cloud/",
			}
			for _, c := range channelList {
				gutils.ExecuteSysCommand(false, "", "conda", "config", "--add", "channels", c)
			}
		}
		installDir := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.Version)
		_, err := gutils.ExecuteSysCommand(
			false, "",
			"conda", "create",
			fmt.Sprintf("--prefix=%s", installDir),
			fmt.Sprintf("python=%s", c.Version),
		)
		if err == nil {
			// TODO: extract a method for $PATH
			symbolicPath := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.AppName)
			os.RemoveAll(symbolicPath)
			utils.SymbolicLink(installDir, symbolicPath)
			binPath := filepath.Join(symbolicPath, "bin")
			if runtime.GOOS == gutils.Windows {
				binPath = symbolicPath
			}
			if ok, _ := gutils.PathIsExist(binPath); ok {
				em := envs.NewEnvManager()
				defer em.CloseKey()
				em.AddToPath(binPath)
			}
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

func (c *CondaInstaller) SetVersion(version string) {
	c.Version = version
}

func (c *CondaInstaller) SearchVersion() {
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

func (c *CondaInstaller) Download() (zipFilePath string) {
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

func (c *CondaInstaller) Unzip(zipFilePath string) {}

func (c *CondaInstaller) Copy() {}

func (c *CondaInstaller) CreateVersionSymbol() {}

func (c *CondaInstaller) CreateBinarySymbol() {}

func (c *CondaInstaller) SetEnv() {}

func (c *CondaInstaller) GetInstall() func(appName, version, zipFilePath string) {
	return c.Install
}

func (c *CondaInstaller) InstallApp(zipFilePath string) {
	if c.Install != nil {
		c.Install(c.AppName, c.Version, zipFilePath)
	}
}

func (c *CondaInstaller) UnInstallApp() {
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

func (c *CondaInstaller) DeleteVersion() {}

func (c *CondaInstaller) DeleteAll() {
	if c.AppName == "" {
		return
	}
	vDir := conf.GetVMVersionsDir(c.AppName)
	symbolicPath := filepath.Join(vDir, c.AppName)
	binPath := filepath.Join(symbolicPath, "bin")
	if runtime.GOOS == gutils.Windows {
		binPath = symbolicPath
	}
	if ok, _ := gutils.PathIsExist(binPath); ok {
		em := envs.NewEnvManager()
		defer em.CloseKey()
		em.DeleteFromPath(binPath)
	}
	os.RemoveAll(vDir)
}

func (c *CondaInstaller) ClearCache() {}

func (c *CondaInstaller) GetHomepage() string {
	return c.HomePage
}
