package shell

import "github.com/gvcgo/version-manager/internal/shell/sh"

type Sheller interface {
	sh.Sheller

	SetEnv(key, value string)
	UnsetEnv(key string)
	SetPath(path string)
	UnsetPath(path string)
}

// TODO: Read vm_env.sh to vmr.sh
