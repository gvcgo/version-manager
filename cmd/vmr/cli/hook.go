package cli

import (
	"strings"

	"github.com/gvcgo/version-manager/cmd/vmr/cli/vcli"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/installer"
	"github.com/spf13/cobra"
)

/*
subcommand for cd hook.
*/
var useHookCmd = &cobra.Command{
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

		sdkName := vl[0]
		versionName := vl[1]

		vList := download.GetVersionList(sdkName, "")
		if len(vList) == 0 {
			return
		}

		vItem, ok := vList[versionName]
		if !ok {
			return
		}

		ins := installer.NewInstaller(sdkName, versionName, "", vItem)

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
	},
}

func init() {
	useHookCmd.Flags().BoolP("enable-locked-version", "E", false, "To enable the locked version for current project.")
	useHookCmd.Flags().BoolP("session-only", "s", false, "New a terminal session and add the specified version to the new session.")
	useHookCmd.Flags().BoolP("lock-version", "l", false, "Lock the specific version for an SDK.")
}
