package installer

import (
	"io/fs"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/archiver"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/envs"
	"github.com/gvcgo/version-manager/pkgs/utils"
	"github.com/gvcgo/version-manager/pkgs/versions"
)

const (
	SymbolicsInfoFileName string = "symbolics.info"
)

type Env struct {
	Name  string
	Value string
}

var DefaultDecorator = func(dUrl string, ft *request.Fetcher) string {
	// proxy
	pxy := os.Getenv(conf.VMProxyEnvName)
	if gutils.VerifyUrls(pxy) || strings.Contains(pxy, "://") {
		if ft != nil {
			ft.Proxy = pxy
		}
		return dUrl
	}
	return conf.DecorateUrl(dUrl)
}

type Installer struct {
	AppName            string
	Version            string
	Searcher           *Searcher
	Fetcher            *request.Fetcher
	V                  *versions.VersionItem
	IsZipFile          bool
	BinaryRenameTo     string
	BinDirGetter       func(version string) [][]string               // Binary dir
	BinListGetter      func() []string                               // Binaries
	FlagFileGetter     func() []string                               // Flags to find home dir of an app
	FlagDirExcepted    bool                                          // whether to find binaries only
	EnvGetter          func(appName, version string) []Env           // Envs to set
	DUrlDecorator      func(dUrl string, ft *request.Fetcher) string // Download url decorator
	PostInstall        func(appName, version string)                 // post install hook
	Install            func(appName, version, zipFileName string)    // customed installation.
	UnInstall          func(appName, version string)                 // customed uninstallation.
	StoreMultiVersions bool                                          // installs only the latest version if false
	ForceReDownload    bool                                          // force to redownload the cached zip file
	AddBinDirToPath    bool                                          // uses $PATH instead of creating symbolics
	NoDownload         bool                                          // diable download
	HomePage           string                                        // home page of the app
}

func NewInstaller(appName, version string) (i *Installer) {
	i = &Installer{
		AppName:  appName,
		Version:  version,
		Searcher: NewSearcher(),
		Fetcher:  conf.GetFetcher(),
	}
	return
}
func (i *Installer) SetVersion(version string) {
	if !i.StoreMultiVersions && version != "all" {
		// the latest version only.
		i.Version = "latest"
		return
	}
	i.Version = version
}

// Searches version files for an application.
func (i *Installer) SearchVersion() {
	if i.Searcher == nil {
		i.Searcher = NewSearcher()
	}
	vf := i.Searcher.GetVersions(i.AppName)
	vs := make([]string, 0)
	for key := range vf {
		if strings.Contains(key, i.Version) {
			vs = append(vs, key)
		}
	}

	if len(vs) == 0 {
		i.V = nil
		gprint.PrintError("Cannot find version: %s", i.Version)
	} else if len(vs) == 1 {
		i.Version = vs[0]
		i.V = &vf[i.Version][0]
	} else {
		i.V = nil
		gprint.PrintError("Found multiple versions: \n%v", strings.Join(vs, "\n"))
	}
}

func (i *Installer) SearchLatestVersion() {
	if i.Searcher == nil {
		i.Searcher = NewSearcher()
	}
	vf := i.Searcher.GetVersions(i.AppName)
	if v, ok := vf["latest"]; ok {
		i.Version = "latest"
		i.V = &v[0]
		return
	}
	for vName, v := range vf {
		i.V = &v[0]
		i.Version = vName
		return
	}
}

func (i *Installer) Download() (zipFilePath string) {
	if i.Fetcher == nil {
		i.Fetcher = conf.GetFetcher()
	}
	if i.NoDownload {
		return
	}
	if i.StoreMultiVersions {
		i.SearchVersion()
	} else {
		i.SearchLatestVersion()
	}

	if i.V == nil {
		return
	}

	// if already installed, switch to the specified version.
	versionPath := filepath.Join(conf.GetVMVersionsDir(i.AppName), i.Version)
	if ok, _ := gutils.PathIsExist(versionPath); ok {
		i.CreateVersionSymbol()
		gprint.PrintSuccess("Switched to %s", i.Version)
		os.Exit(0)
	}

	// install new version.
	zipDir := conf.GetZipFileDir(i.AppName)
	if ok, _ := gutils.PathIsExist(zipDir); !ok {
		if err := os.MkdirAll(zipDir, os.ModePerm); err != nil {
			gprint.PrintError("Failed to create directory: %s", zipDir)
			return
		}
	}

	if i.DUrlDecorator != nil {
		i.Fetcher.SetUrl(i.DUrlDecorator(i.V.Url, i.Fetcher))
	} else {
		i.Fetcher.SetUrl(i.V.Url)
	}
	zipFilePath = filepath.Join(zipDir, filepath.Base(i.V.Url))
	i.Fetcher.GetAndSaveFile(zipFilePath, i.ForceReDownload)

	// checksum
	if i.V.Sum != "" && i.V.SumType != "" {
		if ok := gutils.CheckSum(zipFilePath, strings.TrimSpace(i.V.SumType), strings.TrimSpace(i.V.Sum)); !ok {
			zipFilePath = ""
			os.RemoveAll(zipFilePath) // checksum failed.
		}
	}
	if zipFilePath == "" && i.Install == nil {
		gprint.PrintError("Failed to download file: %s", i.V.Url)
		os.Exit(1)
	}
	return
}

func handleUnzipFailedError(zipFilePath string, err error) {
	gprint.PrintError("Failed to unzip file: %s, %+v", zipFilePath, err)
	os.RemoveAll(zipFilePath)
}

func (i *Installer) Unzip(zipFilePath string) {
	if zipFilePath == "" {
		return
	}

	if i.IsZipFile {
		// rename PortableGit zip file.
		if strings.HasSuffix(zipFilePath, ".7z.exe") {
			newPath := strings.ReplaceAll(zipFilePath, ".7z.exe", ".7z")
			if err := os.Rename(zipFilePath, newPath); err == nil {
				zipFilePath = newPath
			}
		}

		tempDir := conf.GetVMTempDir()
		// use archiver.
		useArchiver := true
		if strings.HasSuffix(zipFilePath, ".gz") && !strings.HasSuffix(zipFilePath, ".tar.gz") {
			useArchiver = false
		}
		if arch, err := archiver.NewArchiver(zipFilePath, tempDir, useArchiver); err == nil {
			_, err = arch.UnArchive()
			if err != nil {
				handleUnzipFailedError(zipFilePath, err)
				return
			}
		} else {
			handleUnzipFailedError(zipFilePath, err)
		}

		// Rename binary in temp dir.
		if i.BinaryRenameTo != "" {
			dList, _ := os.ReadDir(tempDir)
			for _, d := range dList {
				if !d.IsDir() && strings.Contains(d.Name(), i.BinaryRenameTo) {
					os.Rename(filepath.Join(tempDir, d.Name()), filepath.Join(tempDir, i.BinaryRenameTo))
				}
			}
		}
	} else if !i.IsZipFile && i.BinaryRenameTo != "" {
		binName := filepath.Base(zipFilePath)
		if strings.Contains(binName, i.BinaryRenameTo) {
			newName := i.BinaryRenameTo
			if runtime.GOOS == gutils.Windows {
				newName = i.BinaryRenameTo + ".exe"
			}

			os.MkdirAll(conf.GetVMTempDir(), os.ModePerm)
			newPath := filepath.Join(conf.GetVMTempDir(), newName)
			// copy and rename binary file to tmp dir.
			if err := gutils.CopyAFile(zipFilePath, newPath); err != nil {
				gprint.PrintError("Copy file %x to tmp dir failed: %+v", zipFilePath, err)
			}
			if runtime.GOOS != gutils.Windows {
				// add previlledge for exectution.
				gutils.ExecuteSysCommand(false, "", "chmod", "+x", newPath)
			}
		}
	}
}

func (i *Installer) Copy() {
	// find directory to copy.
	if i.FlagFileGetter != nil {
		f := NewFinder(i.FlagFileGetter()...)
		f.ExceptDir = i.FlagDirExcepted
		f.Find(conf.GetVMTempDir())

		if f.Home != "" {
			err := gutils.CopyDirectory(f.Home, filepath.Join(conf.GetVMVersionsDir(i.AppName), i.Version), true)
			if err != nil {
				gprint.PrintError("Failed to copy directory: %s, %+v", f.Home, err)
			}
		}
	}
	os.RemoveAll(conf.GetVMTempDir())
}

func (i *Installer) CreateVersionSymbol() {
	versionPath := filepath.Join(conf.GetVMVersionsDir(i.AppName), i.Version)
	symbolPath := filepath.Join(conf.GetVMVersionsDir(i.AppName), i.AppName)

	if ok, _ := gutils.PathIsExist(versionPath); ok {
		// remove old symbol
		if ok, _ := gutils.PathIsExist(symbolPath); ok {
			os.RemoveAll(symbolPath)
		}
		// create symbolic
		utils.SymbolicLink(versionPath, symbolPath)
	}
}

func (i *Installer) removeOldSymbolic() {
	infoFile := filepath.Join(conf.GetVMVersionsDir(i.AppName), SymbolicsInfoFileName)
	content, _ := os.ReadFile(infoFile)
	if len(content) > 0 {
		sList := strings.Split(string(content), "\n")
		binDir := conf.GetAppBinDir()
		for _, symbolic := range sList {
			os.RemoveAll(filepath.Join(binDir, symbolic))
		}
	}
	os.RemoveAll(infoFile)
}

func (i *Installer) saveSymbolicInfo(symbolic string) {
	infoFile := filepath.Join(conf.GetVMVersionsDir(i.AppName), SymbolicsInfoFileName)
	content, _ := os.ReadFile(infoFile)
	data := string(content)
	if data == "" {
		data = symbolic
	} else {
		data = data + "\n" + symbolic
	}
	os.WriteFile(infoFile, []byte(data), os.ModePerm)
}

func (i *Installer) createSymbolicOrNot(fileName string) bool {
	if i.BinListGetter == nil || len(i.BinListGetter()) == 0 {
		return true
	}
	for _, binName := range i.BinListGetter() {
		if binName == fileName {
			return true
		}
	}
	return false
}

func (i *Installer) CreateBinarySymbol() {
	currentPath := filepath.Join(conf.GetVMVersionsDir(i.AppName), i.AppName)
	if ok, _ := gutils.PathIsExist(currentPath); !ok {
		return
	}
	// Adds binary dir to $PATH env directly.
	if i.AddBinDirToPath {
		pathValue := i.preparePathValue(currentPath)
		if pathValue != "" {
			em := envs.NewEnvManager()
			em.AddToPath(pathValue)
		}
		return // Do not create symbolics in .vm/bin any more.
	}

	// Or creates symbolics in .vm/bin/
	i.removeOldSymbolic()
	if i.BinDirGetter != nil && len(i.BinDirGetter(i.Version)) > 0 {
		for _, bDir := range i.BinDirGetter(i.Version) {
			if len(bDir) == 0 {
				i.createBinarySymbolForCurrentDir(currentPath)
			} else {
				d := filepath.Join(currentPath, filepath.Join(bDir...))
				if dList, err := os.ReadDir(d); err == nil {
					for _, dd := range dList {
						if !dd.IsDir() && i.createSymbolicOrNot(dd.Name()) {
							fPath := filepath.Join(d, dd.Name())
							if runtime.GOOS != gutils.Windows {
								// add previlledge for exectution.
								gutils.ExecuteSysCommand(false, "", "chmod", "+x", fPath)
							}
							symPath := filepath.Join(conf.GetAppBinDir(), dd.Name())
							utils.SymbolicLink(fPath, symPath)
							i.saveSymbolicInfo(dd.Name())
							// extra symbolic.
							i.createExtraSymbolic(dd, fPath)
						}
					}
				}
			}
		}
	} else {
		i.createBinarySymbolForCurrentDir(currentPath)
	}
}

func (i *Installer) createExtraSymbolic(dd fs.DirEntry, fPath string) {
	// special: create bunx for bun.
	if dd.Name() == "bun" {
		extraSymbol := "bunx"
		symPath := filepath.Join(conf.GetAppBinDir(), extraSymbol)
		utils.SymbolicLink(fPath, symPath)
		i.saveSymbolicInfo(extraSymbol)
	} else if dd.Name() == "bun.exe" {
		extraSymbol := "bunx.exe"
		symPath := filepath.Join(conf.GetAppBinDir(), extraSymbol)
		utils.SymbolicLink(fPath, symPath)
		i.saveSymbolicInfo(extraSymbol)
	}
}

func (i *Installer) preparePathValue(currentPath string) (pathValue string) {
	if i.BinDirGetter == nil {
		pathValue = currentPath
	} else {
		pathList := []string{}
		bdList := i.BinDirGetter(i.Version)

		if len(bdList) == 0 {
			pathList = append(pathList, currentPath)
		} else {
			for _, d := range bdList {
				if len(d) == 0 {
					pathList = append(pathList, currentPath)
				} else {
					pathList = append(pathList, filepath.Join(currentPath, filepath.Join(d...)))
				}
			}
		}
		// join multi path value
		sep := ":"
		if runtime.GOOS == gutils.Windows {
			sep = ";"
		}
		pathValue = strings.Join(pathList, sep)
	}
	return
}

func (i *Installer) createBinarySymbolForCurrentDir(currentPath string) {
	dList, _ := os.ReadDir(currentPath)
	for _, dd := range dList {
		if !dd.IsDir() && i.createSymbolicOrNot(dd.Name()) {
			fPath := filepath.Join(currentPath, dd.Name())
			if runtime.GOOS != gutils.Windows {
				// add previlledge for exectution.
				gutils.ExecuteSysCommand(false, "", "chmod", "+x", fPath)
			}
			symPath := filepath.Join(conf.GetAppBinDir(), dd.Name())
			utils.SymbolicLink(fPath, symPath)
			i.saveSymbolicInfo(dd.Name())
			// extra symbolic.
			i.createExtraSymbolic(dd, fPath)
		}
	}
}

func (i *Installer) SetEnv() {
	em := envs.NewEnvManager()
	if i.EnvGetter != nil {
		for _, env := range i.EnvGetter(i.AppName, i.Version) {
			em.Set(env.Name, env.Value)
		}
	}
	em.SetPath()

	// PostInstall
	if i.PostInstall != nil {
		i.PostInstall(i.AppName, i.Version)
	}
}

func (i *Installer) GetInstall() func(appName, version, zipFileName string) {
	return i.Install
}

// customed installation.
func (i *Installer) InstallApp(zipFilePath string) {
	if i.Install != nil {
		i.Install(i.AppName, i.Version, zipFilePath)
	}
}

// uninstall.
func (i *Installer) UnInstallApp() {
	if i.AppName == "" {
		return
	}
	if i.Version == "all" {
		i.DeleteAll()
	} else if !i.StoreMultiVersions && i.UnInstall != nil {
		i.UnInstall(i.AppName, i.Version)
	} else {
		i.DeleteVersion()
	}
}

// Removes a version.
func (i *Installer) DeleteVersion() {
	// whether in use or not.
	vDir := conf.GetVMVersionsDir(i.AppName)
	if dest, err := os.Readlink(filepath.Join(vDir, i.AppName)); err == nil {
		version := filepath.Base(dest)
		if version == i.Version {
			gprint.PrintWarning("version %s is currently in use.", version)
			return
		}
	}

	// remove a version
	versionDir := filepath.Join(vDir, i.Version)
	if err := os.RemoveAll(versionDir); err != nil {
		gprint.PrintError("failed to remove version %s: %+v", i.Version, err)
	}
}

// Removes all installed versions of an app.
func (i *Installer) DeleteAll() {
	if i.AppName == "" {
		return
	}
	// delete symbolics.
	infoFile := filepath.Join(conf.GetVMVersionsDir(i.AppName), SymbolicsInfoFileName)
	data, _ := os.ReadFile(infoFile)
	binDir := conf.GetAppBinDir()
	for _, symbolicName := range strings.Split(string(data), "\n") {
		symbolicPath := filepath.Join(binDir, symbolicName)
		if ok, _ := gutils.PathIsExist(symbolicPath); ok {
			os.Remove(symbolicPath)
		}
	}

	// delete version dir.
	vDir := conf.GetVMVersionsDir(i.AppName)
	os.RemoveAll(vDir)

	// delete env
	em := envs.NewEnvManager()
	if i.EnvGetter != nil {
		if i.AppName == "jdk" {
			// handle jdk8.
			for _, env := range i.EnvGetter(i.AppName, "8u") {
				em.UnSet(env.Name)
			}
			for _, env := range i.EnvGetter(i.AppName, "all") {
				em.UnSet(env.Name)
			}
		} else {
			for _, env := range i.EnvGetter(i.AppName, i.Version) {
				em.UnSet(env.Name)
			}
		}
	}
	if i.AddBinDirToPath {
		versionList := []string{i.Version}
		// handle jdk8.
		if i.AppName == "jdk" {
			versionList = []string{"8u", "all"}
		}
		for _, version := range versionList {
			i.Version = version
			pathValue := i.preparePathValue(filepath.Join(conf.GetVMVersionsDir(i.AppName), i.AppName))
			// fmt.Println("pathValue: ", pathValue)
			if pathValue != "" {
				em := envs.NewEnvManager()
				em.DeleteFromPath(pathValue)
			}
		}
	}
}

func (i *Installer) ClearCache() {
	os.RemoveAll(conf.GetZipFileDir(i.AppName))
}

func (i *Installer) GetHomepage() string {
	return i.HomePage
}
