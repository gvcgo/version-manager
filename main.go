package main

import (
	"fmt"

	"github.com/gvcgo/version-manager/pkgs/versions"
)

func main() {
	vf := versions.NewVInfo("go")
	vl := []string{}
	for v := range vf.GetVersions() {
		vl = append(vl, v)
	}
	fmt.Println(vl)
}
