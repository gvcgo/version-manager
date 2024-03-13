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
			return
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
			symbolicPath := filepath.Join(conf.GetVMVersionsDir(c.AppName), c.AppName)
			utils.SymbolicLink(installDir, symbolicPath)
			binPath := filepath.Join(symbolicPath, "bin")
			if ok, _ := gutils.PathIsExist(binPath); ok {
				em := envs.NewEnvManager()
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

		versionDir := filepath.Join(conf.GetVMVersionsDir(c.AppName), version)
		os.RemoveAll(versionDir)
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
	if ok, _ := gutils.PathIsExist(binPath); ok {
		em := envs.NewEnvManager()
		em.DeleteFromPath(binPath)
	}
	os.RemoveAll(vDir)
}

func (c *CondaInstaller) ClearCache() {}

func (c *CondaInstaller) GetHomepage() string {
	return c.HomePage
}
