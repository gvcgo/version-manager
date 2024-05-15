package cli

import (
	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/locker"
	"github.com/gvcgo/version-manager/pkgs/register"
	"github.com/spf13/cobra"
	"os"
	"strings"
)

var useCmd = &cobra.Command{
	Use:     "use",
	Aliases: []string{"u"},
	GroupID: GroupID,
	Short:   "Installs and switches to specified version.",
	Long:    "Example: vmr use go@1.22.1",
	Run: func(cmd *cobra.Command, args []string) {
		mirrorInChina, _ := cmd.Flags().GetBool("mirror_in_china")
		toLock, _ := cmd.Flags().GetBool("lock")
		// uses a version for current session only.
		sessionOnly, _ := cmd.Flags().GetBool("session-only")
		// enable locked version.
		elv, _ := cmd.Flags().GetBool("enable-locked-version")

		vlocker := locker.NewVLocker()
		lockedVersion := vlocker.Get()
		if elv && lockedVersion == "" {
			return
		}
		// Uses the locked version.
		if lockedVersion != "" && !toLock {
			args = []string{lockedVersion}
			sessionOnly = true
			alreadyLockedVersions := os.Getenv(conf.VMLockedVersionEnvName)
			if strings.Contains(alreadyLockedVersions, lockedVersion) {
				return
			} else {
				_ = os.Setenv(conf.VMLockedVersionEnvName, lockedVersion)
			}
		}

		if toLock {
			sessionOnly = true
		}
		// session only.
		_ = os.Setenv(conf.VMOnlyInCurrentSessionEnvName, gconv.String(sessionOnly))

		if len(args) == 0 || !strings.Contains(args[0], "@") {
			_ = cmd.Help()
			return
		}
		sList := strings.Split(args[0], "@")
		if len(sList) != 2 {
			_ = cmd.Help()
			return
		}
		threads, _ := cmd.Flags().GetInt("threads")
		_ = os.Setenv(conf.VMDownloadThreadsEnvName, gconv.String(threads))

		if mirrorInChina {
			_ = os.Setenv(conf.VMUseMirrorInChinaEnvName, "true")
		} else {
			_ = os.Setenv(conf.VMUseMirrorInChinaEnvName, "false")
		}

		if ins, ok := register.VersionKeeper[sList[0]]; ok {
			if toLock {
				// lock the sdk version for a project.
				vLocker := locker.NewVLocker()
				vLocker.Save(args[0])
				// enable hook for command 'cd'.
			}
			ins.SetVersion(sList[1])
			register.RunInstaller(ins)
		} else {
			gprint.PrintError("Unsupported app: %s.", sList[0])
		}
	},
}

func init() {
	useCmd.Flags().BoolP("lock", "l", false, "To lock the sdk version for current project.")
	useCmd.Flags().BoolP("enable-locked-version", "E", false, "To enable the locked version for current project.")
	useCmd.Flags().IntP("threads", "t", 1, "Number of threads for downloading.")
	useCmd.Flags().BoolP("mirror_in_china", "c", false, "To downlowd from mirror sites in China.")
	useCmd.Flags().BoolP("session-only", "s", false, "To use a version only for the current terminal session.")
}
