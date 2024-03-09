package use

import (
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var JdkInstaller = &installer.Installer{
	AppName: "jdk",
	Version: "21.0.2_13",
	// Version:   "8u402-b06",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bin", "lib", "include"}
	},
	BinDirGetter: func(version string) [][]string {
		if strings.HasPrefix(version, "8u") {
			return [][]string{
				{"bin"},
				{"jre", "bin"},
			}
		}
		return [][]string{
			{"bin"},
		}
	},
	EnvGetter: func(appName, version string) []installer.Env {
		sep := ":"
		if runtime.GOOS == gutils.Windows {
			sep = ";"
		}
		javaHome := filepath.Join(conf.GetVMVersionsDir(appName), appName)
		classPath := strings.Join([]string{
			filepath.Join(javaHome, "lib", "tools.jar"),
			filepath.Join(javaHome, "lib", "dt.jar"),
			filepath.Join(javaHome, "lib", "jre", "rt.jar"),
		}, sep)
		if strings.HasPrefix(version, "8u") {
			return []installer.Env{
				{Name: "JAVA_HOME", Value: javaHome},
				{Name: "CLASSPATH", Value: classPath},
			}
		}
		return []installer.Env{
			{Name: "JAVA_HOME", Value: javaHome},
		}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	AddBinDirToPath:    true,
}

func TestJdk() {
	zf := JdkInstaller.Download()
	JdkInstaller.Unzip(zf)
	JdkInstaller.Copy()
	JdkInstaller.CreateVersionSymbol()
	JdkInstaller.CreateBinarySymbol()
	JdkInstaller.SetEnv()
}
