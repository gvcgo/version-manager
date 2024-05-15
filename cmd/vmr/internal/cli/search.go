package cli

import (
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/pkgs/register"
	"github.com/spf13/cobra"
)

var searchCmd = &cobra.Command{
	Use:     "search",
	Aliases: []string{"s"},
	GroupID: GroupID,
	Short:   "Shows the available versions of an application.",
	Long:    "Example: vmr search go.",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			_ = cmd.Help()
			return
		}

		installer, ok := register.VersionKeeper[args[0]]
		if ok && installer != nil {
			installer.FixAppName()
			installer.SearchVersions()
		} else {
			gprint.PrintWarning("unsupported sdk-name!")
		}
	},
}
