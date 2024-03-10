package use

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var PHPInstaller = &installer.Installer{
	AppName:   "php",
	Version:   "php-8.3-latest",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"bin", "lib"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"php.ini"}
		}
		return r
	},
	BinDirGetter: func(version string) [][]string {
		r := [][]string{
			{"bin"},
		}
		if runtime.GOOS == gutils.Windows {
			r = [][]string{}
		}
		return r
	},
	PostInstall: func(appName, version string) {
		// Fix opcache extension problem.
		var (
			extPath     string
			phpInitFile string
		)
		phpDir := filepath.Join(conf.GetVMVersionsDir(appName), version)

		if runtime.GOOS == gutils.Windows {
			extPath = filepath.Join(phpDir, "ext", "php_opcache.dll")
			if ok, _ := gutils.PathIsExist(extPath); !ok {
				return
			}
			phpInitFile = filepath.Join(phpDir, "php.ini")
			if initFileContent, err := os.ReadFile(phpInitFile); err == nil {
				s := string(initFileContent)
				s = strings.ReplaceAll(s, "zend_extension=php_opcache.dll", fmt.Sprintf("zend_extension=%s", extPath))
				os.WriteFile(phpInitFile, []byte(s), os.ModePerm)
			}
			return
		}

		extPath = filepath.Join(phpDir, "lib", "php", "extensions")
		phpInitFile = filepath.Join(phpDir, "bin", "php.ini")
		dList, _ := os.ReadDir(extPath)
		for _, d := range dList {
			if d.IsDir() && strings.HasPrefix(d.Name(), "no-debug-zts-") {
				extPath = filepath.Join(extPath, d.Name(), "opcache.so")
				break
			}
		}
		if ok, _ := gutils.PathIsExist(extPath); !ok {
			return
		}
		if initFileContent, err := os.ReadFile(phpInitFile); err == nil {
			s := string(initFileContent)
			s = strings.ReplaceAll(s, "zend_extension=opcache.so", fmt.Sprintf("zend_extension=%s", extPath))
			os.WriteFile(phpInitFile, []byte(s), os.ModePerm)
		}
	},
	DUrlDecorator:   installer.DefaultDecorator,
	AddBinDirToPath: true,
}

func TestPHP() {
	zf := PHPInstaller.Download()
	PHPInstaller.Unzip(zf)
	PHPInstaller.Copy()
	PHPInstaller.CreateVersionSymbol()
	PHPInstaller.CreateBinarySymbol()
	PHPInstaller.SetEnv()
}
