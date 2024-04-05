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
	"io/fs"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/archiver"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/envs"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/pkgs/conf"
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
	VersionFilter      func(dUrl string) bool
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

	// search by full name.
	if v, ok := vf[i.Version]; ok {
		i.V = &v[0]
		return
	}

	// search by keywords.
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
	var (
		v     versions.VersionList
		ok    bool
		vName string = "latest"
	)
	v, ok = vf[vName]
	if !ok {
		// Get the first item.
		for vName, v = range vf {
			break
		}
	}

	if i.VersionFilter != nil {
		for _, vv := range v {
			if i.VersionFilter(vv.Url) {
				i.V = &vv
				break
			}
		}
	} else {
		i.V = &v[0]
	}
	i.Version = vName
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
		i.NewPty(versionPath) // for session scope only.

		i.CreateVersionSymbol()
		i.CreateBinarySymbol()
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
			if zipFilePath != "" {
				os.RemoveAll(zipFilePath) // checksum failed.
			}
			zipFilePath = ""
		}
	}
	if zipFilePath == "" && i.Install == nil {
		gprint.PrintError("Failed to download file: %s", i.V.Url)
		os.Exit(1)
	}
	return
}

func handleUnzipFailedError(zipFilePath string, err error) {
	if err == nil {
		return
	}
	gprint.PrintError("Failed to unzip file: %s, %+v", zipFilePath, err)
	if zipFilePath != "" {
		os.RemoveAll(zipFilePath)
	}
}

func (i *Installer) filterRenameBinary(binName string) bool {
	if strings.Contains(binName, i.BinaryRenameTo) {
		return true
	}
	if strings.Contains(binName, "msys2") && strings.HasSuffix(binName, ".exe") {
		return true
	}
	if strings.Contains(binName, "setup") && strings.HasSuffix(binName, ".exe") {
		return true
	}
	return false
}

// use archiver or xtract.
func useArchiver(zipFilePath string) bool {
	if strings.HasSuffix(zipFilePath, ".gz") && !strings.HasSuffix(zipFilePath, ".tar.gz") {
		return false
	}
	if strings.HasSuffix(zipFilePath, ".7z") {
		return false
	}
	return true
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
		gprint.PrintInfo("Unarchiving files, please wait...")

		if arch, err := archiver.NewArchiver(zipFilePath, tempDir, useArchiver(zipFilePath)); err == nil {
			_, err = arch.UnArchive()
			if err != nil && runtime.GOOS == gutils.Windows && strings.HasSuffix(zipFilePath, ".zip") {
				err = utils.UnzipForWindows(zipFilePath, tempDir)
			}
			handleUnzipFailedError(zipFilePath, err)
			return
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
		if i.filterRenameBinary(binName) {
			newName := i.BinaryRenameTo
			if runtime.GOOS == gutils.Windows {
				newName = i.BinaryRenameTo + ".exe"
			}

			os.MkdirAll(conf.GetVMTempDir(), os.ModePerm)
			newPath := filepath.Join(conf.GetVMTempDir(), newName)
			// copy and rename binary file to tmp dir.
			if err := gutils.CopyAFile(zipFilePath, newPath); err != nil {
				gprint.PrintError("Copy file %x to tmp dir failed: %+v", zipFilePath, err)
				os.Exit(1)
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
	i.NewPty(versionPath) // only for session scope.

	symbolPath := filepath.Join(conf.GetVMVersionsDir(i.AppName), i.AppName)

	if ok, _ := gutils.PathIsExist(versionPath); ok {
		// remove old symbol
		if ok, _ := gutils.PathIsExist(symbolPath); ok {
			os.RemoveAll(symbolPath)
		}
		// create symbolic
		utils.SymbolicLink(versionPath, symbolPath)
	}

	// Adds binary dir to $PATH env directly.
	// Or symbolics for binaries will be created.
	if i.AddBinDirToPath {
		pathValue := i.preparePathValue(symbolPath)
		if pathValue != "" {
			em := envs.NewEnvManager()
			defer em.CloseKey()
			em.AddToPath(pathValue)
		}
		return
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

// Uses a version only in current session.
func (i *Installer) NewPty(installDir string) {
	PathDirs := []string{}
	if i.AddBinDirToPath {
		PathDirs = append(PathDirs, i.preparePathValue(installDir))
	} else if i.BinDirGetter != nil && len(i.BinDirGetter(i.Version)) > 0 {
		for _, bDir := range i.BinDirGetter(i.Version) {
			if len(bDir) == 0 {
				PathDirs = append(PathDirs, installDir)
			} else {
				d := filepath.Join(installDir, filepath.Join(bDir...))
				ok := false
				if dList, err := os.ReadDir(d); err == nil {
				INNER:
					for _, dd := range dList {
						if !dd.IsDir() && i.createSymbolicOrNot(dd.Name()) {
							ok = true
							break INNER
						}
					}
				}
				if ok {
					PathDirs = append(PathDirs, d)
				}
			}
		}
	} else {
		PathDirs = append(PathDirs, installDir)
	}

	if gconv.Bool(os.Getenv(conf.VMOnlyInCurrentSessionEnvName)) {
		t := terminal.NewPtyTerminal(i.AppName)
		for _, pStr := range PathDirs {
			t.AddEnv("PATH", pStr)
		}
		if i.EnvGetter != nil {
			for _, env := range i.EnvGetter(i.AppName, i.Version) {
				t.AddEnv(env.Name, env.Value)
			}
		}
		t.Run()
	}
}

func (i *Installer) CreateBinarySymbol() {
	if i.AddBinDirToPath {
		// BinDirs are added to $PATH
		// Do not create symbolics in .vm/bin any more.
		return
	}
	versionPath := filepath.Join(conf.GetVMVersionsDir(i.AppName), i.Version)
	i.NewPty(versionPath) // only for session scope.

	currentPath := filepath.Join(conf.GetVMVersionsDir(i.AppName), i.AppName)
	if ok, _ := gutils.PathIsExist(currentPath); !ok {
		return
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
	defer em.CloseKey()
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
	i.SearchVersion()
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
	defer em.CloseKey()
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
				defer em.CloseKey()
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
