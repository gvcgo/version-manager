package install

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/version-manager/internal/cnf"
)

const (
	VerisonDirPattern        string = "%s%s"
	VersionDirSuffix         string = "_versions"
	VersionInstallDirPattern string = "%s-%s"
)

func GetSDKVersionDir(sdkName string) string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf(VerisonDirPattern, sdkName, VersionDirSuffix))
	os.MkdirAll(d, os.ModePerm)
	return d
}

func IsSDKInstalledByVMR(sdkName string) bool {
	vd := GetSDKVersionDir(sdkName)
	dList, _ := os.ReadDir(vd)
	count := 0
	for _, d := range dList {
		if d.IsDir() {
			count++
		}
	}
	if count == 0 {
		os.RemoveAll(vd)
	}
	return count > 0
}

/*
============================
Installation configs.
============================
*/
type FileItems struct {
	Windows []string `toml:"windows"`
	Linux   []string `toml:"linux"`
	MacOS   []string `toml:"darwin"`
}

type AdditionalEnv struct {
	Name    string
	Value   string // <symbolLinkPath>/Value
	Version string // major>8 or major<=8(for JDK)
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
	BinaryDirs      *FileItems        `toml:"binary_dirs"`
	BinaryRename    *BinaryRename     `toml:"binary_rename"`
	AdditionalEnvs  AdditionalEnvList `toml:"additional_envs"`
}
