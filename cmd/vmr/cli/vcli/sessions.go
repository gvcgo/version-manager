package vcli

import (
	"fmt"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/cnf"
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
