package cache

import (
	"runtime"
)

const (
	VersionFilePattern string = "https://raw.githubusercontent.com/gvcgo/resources/main/%s.version.json"
)

type InstallConf struct {
	DirKeyWord  string
	BinaryDir   string
	Binaries    []string
	Source      bool
	OsHandler   func(platform, arch string) string
	ArchHandler func(platform, arch string) string
}

var AppList = map[string]InstallConf{
	"go": {
		DirKeyWord: "go",
		BinaryDir:  "bin",
		Binaries:   []string{"go", "fmt"},
		Source:     false,
	},
}

/*
	{
		"Url": "https://go.dev/dl/go1.20.darwin-amd64.tar.gz",
		"Arch": "amd64",
		"Os": "darwin",
		"Sum": "777025500f62d14bb5a4923072cd97431887961d24de08433a60c2fe1120531d",
		"SumType": "SHA256",
		"Extra": ""
	}
*/
type Version struct {
	Url     string `json:"Url"`
	Arch    string `json:"Arch"`
	Os      string `json:"Os"`
	Sum     string `json:"Sum"`
	SumType string `json:"SumType"`
	Extra   string `json:"Extra"`
}

type VerList []Version

type VerInfo struct {
	List    map[string]VerList
	AppName string
}

func (v *VerInfo) ParseVersions(appName string, rUri, pxyUri string) {

}

func (v *VerInfo) GetVersion(version string) (r []Version) {
	l := v.List[version]
	if l == nil || v.AppName == "" {
		return
	}
	installConf, ok := AppList[v.AppName]
	if !ok {
		return
	}

	for _, ver := range l {
		platform := ver.Os
		if installConf.OsHandler != nil {
			platform = installConf.OsHandler(ver.Os, ver.Arch)
		}
		arch := ver.Arch
		if installConf.ArchHandler != nil {
			arch = installConf.ArchHandler(ver.Os, ver.Arch)
		}

		if platform == runtime.GOOS && arch == runtime.GOARCH {
			r = append(r, ver)
		}
	}
	return
}
