package cli

import (
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/spf13/cobra"
)

var toggleCustomedMirrors = &cobra.Command{
	Use:     "toggle-customed-mirrors",
	Aliases: []string{"tcm", "tm"},
	GroupID: GroupID,
	Short:   "Toggle customed mirrors.",
	Run: func(cmd *cobra.Command, args []string) {
		cnf.DefaultConfig.ToggleUseCustomedMirrors()
		if cnf.DefaultConfig.UseCustomedMirrors {
			gprint.PrintInfo("Customed mirrors enabled.")
		} else {
			gprint.PrintInfo("Customed mirrors disabled.")
		}
	},
}
