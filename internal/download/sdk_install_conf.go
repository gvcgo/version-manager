package download

import (
	"crypto/sha256"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/pelletier/go-toml/v2"
)

type FileItems struct {
	Windows []string `toml:"windows"`
	Linux   []string `toml:"linux"`
	MacOS   []string `toml:"darwin"`
}

type (
	DirPath  []string
	DirItems struct {
		Windows []DirPath `toml:"windows"` // <symbolLinkPath>/<filepath.Join(List)>, ...
		Linux   []DirPath `toml:"linux"`
		MacOS   []DirPath `toml:"darwin"`
	}
)

type AdditionalEnv struct {
	Name    string
	Value   []DirPath // <symbolLinkPath>/<filepath.Join(Value)>
	Version string    // major>8 or major<=8(for JDK)
}

type AdditionalEnvList []AdditionalEnv

type BinaryRename struct {
	NameFlag string `toml:"name_flag"`
	RenameTo string `toml:"rename_to"`
}

/*
Installation configs
*/
type InstallerConfig struct {
	FlagFiles       *FileItems        `toml:"flag_files"`
	FlagDirExcepted bool              `toml:"flag_dir_excepted"`
	BinaryDirs      *DirItems         `toml:"binary_dirs"`
	BinaryRename    *BinaryRename     `toml:"binary_rename"`
	AdditionalEnvs  AdditionalEnvList `toml:"additional_envs"`
}

/*
SDK installation config file.
*/
func GetSDKInstallationConfig(sdkName, newSha256 string) (ic InstallerConfig) {
	ic = InstallerConfig{FlagFiles: &FileItems{}, BinaryDirs: &DirItems{}, BinaryRename: &BinaryRename{}}
	fPath := filepath.Join(cnf.GetSDKInstallationConfDir(), fmt.Sprintf("%s.toml", sdkName))

	oldContent, _ := os.ReadFile(fPath)
	h := sha256.New()
	h.Write(oldContent)
	oldSha256 := fmt.Sprintf("%x", h.Sum(nil))

	if oldSha256 == newSha256 {
		toml.Unmarshal(oldContent, &ic)
		return
	}

	dUrl := cnf.GetSDKInstallationConfFileUrlBySDKName(sdkName)
	fetcher := request.NewFetcher()
	fetcher.SetUrl(dUrl)
	fetcher.Timeout = 10 * time.Second
	if resp, code := fetcher.GetString(); code == 200 {
		toml.Unmarshal([]byte(resp), &ic)
		os.WriteFile(fPath, []byte(resp), os.ModePerm)
	}
	return
}
