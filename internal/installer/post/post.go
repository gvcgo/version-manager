package post

import (
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
)

type PostInstallHandler func(versionName string, version lua_global.Item)

var PostInstallHandlers map[string]PostInstallHandler = map[string]PostInstallHandler{}

func RegisterPostInstallHandler(sdkName string, handler PostInstallHandler) {
	PostInstallHandlers[sdkName] = handler
}
