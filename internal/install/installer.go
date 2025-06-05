package install

import (
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
)

type Installer struct {
	Version string
	Plugin  *plugin.Plugin
}
