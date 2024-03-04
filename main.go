package main

import (
	"fmt"
	"runtime"

	"github.com/gvcgo/version-manager/pkgs/versions"
)

func main() {
	vf := versions.NewVInfo("neovim")
	vf.RegisterArchHandler(func(archType, osType string) string {
		if osType == "darwin" {
			return runtime.GOARCH
		}
		return archType
	})
	vf.RegisterOsHandler(func(archType, osType string) string {
		return osType
	})
	vl := vf.GetSortedVersionList()
	fmt.Println(vl)
}
