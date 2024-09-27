package vcli

import (
	"github.com/gvcgo/version-manager/internal/tui/cliui"
	"github.com/spf13/cobra"
)

const (
	GroupID string = "vmr"
)

/*
show SDK list
*/
var ShowSDKCmd = &cobra.Command{
	Use:     "show",
	Aliases: []string{"S"},
	GroupID: GroupID,
	Short:   "Show available SDKs.",
	Long:    "Show the SDKs supported by VMR.",
	Run: func(cmd *cobra.Command, args []string) {
		l := cliui.NewSDKSearcher()
		l.Show()
	},
}
