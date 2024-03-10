package multi

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

/*
Keeps multi versions.
*/
var VersionKeeper = map[string]*installer.Installer{}

var BunInstaller = &installer.Installer{
	AppName:   "bun",
	Version:   "1.0.9",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"bun"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"bun.exe"}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"bun"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"bun.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var DenoInstaller = &installer.Installer{
	AppName:   "deno",
	Version:   "1.41.1",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"deno"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"deno.exe"}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"deno"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"deno.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var FdInstaller = &installer.Installer{
	AppName:   "fd",
	Version:   "9.0.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"fd.1", "README.md"}
	},
	BinListGetter: func() []string {
		r := []string{"fd"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"fd.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var FlutterInstaller = &installer.Installer{
	AppName:   "flutter",
	Version:   "3.19.2",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"README.md", "LICENSE", "CODEOWNERS"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"dart", "flutter"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"dart.bat", "flutter.bat"}
		}
		return r
	},
	DUrlDecorator: func(dUrl string, ft *request.Fetcher) string {
		return strings.ReplaceAll(dUrl, "https://storage.googleapis.com", "https://storage.flutter-io.cn")
	},
	StoreMultiVersions: true,
}

var FzFInstaller = &installer.Installer{
	AppName:   "fzf",
	Version:   "0.46.1",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"fzf"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"fzf.exe"}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"fzf"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"fzf.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var GoInstaller = &installer.Installer{
	AppName:   "go",
	Version:   "1.22.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	FlagFileGetter: func() []string {
		return []string{"VERSION", "LICENSE"}
	},
	EnvGetter: func(appName, version string) []installer.Env {
		return []installer.Env{
			{Name: "GOROOT", Value: filepath.Join(conf.GetVMVersionsDir(appName), appName)},
		}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var GradleInstaller = &installer.Installer{
	AppName:   "gradle",
	Version:   "8.6",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	// DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

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

var JuliaInstaller = &installer.Installer{
	AppName:   "julia",
	Version:   "1.10.2",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE.md"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"julia"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"julia.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var KotlinInstaller = &installer.Installer{
	AppName:   "kotlin",
	Version:   "1.9.23",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bin", "tools", "klib"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	AddBinDirToPath:    true,
}

var MavenInstaller = &installer.Installer{
	AppName:   "maven",
	Version:   "3.9.6",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		return []string{"mvn"}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var NeovimInstaller = &installer.Installer{
	AppName:   "neovim",
	Version:   "0.9.5",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bin"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"nvim"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"nvim.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var NodejsInstaller = &installer.Installer{
	AppName:   "nodejs",
	Version:   "20.11.1",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE", "README.md"}
	},
	BinDirGetter: func(version string) [][]string {
		r := [][]string{{"bin"}}
		if runtime.GOOS == gutils.Windows {
			r = [][]string{}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"node", "npm", "npx", "corepack"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"node.exe", "npm.cmd", "npx.cm", "corepack.cmd"}
		}
		return r
	},
	StoreMultiVersions: true,
}

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

var ProtobufInstaller = &installer.Installer{
	AppName:   "protobuf",
	Version:   "25.3",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bin"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"protoc"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"protoc.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var RipgrepInstaller = &installer.Installer{
	AppName:   "ripgrep",
	Version:   "14.1.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"rg"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"rg.exe"}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"rg"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"rg.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var TypstLspInstaller = &installer.Installer{
	AppName:        "typst-lsp",
	Version:        "0.12.1",
	Fetcher:        conf.GetFetcher(),
	IsZipFile:      false,
	BinaryRenameTo: "typst-lsp",
	FlagFileGetter: func() []string {
		r := []string{"typst-lsp"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"typst-lsp.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
}

var TypstInstaller = &installer.Installer{
	AppName:   "typst",
	Version:   "0.10.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"LICENSE"}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"typst"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"typst.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
}

var VlangLspInstaller = &installer.Installer{
	AppName:   "v-analyzer",
	Version:   "0.0.3",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"v-analyzer"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"v-analyzer.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var VlangInstaller = &installer.Installer{
	AppName:   "v",
	Version:   "0.4.4",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"cmd", "v"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"cmd", "v.exe"}
		}
		return r
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{},
			{"cmd", "tools"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"v", "vdoctor", "vup"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"v.exe", "vdoctor.exe", "vup.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var ZigLspInstaller = &installer.Installer{
	AppName:   "zls",
	Version:   "0.11.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"README.md"}
	},
	BinDirGetter: func(version string) [][]string {
		if strings.HasPrefix(version, "0.1.") || strings.HasPrefix(version, "0.2.") {
			return [][]string{}
		}
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"zls"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"zls.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

var ZigInstaller = &installer.Installer{
	AppName:   "zig",
	Version:   "0.11.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE"}
	},
	BinListGetter: func() []string {
		r := []string{"zig"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"zig.exe"}
		}
		return r
	},
	StoreMultiVersions: true,
}

func init() {
	VersionKeeper["bun"] = BunInstaller
	VersionKeeper["deno"] = DenoInstaller
	VersionKeeper["fd"] = FdInstaller
	VersionKeeper["flutter"] = FlutterInstaller
	VersionKeeper["fzf"] = FzFInstaller
	VersionKeeper["go"] = GoInstaller
	VersionKeeper["gradle"] = GradleInstaller
	VersionKeeper["jdk"] = JdkInstaller
	VersionKeeper["julia"] = JuliaInstaller
	VersionKeeper["kotlin"] = KotlinInstaller
	VersionKeeper["maven"] = MavenInstaller
	VersionKeeper["neovim"] = NeovimInstaller
	VersionKeeper["node"] = NodejsInstaller
	VersionKeeper["php"] = PHPInstaller
	VersionKeeper["protobuf"] = ProtobufInstaller
	VersionKeeper["ripgrep"] = RipgrepInstaller
	VersionKeeper["typst-lsp"] = TypstLspInstaller
	VersionKeeper["typst"] = TypstInstaller
	VersionKeeper["v-analyzer"] = VlangLspInstaller
	VersionKeeper["v"] = VlangInstaller
	VersionKeeper["zls"] = ZigLspInstaller
	VersionKeeper["zig"] = ZigInstaller
}

func RunInstaller(ins *installer.Installer) {
	zf := ins.Download()
	ins.Unzip(zf)
	ins.Copy()
	ins.CreateVersionSymbol()
	ins.CreateBinarySymbol()
	ins.SetEnv()
}
