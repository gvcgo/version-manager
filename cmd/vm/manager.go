package main

import (
	"os"

	"github.com/gvcgo/version-manager/cmd"
	"github.com/gvcgo/version-manager/pkgs/conf"
)

/*
To be compiled.
*/
func main() {
	os.Setenv(conf.VMReverseProxyEnvName, "https://gvc.1710717.xyz/proxy/")
	cli := cmd.New()
	cli.Run()
}