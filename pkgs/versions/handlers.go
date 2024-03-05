package versions

import "runtime"

/*
Arch handler
*/
func ToAnyArch(archType, osType string) string {
	return runtime.GOARCH
}

func ToUniverseForMac(archType, osType string) string {
	if osType == "darwin" {
		return runtime.GOARCH
	}
	return archType
}

func ToUnixArch(archType, osType string) string {
	if archType == "all" && osType == "linux" && runtime.GOOS != "windows" {
		return runtime.GOARCH
	}
	return archType
}

func ToDarwinX64(archType, osType string) string {
	if osType == "darwin" && archType == "" {
		return "amd64"
	}
	return archType
}

var ArchHandlerList = map[string]func(archType, osType string) string{
	"gradle":     ToAnyArch,
	"gsudo":      ToAnyArch,
	"maven":      ToAnyArch,
	"neovim":     ToUniverseForMac,
	"php":        ToUnixArch,
	"python":     ToAnyArch,
	"rust":       ToAnyArch,
	"sdkmanager": ToAnyArch,
	"vscode":     ToDarwinX64,
}

/*
Os handler
*/
func ToAnyOs(archType, osType string) string {
	return runtime.GOOS
}

func ToWindowsOnly(archType, osType string) string {
	return "windows"
}

func ToUnixOs(archType, osType string) string {
	if archType == "all" && osType == "linux" && runtime.GOOS != "windows" {
		return runtime.GOOS
	}
	return osType
}

var OsHandlerList = map[string]func(archType, osType string) string{
	"gradle": ToAnyOs,
	"gsudo":  ToWindowsOnly,
	"maven":  ToAnyOs,
	"php":    ToUnixOs,
	"python": ToAnyOs,
	"rust":   ToUnixOs,
}
