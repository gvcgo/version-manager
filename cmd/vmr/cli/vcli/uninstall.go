package vcli

import (
	"strings"

	"github.com/gvcgo/version-manager/internal/installer"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
	"github.com/spf13/cobra"
)

/*
Uninstalls a version for an SDK.
*/
var UninstallVersionCmd = &cobra.Command{
	Use:     "uninstall",
	Aliases: []string{"uni", "r"},
	GroupID: GroupID,
	Short:   "Uninstall versions for an SDK.",
	Long:    "Example: vmr uninstall sdkname@version or sdkname@all.",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			cmd.Help()
			return
		}
		versionInfo := args[0]
		if !strings.Contains(versionInfo, "@") {
			cmd.Help()
			return
		}
		sList := strings.Split(versionInfo, "@")
		if len(sList) != 2 || sList[1] == "" {
			cmd.Help()
			return
		}
		sdkName := sList[0]
		version := sList[1]

		if version == "all" {
			lif := installer.NewIVFinder(sdkName)
			lif.UninstallAllVersions()
		} else {
			pls := plugin.NewPlugins()
			pls.LoadAll()
			p := pls.GetPluginBySDKName(sdkName)

			v := plugin.NewVersions(p.PluginName)
			if v == nil {
				return
			}
			vItem := v.GetVersionByName(version)
			ins := installer.NewInstaller(p.SDKName, p.PluginName, version, vItem)
			ins.Uninstall()
		}
	},
}
