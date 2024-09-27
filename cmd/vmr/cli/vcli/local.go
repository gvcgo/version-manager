package vcli

import (
	"github.com/gvcgo/version-manager/internal/tui/cliui"
	"github.com/spf13/cobra"
)

/*
Show installed versions for an SDK.
*/
var ShowInstalledCmd = &cobra.Command{
	Use:     "local",
	Aliases: []string{"l"},
	GroupID: GroupID,
	Short:   "Shows installed versions for an SDK.",
	Long:    "Example: vmr local <sdk name>.",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			cmd.Help()
			return
		}
		sdkName := args[0]
		l := cliui.NewLocalInstalled()
		l.Search(sdkName)
		l.Show()
	},
}
