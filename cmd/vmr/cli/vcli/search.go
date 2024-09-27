package vcli

import (
	"github.com/gvcgo/version-manager/internal/tui/cliui"
	"github.com/spf13/cobra"
)

/*
search versions for an SDK
*/
var SearchVersionsCmd = &cobra.Command{
	Use:     "search",
	Aliases: []string{"s"},
	GroupID: GroupID,
	Short:   "Searches available versions.",
	Long:    "Example: vmr search <sdk-name>.",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			cmd.Help()
			return
		}
		sdkName := args[0]
		l := cliui.NewVersionSearcher()
		l.Search(sdkName, "")
	},
}
