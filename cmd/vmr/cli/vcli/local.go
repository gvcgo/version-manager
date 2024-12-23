package vcli

import (
	"fmt"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
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

var ShowInstalledSDKs = &cobra.Command{
	Use:     "installed-sdks",
	Aliases: []string{"in"},
	GroupID: GroupID,
	Short:   "Shows installed SDKs.",
	Long:    "Example: vmr in.",
	Run: func(cmd *cobra.Command, args []string) {
		l := cliui.NewSDKSearcher()
		l.PrintInstalledSDKs()
	},
}

var ShowInstalledSDKInfo = &cobra.Command{
	Use:     "installed-info",
	Aliases: []string{"ii"},
	GroupID: GroupID,
	Short:   "Shows installed SDK information.",
	Long:    "Example: vmr ii.",
	Run: func(cmd *cobra.Command, args []string) {
		l := cliui.NewSDKSearcher()
		installedSDKList := l.GetInstalledSDKList()
		for _, sdkName := range installedSDKList {
			ll := cliui.NewLocalInstalled()
			ll.Search(sdkName)
			versionListString := gprint.YellowStr("%s", sdkName) + ": " + gprint.CyanStr("%s", strings.Join(ll.VersionList, ","))
			fmt.Println(versionListString)
		}
	},
}
