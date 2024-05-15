package cli

import (
	"github.com/gvcgo/version-manager/pkgs/register"
	"github.com/spf13/cobra"
)

var listCmd = &cobra.Command{
	Use:     "list",
	Aliases: []string{"l"},
	GroupID: GroupID,
	Short:   "Shows the supported applications.",
	Run: func(cmd *cobra.Command, args []string) {
		register.ShowAppList()
	},
}
