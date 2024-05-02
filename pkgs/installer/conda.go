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

	"github.com/atotto/clipboard"
	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/envs"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/utils"
	"github.com/gvcgo/version-manager/pkgs/versions"
)

func IsMinicondaInstalled() bool {
	_, err := gutils.ExecuteSysCommand(true, "", "conda", "--help")
	return err == nil
}

type CondaInstallerType string

const (
	CondaPython CondaInstallerType = "python"
	CondaPyPy   CondaInstallerType = "pypy"
)

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

func NewCondaInstaller(t CondaInstallerType) *CondaInstaller {
	appName := "python"
	homePage := "https://anaconda.org/conda-forge/python/files"
	defaultVersion := "3.12.0"
	if t == CondaPyPy {
		appName = "pypy"
		homePage = "https://anaconda.org/conda-forge/pypy/files"
		defaultVersion = "3.9"
	}
	c := &CondaInstaller{
		AppName:  appName,
		Version:  defaultVersion,
		Searcher: NewSearcher(),
		HomePage: homePage,
	}
	c.Install = c.InstallPython
	c.UnInstall = c.UnInstallPython
	if t == CondaPyPy {
		c.Install = c.InstallPyPy
		c.UnInstall = c.UnInstallPyPy
	}
	return c
}

func (c *CondaInstaller) InstallPython(appName, version, zipFilePath string) {
	if c.V == nil {
		c.SearchVersion()
	}
	if c.V == nil {
		gprint.PrintError("Can't find version: %s", c.Version)
		return
	}
	if !IsMinicondaInstalled() {
		gprint.PrintWarning("No conda is installed. Please install miniconda first.")
		cmdStr := fmt.Sprintf("%s search %s", "vmr", "miniconda")
		if err := clipboard.WriteAll(cmdStr); err == nil {
			gprint.PrintInfo("Now you can use 'ctrl+v/cmd+v' to search versions for miniconda.")
		}
		os.Exit(1)
	}

	c.useMirrorInChina()

	installDir := c.getInstallDir()
	_, err := gutils.ExecuteSysCommand(
		false, "",
		"conda", "create",
		fmt.Sprintf("--prefix=%s", installDir),
		fmt.Sprintf("python=%s", c.Version),
	)
	if err == nil {
		c.NewPTY(installDir) // for session scope only.

		symbolicPath := c.getSymbolicPath()
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

func (c *CondaInstaller) UnInstallPython(appName, version string) {
	slink, _ := os.Readlink(c.getSymbolicPath())
	if filepath.Base(slink) == version {
		gprint.PrintWarning("Can not remove a version currently in use: %s", version)
		return
	}
	os.RemoveAll(c.getInstallDir())
}

func (c *CondaInstaller) InstallPyPy(appName, version, zipFilePath string) {
	if c.V == nil {
		c.SearchVersion()
	}
	if c.V == nil {
		gprint.PrintError("Can't find version: %s", c.Version)
		os.Exit(1)
	}
	if !IsMinicondaInstalled() {
		gprint.PrintWarning("No conda is installed. Please install miniconda first.")
		cmdStr := fmt.Sprintf("%s search %s", "vmr", "miniconda")
		if err := clipboard.WriteAll(cmdStr); err == nil {
			gprint.PrintInfo("Now you can use 'ctrl+v/cmd+v' to search versions for miniconda.")
		}
		os.Exit(1)
	}

	c.useMirrorInChina()

	installDir := c.getInstallDir()

	// conda create --prefix=~/.vm/versions/pypy_versions -c conda-forge pypy python=3.8
	_, err := gutils.ExecuteSysCommand(
		false, "",
		"conda", "create",
		fmt.Sprintf("--prefix=%s", installDir),
		"-c", "conda-forge", "pypy",
		fmt.Sprintf("python=%s", c.Version),
	)

	if err == nil {
		c.NewPTY(installDir) // for session scope only.

		symbolicPath := c.getSymbolicPath()
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

func (c *CondaInstaller) UnInstallPyPy(appName, version string) {
	slink, _ := os.Readlink(c.getSymbolicPath())
	if filepath.Base(slink) == version {
		gprint.PrintWarning("Can not remove a version currently in use: %s", version)
		return
	}

	os.RemoveAll(c.getInstallDir())
}

func (c *CondaInstaller) useMirrorInChina() {
	if conf.UseMirrorSiteInChina() {
		/*
			conda config --add channels https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/free/
			conda config --add channels https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/main/
			conda config --add channels https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/pytorch/
			conda config --add channels https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/pytorch/linux-64/
			conda config --set show_channel_urls yes
		*/
		channelList := []string{
			"https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/main/",
			"https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/free/",
			"https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/pytorch/",
			"https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/pytorch/linux-64/",
		}
		for _, c := range channelList {
			gutils.ExecuteSysCommand(false, "", "conda", "config", "--add", "channels", c)
		}
	}
}

func (c *CondaInstaller) SetVersion(version string) {
	c.Version = version
}

func (c *CondaInstaller) FixAppName() {}

func (c *CondaInstaller) SearchVersion() {
	if c.Searcher == nil {
		c.Searcher = NewSearcher()
	}
	vf := c.Searcher.GetVersions(c.AppName)
	vs := make([]string, 0)

	// accurate
	if _, ok := vf[c.Version]; ok {
		vs = append(vs, c.Version)
	}

	// fuzzy
	if len(vs) == 0 {
		for key := range vf {
			if strings.Contains(key, c.Version) {
				vs = append(vs, key)
			}
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

func (c *CondaInstaller) getSymbolicPath() string {
	pseudoAppName := "python"
	return filepath.Join(conf.GetVMVersionsDir(pseudoAppName), pseudoAppName)
}

func (c *CondaInstaller) getInstallDir() string {
	pseudoAppName := "python"
	if c.AppName == "pypy" {
		return filepath.Join(conf.GetVMVersionsDir(pseudoAppName), fmt.Sprintf("pypy%s", c.Version))
	}
	return filepath.Join(conf.GetVMVersionsDir(pseudoAppName), c.Version)
}

// Uses a version only in current session.
func (c *CondaInstaller) NewPTY(installDir string) {
	if gconv.Bool(os.Getenv(conf.VMOnlyInCurrentSessionEnvName)) {
		t := terminal.NewPtyTerminal(c.AppName)
		t.AddEnv("PATH", filepath.Join(installDir, "bin"))
		t.Run()
	}
}

func (c *CondaInstaller) Download() (zipFilePath string) {
	c.SearchVersion()
	if c.V != nil {
		symbolicPath := c.getSymbolicPath()
		installDir := c.getInstallDir()
		if ok, _ := gutils.PathIsExist(installDir); ok {
			c.NewPTY(installDir) // for session scope only.
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
	vDir := conf.GetVMVersionsDir("python")
	symbolicPath := c.getSymbolicPath()
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

func (c *CondaInstaller) SearchVersions() {
	if c.Searcher == nil {
		c.Searcher = NewSearcher()
	}
	c.Searcher.Search(c.AppName)
}
