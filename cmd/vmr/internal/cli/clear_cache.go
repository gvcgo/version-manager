package cli

import (
	"github.com/gvcgo/version-manager/pkgs/register"
	"github.com/spf13/cobra"
)

var clearCacheCmd = &cobra.Command{
	Use:     "clear-cache",
	Aliases: []string{"c", "cc"},
	GroupID: GroupID,
	Short:   "Clears cached zip files for an app.",
	Long:    "Example: vmr c <sdk-name>",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			_ = cmd.Help()
			return
		}
		appName := args[0]
		if ins, ok := register.VersionKeeper[appName]; ok {
			register.RunClearCache(ins)
		}
	},
}
