package cli

import (
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/pkgs/register"
	"github.com/spf13/cobra"
	"strings"
)

var uninstallCmd = &cobra.Command{
	Use:     "uninstall",
	Aliases: []string{"U"},
	GroupID: GroupID,
	Short:   "Uninstalls a version or an app.",
	Long:    "Example: 1. vmr U go@all; 2. vmr U go@1.22.1",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 || !strings.Contains(args[0], "@") {
			_ = cmd.Help()
			return
		}
		sList := strings.Split(args[0], "@")
		if len(sList) != 2 {
			_ = cmd.Help()
			return
		}
		if ins, ok := register.VersionKeeper[sList[0]]; ok {
			ins.SetVersion(sList[1])
			register.RunUnInstaller(ins)
		} else {
			gprint.PrintError("Unsupported app: %s.", sList[0])
		}
	},
}
