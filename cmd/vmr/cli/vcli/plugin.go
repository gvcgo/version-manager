package vcli

import (
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
	"github.com/spf13/cobra"
)

var UpdatePlugins = &cobra.Command{
	Use:     "update-plugins",
	Aliases: []string{"up"},
	GroupID: GroupID,
	Short:   "Update plugins for vmr.",
	Long:    "Example: vmr up",
	Run: func(cmd *cobra.Command, args []string) {
		err := plugin.UpdatePlugins()
		if err != nil {
			gprint.PrintError("Update plugins failed: %s", err)
		}
	},
}
