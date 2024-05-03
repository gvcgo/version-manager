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
	"github.com/gvcgo/version-manager/internal/envs"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/installer"
	"github.com/gvcgo/version-manager/pkgs/utils"
)

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
	SearchVersions()
	FixAppName()
}

/*
Keeps multi versions.
*/
var VersionKeeper = map[string]VersionManager{}

var AggInstaller = &installer.Installer{
	AppName:        "agg",
	Version:        "1.4.3",
	IsZipFile:      false,
	BinaryRenameTo: "agg",
	FlagFileGetter: func() []string {
		r := []string{"agg"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"agg.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
	HomePage:           "https://github.com/asciinema/agg",
}

var AsciinemaInstaller = &installer.Installer{
	AppName:   "asciinema",
	Version:   "0.3.9",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"acast"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"acast.exe"}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"acast"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"acast.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
	HomePage:           "https://github.com/gvcgo/asciinema",
}

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

var DlangInstaller = &installer.Installer{
	AppName:   "dlang",
	Version:   "2.108.0",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		switch runtime.GOOS {
		case gutils.Darwin:
			return []string{"osx"}
		case gutils.Linux:
			return []string{"linux"}
		default:
			return []string{"windows"}
		}
	},

	BinDirGetter: func(version string) [][]string {
		switch runtime.GOOS {
		case gutils.Darwin:
			return [][]string{
				{filepath.Join("osx", "bin")},
			}
		case gutils.Windows:
			return [][]string{
				{filepath.Join("windows", "bin")},
			}
		case gutils.Linux:
			return [][]string{
				{filepath.Join("linux", "bin64")},
			}
		default:
			return [][]string{}
		}
	},
	BinListGetter: func() []string {
		if runtime.GOOS == gutils.Windows {
			return []string{"dmd.exe"}
		}
		return []string{"dmd"}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	AddBinDirToPath:    true,
	StoreMultiVersions: true,
	HomePage:           "https://dlang.org/",
}

var DlangLspInstaller = &installer.Installer{
	AppName:   "serve-d",
	Version:   "v0.7.6",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		if runtime.GOOS == gutils.Windows {
			return []string{"serve-d.exe"}
		}
		return []string{"serve-d"}
	},
	BinListGetter: func() []string {
		if runtime.GOOS == gutils.Windows {
			return []string{"serve-d.exe"}
		}
		return []string{"serve-d"}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	HomePage:           "https://github.com/Pure-D/serve-d",
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
	BinListGetter: func() []string {
		if runtime.GOOS == gutils.Windows {
			return []string{"gradle.bat"}
		}
		return []string{"gradle"}
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

var GroovyInstaller = &installer.Installer{
	AppName:   "groovy",
	Version:   "4.0.9",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bin"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	AddBinDirToPath:    true,
	HomePage:           "https://www.groovy-lang.org/",
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

var KubectlInstaller = &installer.Installer{
	AppName:        "kubectl",
	Version:        "1.29.3",
	IsZipFile:      false,
	BinaryRenameTo: "kubectl",
	FlagFileGetter: func() []string {
		r := []string{"kubectl"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"kubectl.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
	HomePage:           "https://kubernetes.io/docs/tasks/tools/",
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
		if runtime.GOOS == gutils.Windows {
			return []string{"mvn.cmd"}
		}
		return []string{"mvn"}
	},
	DUrlDecorator: func(dUrl string, ft *request.Fetcher) string {
		if conf.UseMirrorSiteInChina() {
			return strings.ReplaceAll(dUrl, "https://dlcdn.apache.org/maven/", "https://mirrors.aliyun.com/apache/maven/")
		}
		return installer.DefaultDecorator(dUrl, ft)
	},
	EnvGetter: func(appName, version string) []installer.Env {
		return []installer.Env{
			{
				Name:  "MAVEN_HOME",
				Value: filepath.Join(conf.GetVMVersionsDir(appName), appName),
			},
		}
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

var OdinInstaller = &installer.Installer{
	AppName:   "odin",
	Version:   "dev-2024-04",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE", "base"}
	},
	BinListGetter: func() []string {
		if runtime.GOOS == gutils.Windows {
			return []string{"odin.exe"}
		}
		return []string{"odin"}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	AddBinDirToPath:    true,
	StoreMultiVersions: true,
	HomePage:           "https://odin-lang.org/",
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
	ForceReDownload:    true,
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

var TypstPreviewInstaller = &installer.Installer{
	AppName:        "typst-preview",
	Version:        "0.11.1",
	IsZipFile:      false,
	BinaryRenameTo: "typst-preview",
	FlagFileGetter: func() []string {
		if runtime.GOOS == gutils.Windows {
			return []string{"typst-preview.exe"}
		}
		return []string{"typst-preview"}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
	HomePage:           "https://github.com/Enter-tainer/typst-preview",
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

var UPXInstaller = &installer.Installer{
	AppName:   "upx",
	Version:   "v4.2.3",
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"upx"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"upx.exe"}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"upx"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"upx.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
	HomePage:           "https://github.com/upx/upx",
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
	ForceReDownload:    true,
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
	ForceReDownload:    true,
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

var PythonInstaller = installer.NewCondaInstaller(installer.CondaPython)

var PyPyInstaller = installer.NewCondaInstaller(installer.CondaPyPy)

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
	DUrlDecorator:      installer.DefaultDecorator,
	AddBinDirToPath:    true,
	StoreMultiVersions: true,
	HomePage:           "https://gitforwindows.org/",
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
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
	HomePage:           "https://gerardog.github.io/gsudo/",
}

var CygwinInstaller = &installer.Installer{
	AppName:        "cygwin",
	Version:        "latest",
	IsZipFile:      false,
	BinaryRenameTo: "cygwin-installer",
	FlagFileGetter: func() []string {
		return []string{"cygwin-installer.exe"}
	},
	BinListGetter: func() []string {
		return []string{"cygwin-installer.exe"}
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
		return []string{"msys2-installer.exe"}
	},
	BinListGetter: func() []string {
		return []string{"msys2-installer.exe"}
	},
	DUrlDecorator:   installer.DefaultDecorator,
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
	AppName:   "android-cmdline-tools", // cmdline-tools
	Version:   "latest",
	IsZipFile: true,
	Install: func(appName, version, zipFileName string) {
		tmpDir := conf.GetVMTempDir()
		if ok, _ := gutils.PathIsExist(tmpDir); ok {
			// Must be in cmdline-tools/latest/...
			dstDir := filepath.Join(installer.GetAndroidHomeDir(), "cmdline-tools", "latest")
			finder := installer.NewFinder("bin", "lib")
			finder.Find(tmpDir)
			err := gutils.CopyDirectory(finder.Home, dstDir, true)
			if err != nil {
				gprint.PrintError("Copy file failed: %+v", err)
				os.RemoveAll(tmpDir)
				os.Exit(1)
			}

			binDir := filepath.Join(dstDir, "bin")
			em := envs.NewEnvManager()
			defer em.CloseKey()
			em.AddToPath(binDir)
			em.Set("ADROID_HOME", installer.GetAndroidHomeDir())
		}
		os.RemoveAll(tmpDir)
	},
	UnInstall: func(appName, version string) {
		cmdlineToolsDir := filepath.Join(installer.GetAndroidHomeDir(), "cmdline-tools")
		os.RemoveAll(cmdlineToolsDir)

		binDir := filepath.Join(cmdlineToolsDir, "latest", "bin")
		em := envs.NewEnvManager()
		defer em.CloseKey()
		em.DeleteFromPath(binDir)
		em.UnSet("ANDROID_HOME")
	},
	StoreMultiVersions: false,
	HomePage:           "https://developer.android.google.cn/tools/releases/cmdline-tools",
}

var AndroidBuildToolsInstaller = &installer.AndroidSDKInstaller{
	AppName:  "android-build-tools",
	Version:  "latest",
	HomePage: "https://developer.android.com/tools/releases/build-tools?hl=en",
	EnvGetter: func(appName, version string) []installer.Env {
		r := []installer.Env{}
		sList := strings.Split(version, ";")
		if len(sList) == 2 {
			binDir := filepath.Join(installer.GetAndroidHomeDir(), "build-tools", sList[1])
			if ok, _ := gutils.PathIsExist(binDir); ok {
				r = append(r, installer.Env{
					Name:  "PATH",
					Value: binDir,
				})
			}
		}
		return r
	},
}

var AndroidPlatformsInstaller = &installer.AndroidSDKInstaller{
	AppName:  "android-platforms",
	Version:  "latest",
	HomePage: "https://developer.android.com/studio",
	EnvGetter: func(appName, version string) []installer.Env {
		r := []installer.Env{}
		platformToolsDir := filepath.Join(installer.GetAndroidHomeDir(), "platform-tools")
		if ok, _ := gutils.PathIsExist(platformToolsDir); ok {
			r = append(r, installer.Env{
				Name:  "PATH",
				Value: platformToolsDir,
			})
		}
		return r
	},
}

var AndroidSystemImagesInstaller = &installer.AndroidSDKInstaller{
	AppName:  "android-system-images",
	Version:  "latest",
	HomePage: "https://developer.android.google.cn/topic/generic-system-image/releases?hl=en",
	EnvGetter: func(appName, version string) []installer.Env {
		r := []installer.Env{}
		emulatorDir := filepath.Join(installer.GetAndroidHomeDir(), "emulator")
		if ok, _ := gutils.PathIsExist(emulatorDir); ok {
			r = append(r, installer.Env{
				Name:  "PATH",
				Value: emulatorDir,
			})

			// utils.ExecuteSysCommand(false, "avdmanager", "create", "avd", "--name", avdName, "--package", systemImage)
			gprint.PrintSuccess("If you wanna create an avd further, please use commad as below:")
			fmt.Printf("avdmanager create avd --name <your-avd-name> --package %s\n", version)
		}
		return r
	},
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
		if runtime.GOOS == gutils.Windows {
			binPath, _ = os.Readlink(binPath)
		}

		if ok, _ := gutils.PathIsExist(binPath); !ok {
			gprint.PrintInfo("Installing rustup-init...")
			gutils.ExecuteSysCommand(false, "", "vmr", "use", "rustup@latest")
		}

		if ok, _ := gutils.PathIsExist(binPath); ok {
			os.Setenv("CARGO_HOME", filepath.Join(rustDir, "cargo"))
			os.Setenv("RUSTUP_HOME", filepath.Join(rustDir, "rustups"))
			if _, err := gutils.ExecuteSysCommand(false, "", binPath); err != nil {
				gprint.PrintError("Execute %s failed.", rustupInitName)
			}
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
			binDir2 := filepath.Join(vDir, "condabin")
			em := envs.NewEnvManager()
			defer em.CloseKey()
			if ok, _ := gutils.PathIsExist(binDir); ok {
				em.AddToPath(binDir)
			}
			if ok, _ := gutils.PathIsExist(binDir2); ok {
				em.AddToPath(binDir2)
			}
		}
	},
	UnInstall: func(appName, version string) {
		miniDir := conf.GetVMVersionsDir(appName)
		os.RemoveAll(miniDir)
		binDir := filepath.Join(miniDir, appName, "bin")
		binDir2 := filepath.Join(miniDir, appName, "condabin")
		em := envs.NewEnvManager()
		defer em.CloseKey()
		em.DeleteFromPath(binDir)
		em.DeleteFromPath(binDir2)
	},
	DUrlDecorator: func(dUrl string, ft *request.Fetcher) string {
		if conf.UseMirrorSiteInChina() {
			return strings.ReplaceAll(dUrl, "https://repo.anaconda.com/miniconda/", "https://mirrors.tuna.tsinghua.edu.cn/anaconda/miniconda/")
		}
		return dUrl
	},
	ForceReDownload: true,
	HomePage:        "https://docs.anaconda.com/free/miniconda/index.html",
}

func vscodeIsZipFile() bool {
	return runtime.GOOS == gutils.Darwin
}

var VSCodeInstaller = &installer.Installer{
	AppName:   "vscode",
	Version:   "latest",
	HomePage:  "https://code.visualstudio.com/",
	IsZipFile: vscodeIsZipFile(),
	VersionFilter: func(dUrl string) bool {
		switch runtime.GOOS {
		case gutils.Linux:
			installerCmd := utils.DNForAPTonLinux()
			if installerCmd == "apt" && strings.HasSuffix(dUrl, ".deb") {
				return true
			} else if installerCmd == "yum" && strings.HasSuffix(dUrl, ".rpm") {
				return true
			} else if installerCmd == "dnf" && strings.HasSuffix(dUrl, ".rpm") {
				return true
			}
			return false
		default:
			return true
		}
	},
	Install: func(appName, version, zipFileName string) {
		homeDir, _ := os.UserHomeDir()
		switch runtime.GOOS {
		case gutils.Windows:
			if strings.HasSuffix(zipFileName, ".exe") {
				gutils.ExecuteSysCommand(false, homeDir, zipFileName, "/VERYSILENT", "/MERGETASKS=!runcode")
			}
		case gutils.Darwin:
			appName := "Visual Studio Code.app"
			f := installer.NewFinder(appName)
			f.Find(conf.GetVMTempDir())
			appPath := filepath.Join(f.Home, appName)
			if ok, _ := gutils.PathIsExist(appPath); ok {
				utils.MoveFileOnUnixSudo(appPath, "/Applications")
			}
			os.RemoveAll(conf.GetVMTempDir())
		case gutils.Linux:
			if ok, _ := gutils.PathIsExist(zipFileName); !ok {
				return
			}
			installerCmd := utils.DNForAPTonLinux()
			if installerCmd == "apt" {
				gutils.ExecuteSysCommand(false, homeDir,
					"sudo", "dpkg", "-i", zipFileName)
			} else if installerCmd == "yum" || installerCmd == "dnf" {
				gutils.ExecuteSysCommand(false, homeDir,
					"sudo", "rpm", "-ivh", zipFileName)
			}
		default:
			gprint.PrintError("Not supported.")
		}
	},
	UnInstall: func(appName, version string) {
		gprint.PrintInfo("Please uninstall vscode manually.")
	},
	ForceReDownload: true,
}

func init() {
	VersionKeeper["agg"] = AggInstaller
	VersionKeeper["asciinema"] = AsciinemaInstaller
	VersionKeeper["bun"] = BunInstaller

	// Android SDKs
	VersionKeeper["android-cmdline-tools"] = SDKManagerInstaller
	VersionKeeper["android-build-tools"] = AndroidBuildToolsInstaller
	VersionKeeper["android-platforms"] = AndroidPlatformsInstaller
	VersionKeeper["android-system-images"] = AndroidSystemImagesInstaller

	VersionKeeper["coursier"] = CoursierInstaller
	VersionKeeper["cygwin"] = CygwinInstaller
	VersionKeeper["deno"] = DenoInstaller
	VersionKeeper["dlang"] = DlangInstaller
	VersionKeeper["serve-d"] = DlangLspInstaller
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
	VersionKeeper["groovy"] = GroovyInstaller
	VersionKeeper["jdk"] = JdkInstaller
	VersionKeeper["julia"] = JuliaInstaller
	VersionKeeper["kotlin"] = KotlinInstaller
	VersionKeeper["kubectl"] = KubectlInstaller
	VersionKeeper["lazygit"] = LazyGitInstaller
	VersionKeeper["maven"] = MavenInstaller
	VersionKeeper["miniconda"] = MinicondaInstaller
	VersionKeeper["msys2"] = Msys2Installer
	VersionKeeper["neovim"] = NeovimInstaller
	VersionKeeper["nodejs"] = NodejsInstaller
	VersionKeeper["odin"] = OdinInstaller
	VersionKeeper["php"] = PHPInstaller
	VersionKeeper["protobuf"] = ProtobufInstaller
	VersionKeeper["python"] = PythonInstaller
	VersionKeeper["pypy"] = PyPyInstaller
	VersionKeeper["ripgrep"] = RipgrepInstaller
	VersionKeeper["rust"] = RustInstaller
	VersionKeeper["rustup"] = RustupInstaller
	VersionKeeper["scala"] = ScalaInstaller
	VersionKeeper["tree-sitter"] = TreesitterInstaller
	VersionKeeper["typst-lsp"] = TypstLspInstaller
	VersionKeeper["typst-preview"] = TypstPreviewInstaller
	VersionKeeper["typst"] = TypstInstaller
	VersionKeeper["upx"] = UPXInstaller
	VersionKeeper["vhs"] = VHSInstaller
	VersionKeeper["v-analyzer"] = VlangLspInstaller
	VersionKeeper["v"] = VlangInstaller
	VersionKeeper["vscode"] = VSCodeInstaller
	VersionKeeper["zls"] = ZigLspInstaller
	VersionKeeper["zig"] = ZigInstaller
}

func RunInstaller(manager VersionManager) {
	manager.FixAppName()
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
	manager.FixAppName()
	manager.UnInstallApp()
}

func RunClearCache(manager VersionManager) {
	manager.FixAppName()
	manager.ClearCache()
}
