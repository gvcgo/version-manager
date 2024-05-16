package sh

import "io/fs"

const (
	Bash = "bash"
	Zsh  = "zsh"
	Fish = "fish"
)

const (
	ModePerm         fs.FileMode = 0o644
	VMDisableEnvName string      = "VM_DISABLE"
	vmEnvFileName    string      = "vmr"
)

type Sheller interface {
	ConfPath() string
	VMEnvConfPath() string
	WriteVMEnvToShell()
	PackPath(path string) string
	PackEnv(key, value string) string
}
