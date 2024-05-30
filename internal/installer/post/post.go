package post

import "github.com/gvcgo/version-manager/internal/download"

type PostInstallHandler func(versionName string, version download.Item)

var PostInstallHandlers map[string]PostInstallHandler = map[string]PostInstallHandler{}

func RegisterPostInstallHandler(sdkName string, handler PostInstallHandler) {
	PostInstallHandlers[sdkName] = handler
}
