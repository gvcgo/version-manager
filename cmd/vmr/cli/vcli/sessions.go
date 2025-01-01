package vcli

import (
	"fmt"
	"os"
	"runtime"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/shell/sh"
	"github.com/spf13/cobra"
)

/*
Allow/Disallow nested sessions.
*/

var ToggleAllowNestedSessions = &cobra.Command{
	Use:     "nested-sessions",
	Aliases: []string{"ns"},
	GroupID: GroupID,
	Short:   "Toggle nested sessions.",
	Long:    "Example: vmr ns.",
	Run: func(cmd *cobra.Command, args []string) {
		if ok := cnf.DefaultConfig.ToggleAllowNestedSessions(); ok {
			fmt.Println(gprint.CyanStr("Nested sessions are now allowed."))
		} else {
			fmt.Println(gprint.YellowStr("Nested sessions are now disallowed."))
		}
	},
}

/*
Test session mode.

Nested session mode is not recommended, vmr ism can be used to check if current shell is in session mode.
If current shell is in session mode, users can use exit command to exit current shell.
*/

var IsSessionMode = &cobra.Command{
	Use:     "is-session-mode",
	Aliases: []string{"ism"},
	GroupID: GroupID,
	Short:   "Show if current shell is in session mode.",
	Long:    "Example: vmr ism.",
	Run: func(cmd *cobra.Command, args []string) {
		if gconv.Bool(os.Getenv(sh.VMDisableEnvName)) {
			fmt.Println(gprint.BrownStr("Current shell is in session mode."))
			// Hinsts for handling nested sessions.
			if runtime.GOOS != gutils.Windows {
				fmt.Println(gprint.YellowStr("Nested sessions are not recommeded for vmr. You can use 'exit' to exit session mode."))
				fmt.Println(gprint.YellowStr("If you install an SDK in global mode under a session mode shell, the 'source' command will not work, you should use the customed command 'svmr' instead."))
			}
		} else {
			fmt.Println(gprint.CyanStr("Current shell is in global mode."))
		}
	},
}
