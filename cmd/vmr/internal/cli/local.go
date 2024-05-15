package cli

import (
	"github.com/gvcgo/version-manager/pkgs/register"
	"github.com/spf13/cobra"
)

var localCmd = &cobra.Command{
	Use:     "local",
	Aliases: []string{"L"},
	GroupID: GroupID,
	Short:   "Shows installed versions for an app.",
	Long:    "Example: vmr L go.",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			_ = cmd.Help()
			return
		}
		register.ShowInstalled(args[0])
	},
}
