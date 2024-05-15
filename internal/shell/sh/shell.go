package sh

const (
	Bash = "bash"
	Zsh  = "zsh"
	Fish = "fish"
)

const (
	vmEnvFileName = "vmr"
)

type Sheller interface {
	ConfPath() string
	VMEnvConfPath() string
	WriteVMEnvToShell()
	PackPath(path string) string
	PackEnv(key, value string) string
}
