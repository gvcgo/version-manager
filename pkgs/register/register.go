package register

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/envs"
	"github.com/gvcgo/version-manager/pkgs/installer"
	"github.com/gvcgo/version-manager/pkgs/utils"
)

// TODO: test for windows and linux.
type VersionManager interface {
	Download() (zipFilePath string)
	Unzip(zipFilePath string)
	Copy()
	CreateVersionSymbol()
	CreateBinarySymbol()
	SetEnv()
	GetInstall() func(appName, version, zipFilePath string)
	InstallApp(zipFilePath string)
	UnInstallApp()
	DeleteVersion()
	DeleteAll()
	ClearCache()
	GetHomepage() string
	SetVersion(version string)
}

/*
Keeps multi versions.
*/
var VersionKeeper = map[string]VersionManager{}

var BunInstaller = &installer.Installer{
	AppName:   "bun",
	Version:   "1.0.9",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"bun"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"bun.exe"}
		}
		return r
	},
	FlagDirExcepted: true,
	BinListGetter: func() []string {
		r := []string{"bun"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"bun.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
	HomePage:           "https://bun.sh",
}

var CoursierInstaller = &installer.Installer{
	AppName:   "coursier",
	Version:   "2.1.9",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"cs"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"cs.exe"}
		}
		return r
	},
	FlagDirExcepted: true,
	BinListGetter: func() []string {
		r := []string{"cs"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"cs.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
	HomePage:           "https://get-coursier.io/",
}

var DenoInstaller = &installer.Installer{
	AppName:   "deno",
	Version:   "1.41.1",
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
	ForceReDownload:    true,
	HomePage:           "https://deno.com/",
}

var DotNetInstaller = &installer.Installer{
	AppName:         "dotnet",
	Version:         "8.0.202",
	IsZipFile:       true,
	AddBinDirToPath: true,
	FlagFileGetter: func() []string {
		return []string{"shared", "sdk", "host"}
	},
	BinListGetter: func() []string {
		if runtime.GOOS == gutils.Windows {
			return []string{"dotnet.exe"}
		}
		return []string{"dotnet"}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	EnvGetter: func(appName, version string) []installer.Env {
		return []installer.Env{
			{Name: "DOTNET_ROOT", Value: filepath.Join(conf.GetVMVersionsDir(appName), appName)},
		}
	},
	HomePage: "https://dotnet.microsoft.com/",
}

var FdInstaller = &installer.Installer{
	AppName:   "fd",
	Version:   "9.0.0",
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
	HomePage:           "https://github.com/sharkdp/fd",
}

var FlutterInstaller = &installer.Installer{
	AppName:   "flutter",
	Version:   "3.19.2",
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
		if conf.UseMirrorSiteInChina() {
			return strings.ReplaceAll(dUrl, "https://storage.googleapis.com", "https://storage.flutter-io.cn")
		}
		return dUrl
	},
	AddBinDirToPath:    true,
	StoreMultiVersions: true,
	HomePage:           "https://flutter.dev/",
}

var FzFInstaller = &installer.Installer{
	AppName:   "fzf",
	Version:   "0.46.1",
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
	HomePage:           "https://github.com/junegunn/fzf",
}

var GleamInstaller = &installer.Installer{
	AppName:   "gleam",
	Version:   "1.0.0",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"gleam"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"gleam.exe"}
		}
		return r
	},
	FlagDirExcepted: true,
	BinListGetter: func() []string {
		r := []string{"gleam"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"gleam.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	HomePage:           "https://gleam.run/",
}

var GlowInstaller = &installer.Installer{
	AppName:   "glow",
	Version:   "1.5.1",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE"}
	},
	FlagDirExcepted: true,
	BinListGetter: func() []string {
		r := []string{"glow"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"glow.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
	HomePage:           "https://github.com/charmbracelet/glow",
}

var GoInstaller = &installer.Installer{
	AppName:   "go",
	Version:   "1.22.0",
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
	DUrlDecorator: func(dUrl string, ft *request.Fetcher) string {
		if conf.UseMirrorSiteInChina() {
			return strings.ReplaceAll(dUrl, "https://go.dev/dl/", "https://golang.google.cn/dl/")
		}
		return installer.DefaultDecorator(dUrl, ft)
	},
	StoreMultiVersions: true,
	HomePage:           "https://go.dev/",
}

var GradleInstaller = &installer.Installer{
	AppName:   "gradle",
	Version:   "8.6",
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
	DUrlDecorator: func(dUrl string, ft *request.Fetcher) string {
		if conf.UseMirrorSiteInChina() {
			return strings.ReplaceAll(dUrl, "https://services.gradle.org/distributions/", "https://mirrors.cloud.tencent.com/gradle/")
		}
		return installer.DefaultDecorator(dUrl, ft)
	},
	StoreMultiVersions: true,
	HomePage:           "https://gradle.org/",
}

var JdkInstaller = &installer.Installer{
	AppName: "jdk",
	Version: "21.0.2_13",
	// Version:   "8u402-b06",
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
	HomePage:           "https://adoptium.net/",
}

var JuliaInstaller = &installer.Installer{
	AppName:   "julia",
	Version:   "1.10.2",
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
	DUrlDecorator: func(dUrl string, ft *request.Fetcher) string {
		if conf.UseMirrorSiteInChina() {
			return strings.ReplaceAll(dUrl, "https://julialang-s3.julialang.org/", "https://mirrors.nju.edu.cn/julia-releases/")
		}
		return dUrl
	},
	StoreMultiVersions: true,
	HomePage:           "https://julialang.org/",
}

var KotlinInstaller = &installer.Installer{
	AppName:   "kotlin",
	Version:   "1.9.23",
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
	HomePage:           "https://kotlinlang.org/",
}

var LazyGitInstaller = &installer.Installer{
	AppName:   "lazygit",
	Version:   "0.40.2",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE"}
	},
	FlagDirExcepted: true,
	BinListGetter: func() []string {
		r := []string{"lazygit"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"lazygit.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	HomePage:           "https://github.com/jesseduffield/lazygit",
}

var MavenInstaller = &installer.Installer{
	AppName:   "maven",
	Version:   "3.9.6",
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
	DUrlDecorator: func(dUrl string, ft *request.Fetcher) string {
		if conf.UseMirrorSiteInChina() {
			return strings.ReplaceAll(dUrl, "https://dlcdn.apache.org/maven/", "https://mirrors.aliyun.com/apache/maven/")
		}
		return installer.DefaultDecorator(dUrl, ft)
	},
	StoreMultiVersions: true,
	HomePage:           "https://maven.apache.org/",
}

var NeovimInstaller = &installer.Installer{
	AppName:   "neovim",
	Version:   "0.9.5",
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
	ForceReDownload:    true,
	HomePage:           "https://neovim.io/",
}

var NodejsInstaller = &installer.Installer{
	AppName:   "nodejs",
	Version:   "20.11.1",
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
	DUrlDecorator: func(dUrl string, ft *request.Fetcher) string {
		if conf.UseMirrorSiteInChina() {
			return strings.ReplaceAll(dUrl, "https://nodejs.org/download/release/", "https://mirrors.tuna.tsinghua.edu.cn/nodejs-release/")
		}
		return dUrl
	},
	StoreMultiVersions: true,
	AddBinDirToPath:    true,
	HomePage:           "https://nodejs.org/en",
}

var PHPInstaller = &installer.Installer{
	AppName:   "php",
	Version:   "php-8.3-latest",
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
	DUrlDecorator:      installer.DefaultDecorator,
	AddBinDirToPath:    true,
	StoreMultiVersions: true,
	HomePage:           "https://github.com/pmmp/PHP-Binaries",
}

var ProtobufInstaller = &installer.Installer{
	AppName:   "protobuf",
	Version:   "25.3",
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
	HomePage:           "https://protobuf.dev/",
}

var RipgrepInstaller = &installer.Installer{
	AppName:   "ripgrep",
	Version:   "14.1.0",
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
	HomePage:           "https://github.com/BurntSushi/ripgrep",
}

var ScalaInstaller = installer.NewCoursierInstaller()

var TreesitterInstaller = &installer.Installer{
	AppName:        "tree-sitter",
	Version:        "0.22.1",
	IsZipFile:      true,
	BinaryRenameTo: "tree-sitter",
	FlagFileGetter: func() []string {
		return []string{"tree-sitter"}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
	HomePage:           "https://tree-sitter.github.io/tree-sitter/",
}

var TypstLspInstaller = &installer.Installer{
	AppName:        "typst-lsp",
	Version:        "0.12.1",
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
	HomePage:           "https://github.com/nvarner/typst-lsp",
}

var TypstInstaller = &installer.Installer{
	AppName:   "typst",
	Version:   "0.10.0",
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
	HomePage:           "https://typst.app/",
}

var VHSInstaller = &installer.Installer{
	AppName:   "vhs",
	Version:   "0.7.1",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE"}
	},
	FlagDirExcepted: true,
	BinListGetter: func() []string {
		r := []string{"vhs"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"vhs.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	HomePage:           "https://github.com/charmbracelet/vhs",
}

var VlangLspInstaller = &installer.Installer{
	AppName:   "v-analyzer",
	Version:   "0.0.3",
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
	HomePage:           "https://github.com/v-analyzer/v-analyzer",
}

var VlangInstaller = &installer.Installer{
	AppName:   "v",
	Version:   "0.4.4",
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
	HomePage:           "https://vlang.io/",
}

var ZigLspInstaller = &installer.Installer{
	AppName:   "zls",
	Version:   "0.11.0",
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
	HomePage:           "https://github.com/zigtools/zls",
}

var ZigInstaller = &installer.Installer{
	AppName:   "zig",
	Version:   "0.11.0",
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
	HomePage:           "https://ziglang.org/",
}

var PythonInstaller = installer.NewCondaInstaller()

/*
Windows only.
or
Latest version only.
*/
var GitWinInstaller = &installer.Installer{
	AppName:   "git",
	Version:   "2.44.0",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bin", "cmd", "usr"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
			{"usr", "bin"},
			{"cmd"},
			{"mingw64", "bin"},
		}
	},
	AddBinDirToPath: true,
	HomePage:        "https://gitforwindows.org/",
}

var GsudoWinInstaller = &installer.Installer{
	AppName:   "gsudo",
	Version:   "2.4.4",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"x86", "x64", "arm64"}
	},
	BinDirGetter: func(version string) (r [][]string) {
		switch runtime.GOARCH {
		case "amd64":
			r = [][]string{{"x64"}}
		case "arm64":
			r = [][]string{{"arm64"}}
		case "386":
			r = [][]string{{"x86"}}
		default:
			r = [][]string{{"net46-AnyCpu"}}
		}
		return
	},
	BinListGetter: func() []string {
		return []string{"gsudo.exe"}
	},
	ForceReDownload: true,
	HomePage:        "https://gerardog.github.io/gsudo/",
}

var CygwinInstaller = &installer.Installer{
	AppName:        "cygwin",
	Version:        "latest",
	IsZipFile:      false,
	BinaryRenameTo: "cygwin-installer",
	FlagFileGetter: func() []string {
		return []string{"setup-x86_64.exe"}
	},
	ForceReDownload: true,
	HomePage:        "https://www.cygwin.com/",
}

var Msys2Installer = &installer.Installer{
	AppName:        "msys2",
	Version:        "latest",
	IsZipFile:      false,
	BinaryRenameTo: "msys2-installer",
	FlagFileGetter: func() []string {
		return []string{"msys2-x86_64-latest.exe"}
	},
	ForceReDownload: true,
	HomePage:        "https://www.msys2.org/",
}

var RustupInstaller = &installer.Installer{
	AppName:        "rustup",
	Version:        "latest",
	IsZipFile:      false,
	BinaryRenameTo: "rustup-init",
	FlagFileGetter: func() []string {
		return []string{"rustup"}
	},
	DUrlDecorator:   installer.DefaultDecorator,
	ForceReDownload: true,
	HomePage:        "https://rustup.rs/",
}

var SDKManagerInstaller = &installer.Installer{
	AppName:   "sdkmanager", // commandline-tools
	Version:   "latest",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bin", "lib"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	AddBinDirToPath: true,
	HomePage:        "https://developer.android.google.cn/tools/releases/cmdline-tools",
}

/*
customed installation.
*/
var RustInstaller = &installer.Installer{
	AppName:    "rust",
	Version:    "latest",
	IsZipFile:  false,
	NoDownload: true,
	Install: func(appName, version, zipFileName string) {
		if conf.UseMirrorSiteInChina() {
			/*
				export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
				export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
			*/
			os.Setenv("RUSTUP_DIST_SERVER", "https://mirrors.ustc.edu.cn/rust-static")
			os.Setenv("RUSTUP_UPDATE_ROOT", "https://mirrors.ustc.edu.cn/rust-static/rustup")
		}
		rustDir := conf.GetVMVersionsDir(appName)
		binDir := conf.GetAppBinDir()
		rustupInitName := "rustup-init"
		if runtime.GOOS == gutils.Windows {
			rustupInitName += ".exe"
		}
		binPath := filepath.Join(binDir, rustupInitName)
		if ok, _ := gutils.PathIsExist(binPath); ok {
			os.Setenv("CARGO_HOME", filepath.Join(rustDir, "cargo"))
			os.Setenv("RUSTUP_HOME", filepath.Join(rustDir, "rustups"))
			if _, err := gutils.ExecuteSysCommand(false, "", binPath); err != nil {
				gprint.PrintError("Execute %s failed.", rustupInitName)
			}
		} else {
			gprint.PrintWarning("Please intall rustup-init first.")
		}
	},
	UnInstall: func(appName, version string) {
		rustDir := conf.GetVMVersionsDir(appName)
		os.RemoveAll(rustDir)
	},
	HomePage: "https://www.rust-lang.org/",
}

var MinicondaInstaller = &installer.Installer{
	AppName:   "miniconda",
	Version:   "latest",
	IsZipFile: false,
	Install: func(appName, version, zipFileName string) {
		vDir := filepath.Join(conf.GetVMVersionsDir(appName), appName)
		if ok, _ := gutils.PathIsExist(vDir); ok {
			os.RemoveAll(vDir)
		}
		var err error
		if runtime.GOOS != gutils.Windows {
			// bash ~/miniconda.sh -b -p $HOME/miniconda
			gutils.ExecuteSysCommand(false, "", "chmod", "+x", zipFileName)
			_, err = gutils.ExecuteSysCommand(false, "", "bash", zipFileName, "-b", "-p", vDir)
		} else {
			// start /wait "" Miniconda3-latest-Windows-x86_64.exe /InstallationType=JustMe /RegisterPython=0 /S /D=%UserProfile%\Miniconda3
			_, err = gutils.ExecuteSysCommand(false, "",
				"start", "/wait", "", zipFileName, "/InstallationType=JustMe",
				"/RegisterPython=0", "/S", fmt.Sprintf("/D=%s", vDir))
		}
		if err != nil {
			gprint.PrintError("Install %s failed.", appName)
		} else {
			binDir := filepath.Join(vDir, "bin")
			if ok, _ := gutils.PathIsExist(binDir); ok {
				em := envs.NewEnvManager()
				em.AddToPath(binDir)
			}
		}
	},
	UnInstall: func(appName, version string) {
		miniDir := conf.GetVMVersionsDir(appName)
		os.RemoveAll(miniDir)
		binDir := filepath.Join(miniDir, appName, "bin")
		em := envs.NewEnvManager()
		em.DeleteFromPath(binDir)
	},
	DUrlDecorator: func(dUrl string, ft *request.Fetcher) string {
		if conf.UseMirrorSiteInChina() {
			return strings.ReplaceAll(dUrl, "https://repo.anaconda.com/miniconda/", "https://mirrors.tuna.tsinghua.edu.cn/anaconda/miniconda/")
		}
		return dUrl
	},
	HomePage: "https://docs.anaconda.com/free/miniconda/index.html",
}

func vscodeNoDownload() bool {
	return runtime.GOOS == gutils.Linux
}

func vscodeIsZipFile() bool {
	return runtime.GOOS == gutils.Windows
}

var VSCodeInstaller = &installer.Installer{
	AppName:    "vscode",
	Version:    "latest",
	HomePage:   "https://code.visualstudio.com/",
	IsZipFile:  vscodeIsZipFile(),
	NoDownload: vscodeNoDownload(),
	Install: func(appName, version, zipFileName string) {
		var installDir string = filepath.Join("/Applications", "Visual Studio Code.app") // macOS
		homeDir, _ := os.UserHomeDir()
		switch runtime.GOOS {
		case gutils.Windows:
			if strings.HasSuffix(zipFileName, ".exe") {
				gutils.ExecuteSysCommand(false, homeDir, zipFileName, "/VERYSILENT", "/MERGETASKS=!runcode")
			}
		case gutils.Darwin:
			f := installer.NewFinder("Visual Studio Code.app")
			f.Find(conf.GetVMTempDir())
			if ok, _ := gutils.PathIsExist(f.Home); ok {
				utils.CopyFileOnUnixSudo(f.Home, installDir)
			}
			os.RemoveAll(conf.GetVMTempDir())
		case gutils.Linux:
			/*
				https://code.visualstudio.com/docs/setup/linux

				1.
				sudo apt-get install wget gpg
				wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > packages.microsoft.gpg
				sudo install -D -o root -g root -m 644 packages.microsoft.gpg /etc/apt/keyrings/packages.microsoft.gpg
				sudo sh -c 'echo "deb [arch=amd64,arm64,armhf signed-by=/etc/apt/keyrings/packages.microsoft.gpg] https://packages.microsoft.com/repos/code stable main" > /etc/apt/sources.list.d/vscode.list'
				rm -f packages.microsoft.gpg

				sudo apt install apt-transport-https
				sudo apt update
				sudo apt install code # or code-insiders

				2.
				sudo rpm --import https://packages.microsoft.com/keys/microsoft.asc
				sudo sh -c 'echo -e "[code]\nname=Visual Studio Code\nbaseurl=https://packages.microsoft.com/yumrepos/vscode\nenabled=1\ngpgcheck=1\ngpgkey=https://packages.microsoft.com/keys/microsoft.asc" > /etc/yum.repos.d/vscode.repo'

				dnf check-update
				sudo dnf install code # or code-insiders

				yum check-update
				sudo yum install code # or code-insiders
			*/
			installerCmd := utils.DNForAPTonLinux()
			if installerCmd == "apt" {
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "apt-get", "install", "wget", "gpg")
				gutils.ExecuteSysCommand(false, homeDir, "wget", "-qO-", "https://packages.microsoft.com/keys/microsoft.asc", "|", "gpg", "--dearmor", ">", "packages.microsoft.gpg")
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "install", "-D", "-o", "root", "-g", "root", "-m", "644", "packages.microsoft.gpg", "/etc/apt/keyrings/packages.microsoft.gpg")
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "sh", "-c", `'echo "deb [arch=amd64,arm64,armhf signed-by=/etc/apt/keyrings/packages.microsoft.gpg] https://packages.microsoft.com/repos/code stable main" > /etc/apt/sources.list.d/vscode.list'`)
				gutils.ExecuteSysCommand(false, homeDir, "rm", "-f", "packages.microsoft.gpg")

				gutils.ExecuteSysCommand(false, homeDir, "sudo", "apt", "install", "apt-transport-https")
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "apt", "update")
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "apt", "install", "code")
			} else if installerCmd == "yum" {
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "rpm", "--import", "https://packages.microsoft.com/keys/microsoft.asc")
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "sh", "-c", `'echo -e "[code]\nname=Visual Studio Code\nbaseurl=https://packages.microsoft.com/yumrepos/vscode\nenabled=1\ngpgcheck=1\ngpgkey=https://packages.microsoft.com/keys/microsoft.asc" > /etc/yum.repos.d/vscode.repo'`)
				gutils.ExecuteSysCommand(false, homeDir, "yum", "check-update")
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "yum", "install", "code")
			} else if installerCmd == "dnf" {
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "rpm", "--import", "https://packages.microsoft.com/keys/microsoft.asc")
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "sh", "-c", `'echo -e "[code]\nname=Visual Studio Code\nbaseurl=https://packages.microsoft.com/yumrepos/vscode\nenabled=1\ngpgcheck=1\ngpgkey=https://packages.microsoft.com/keys/microsoft.asc" > /etc/yum.repos.d/vscode.repo'`)
				gutils.ExecuteSysCommand(false, homeDir, "dnf", "check-update")
				gutils.ExecuteSysCommand(false, homeDir, "sudo", "dnf", "install", "code")
			}
		default:
			gprint.PrintError("Not supported.")
		}
	},
	UnInstall: func(appName, version string) {
		gprint.PrintInfo("Please uninstall vscode manually.")
	},
}

func init() {
	VersionKeeper["bun"] = BunInstaller
	VersionKeeper["cmdtools"] = SDKManagerInstaller
	VersionKeeper["coursier"] = CoursierInstaller
	VersionKeeper["cygwin"] = CygwinInstaller
	VersionKeeper["deno"] = DenoInstaller
	VersionKeeper["dotnet"] = DotNetInstaller
	VersionKeeper["fd"] = FdInstaller
	VersionKeeper["flutter"] = FlutterInstaller
	VersionKeeper["fzf"] = FzFInstaller
	VersionKeeper["git"] = GitWinInstaller
	VersionKeeper["gsudo"] = GsudoWinInstaller
	VersionKeeper["gleam"] = GleamInstaller
	VersionKeeper["glow"] = GlowInstaller
	VersionKeeper["go"] = GoInstaller
	VersionKeeper["gradle"] = GradleInstaller
	VersionKeeper["jdk"] = JdkInstaller
	VersionKeeper["julia"] = JuliaInstaller
	VersionKeeper["kotlin"] = KotlinInstaller
	VersionKeeper["lazygit"] = LazyGitInstaller
	VersionKeeper["maven"] = MavenInstaller
	VersionKeeper["miniconda"] = MinicondaInstaller
	VersionKeeper["msys2"] = Msys2Installer
	VersionKeeper["neovim"] = NeovimInstaller
	VersionKeeper["nodejs"] = NodejsInstaller
	VersionKeeper["php"] = PHPInstaller
	VersionKeeper["protobuf"] = ProtobufInstaller
	VersionKeeper["python"] = PythonInstaller
	VersionKeeper["ripgrep"] = RipgrepInstaller
	VersionKeeper["rust"] = RustInstaller
	VersionKeeper["rustup"] = RustupInstaller
	VersionKeeper["scala"] = ScalaInstaller
	VersionKeeper["tree-sitter"] = TreesitterInstaller
	VersionKeeper["typst-lsp"] = TypstLspInstaller
	VersionKeeper["typst"] = TypstInstaller
	VersionKeeper["vhs"] = VHSInstaller
	VersionKeeper["v-analyzer"] = VlangLspInstaller
	VersionKeeper["v"] = VlangInstaller
	VersionKeeper["vscode"] = VSCodeInstaller
	VersionKeeper["zls"] = ZigLspInstaller
	VersionKeeper["zig"] = ZigInstaller
}

func RunInstaller(manager VersionManager) {
	zf := manager.Download()
	manager.Unzip(zf)
	if manager.GetInstall() != nil {
		manager.InstallApp(zf) // customed installation.
	} else {
		// ordinary installation.
		manager.Copy()
		manager.CreateVersionSymbol()
		manager.CreateBinarySymbol()
		manager.SetEnv()
	}
}

func RunUnInstaller(manager VersionManager) {
	manager.UnInstallApp()
}

func RunClearCache(manager VersionManager) {
	manager.ClearCache()
}
