package envs

import (
	"fmt"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
)

const (
	ShellFileName string = "vm_env.sh"
)

/*
Adds env manually.
*/
func AddEnvManually() {
	var key, value string
	fmt.Println(gprint.CyanStr("Input env name: "))
	fmt.Scanln(&key)
	fmt.Println(gprint.CyanStr("Input env value: "))
	fmt.Scanln(&value)
	key = strings.TrimSpace(key)
	value = strings.TrimSpace(value)
	if value != "" && key != "" {
		em := NewEnvManager()
		if strings.ToLower(key) == "path" {
			em.AddToPath(value)
		} else {
			em.Set(key, value)
		}
	}
}

/*
Removes env manually.
*/
func RemoveEnvManually() {
	var key, value string
	fmt.Println(gprint.CyanStr("Input env name: "))
	fmt.Scanln(&key)
	key = strings.TrimSpace(key)
	if strings.ToLower(key) == "path" {
		fmt.Println(gprint.CyanStr("Input the value in $PATH: "))
		fmt.Scanln(&value)
		value = strings.TrimSpace(value)
	}
	em := NewEnvManager()
	if key != "" && strings.ToLower(key) != "path" {
		em.UnSet(key)
	} else if value != "" {
		em.DeleteFromPath(value)
	}
}
