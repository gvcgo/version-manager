package post

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/download"
)

func init() {
	RegisterPostInstallHandler(PHPSdkName, PostInstallForPHP)
}

const (
	PHPSdkName string = "php"
)

/*
post-installation handler for PHP.
*/
func PostInstallForPHP(versionName string, version download.Item) {
	if !strings.Contains(version.Url, "github.com") {
		return
	}

	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf("%s_versions", PHPSdkName))
	phpInstallDir := filepath.Join(d, fmt.Sprintf("%s-%s", PHPSdkName, versionName))
	if ok, _ := gutils.PathIsExist(phpInstallDir); !ok {
		return
	}
	var (
		extPath     string
		phpInitFile string
	)

	if runtime.GOOS == gutils.Windows {
		extPath = filepath.Join(phpInstallDir, "ext", "php_opcache.dll")
		if ok, _ := gutils.PathIsExist(extPath); !ok {
			return
		}
		phpInitFile = filepath.Join(phpInstallDir, "php.ini")
		if initFileContent, err := os.ReadFile(phpInitFile); err == nil {
			s := string(initFileContent)
			s = strings.ReplaceAll(s, "zend_extension=php_opcache.dll", fmt.Sprintf("zend_extension=%s", extPath))
			os.WriteFile(phpInitFile, []byte(s), os.ModePerm)
		}
		return
	}

	extPath = filepath.Join(phpInstallDir, "lib", "php", "extensions")
	phpInitFile = filepath.Join(phpInstallDir, "bin", "php.ini")
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
}
