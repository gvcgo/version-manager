package install

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

/*
Installation configs
*/
type InstallerConfig struct {
	FlagFiles       *FileItems        `toml:"flag_files"`
	FlagDirExcepted bool              `toml:"flag_dir_excepted"`
	BinaryDirs      *FileItems        `toml:"binary_dirs"`
	AdditionalEnvs  AdditionalEnvList `toml:"additional_envs"`
}
