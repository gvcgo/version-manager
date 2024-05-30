package install

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/version-manager/internal/cnf"
)

const (
	VerisonDirPattern        string = "%s_versions"
	VersionInstallDirPattern string = "%s-%s"
)

func GetSDKVersionDir(sdkName string) string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf(VerisonDirPattern, sdkName))
	os.MkdirAll(d, os.ModePerm)
	return d
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
