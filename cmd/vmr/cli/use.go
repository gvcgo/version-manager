package cli

import (
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/cmd/vmr/cli/vcli"
	"github.com/gvcgo/version-manager/internal/installer"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
	"github.com/spf13/cobra"
)

/*
subcommand for cd hook.
*/
var useCmd = &cobra.Command{
	Use:     "use",
	Aliases: []string{"u", "h"},
	GroupID: vcli.GroupID,
	Short:   "Installs and switches to a version for an SDK.",
	Long:    "Example: vmr use sdkname@version.",
	Run: func(cmd *cobra.Command, args []string) {

		// enable locked version.
		if ok, _ := cmd.Flags().GetBool("enable-locked-version"); ok {
			l := installer.NewVLocker()
			l.HookForCdCommand()
			return
		}

		if len(args) == 0 {
			cmd.Help()
			return
		}

		verInfo := args[0]
		if !strings.Contains(verInfo, "@") {
			cmd.Help()
			return
		}

		vl := strings.Split(verInfo, "@")
		if len(vl) != 2 {
			cmd.Help()
			return
		}

		pluginName := vl[0]
		versionName := vl[1]

		ok2, _ := cmd.Flags().GetBool("install-by-conda")

		versions := plugin.NewVersions(pluginName)
		if versions == nil {
			gprint.PrintError("No Versions Found.")
			return
		}

		vList := versions.GetSdkVersions()

		if len(vList) == 0 && !ok2 {
			gprint.PrintError("No SDK Found.")
			return
		}

		vItem, ok := vList[versionName]

		if !ok && !ok2 {
			gprint.PrintError("No Versions Found.")
			return
		} else if ok2 {
			vItem = lua_global.Item{
				Arch:      runtime.GOARCH,
				Os:        runtime.GOOS,
				Installer: lua_global.Conda,
			}
		}

		ins := installer.NewInstaller(pluginName, versionName, "", vItem)

		if ok2 {
			// If an SDK is installed by Conda only, and it is not supported by VMR yet,
			// VMR will not know how to add Envs for this SDK.
			ins.DisableEnvs()
		}

		if ok, _ := cmd.Flags().GetBool("session-only"); ok {
			// use a version only for current session.
			ins.SetInvokeMode(installer.ModeSessionly)
		} else if ok, _ := cmd.Flags().GetBool("lock-version"); ok {
			// lock a version.
			ins.SetInvokeMode(installer.ModeToLock)
		} else {
			// use a version globally.
			ins.SetInvokeMode(installer.ModeGlobally)
		}

		ins.Install()

		if ok2 {
			sdkInstaller := ins.GetSDKInstaller()
			gprint.PrintInfo("The SDK is installed by Conda in %s, you need to add Envs by yourself.", sdkInstaller.GetSymbolLinkPath())
		}

	},
}

func init() {
	useCmd.Flags().BoolP("enable-locked-version", "E", false, "To enable the locked version for current project.")
	useCmd.Flags().BoolP("session-only", "s", false, "New a terminal session and add the specified version to the new session.")
	useCmd.Flags().BoolP("lock-version", "l", false, "Lock the specific version for an SDK.")
	useCmd.Flags().BoolP("install-by-conda", "c", false, "Install an SDK by Conda, and you need to add Envs by your self.")
}
