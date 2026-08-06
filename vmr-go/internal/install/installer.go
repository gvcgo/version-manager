package install

import (
	"github.com/gvcgo/version-manager/vmr-go/internal/luapi/plugin"
)

type Installer struct {
	Version string
	Plugin  *plugin.Plugin
}
