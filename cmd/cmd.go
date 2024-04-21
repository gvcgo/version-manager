/*
 @@    Copyright (c) 2024 moqsien@hotmail.com
 @@
 @@    Permission is hereby granted, free of charge, to any person obtaining a copy of
 @@    this software and associated documentation files (the "Software"), to deal in
 @@    the Software without restriction, including without limitation the rights to
 @@    use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 @@    the Software, and to permit persons to whom the Software is furnished to do so,
 @@    subject to the following conditions:
 @@
 @@    The above copyright notice and this permission notice shall be included in all
 @@    copies or substantial portions of the Software.
 @@
 @@    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 @@    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 @@    FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 @@    COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 @@    IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 @@    CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

package cmd

import (
	"fmt"
	"os"
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/envs"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/locker"
	"github.com/gvcgo/version-manager/pkgs/register"
	"github.com/gvcgo/version-manager/pkgs/self"
	"github.com/spf13/cobra"
)

const (
	GroupID string = "vmr"
)

/*
CLIs
*/
type Cli struct {
	rootCmd *cobra.Command
	groupID string
	gitTag  string
	gitHash string
}

func New(gitTag, gitHash string) (c *Cli) {
	c = &Cli{
		rootCmd: &cobra.Command{
			Short: "version manager",
			Long:  "vmr <Command> <SubCommand> --flags args...",
		},
		groupID: GroupID,
		gitTag:  gitTag,
		gitHash: gitHash,
	}
	c.rootCmd.AddGroup(&cobra.Group{ID: c.groupID, Title: "Command list: "})
	c.initiate()
	return
}

func (c *Cli) initiate() {
	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "search",
		Aliases: []string{"s"},
		GroupID: GroupID,
		Short:   "Shows the available versions of an application.",
		Long:    "Example: vmr search go.",
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) == 0 {
				cmd.Help()
				return
			}

			installer, ok := register.VersionKeeper[args[0]]
			if ok && installer != nil {
				installer.FixAppName()
				installer.SearchVersions()
			} else {
				gprint.PrintWarning("unsupported sdk-name!")
			}
		},
	})

	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "list",
		Aliases: []string{"l"},
		GroupID: GroupID,
		Short:   "Shows the supported applications.",
		Run: func(cmd *cobra.Command, args []string) {
			register.ShowAppList()
		},
	})

	useCmd := &cobra.Command{
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
					os.Setenv(conf.VMLockedVersionEnvName, lockedVersion)
				}
			}

			// session only.
			os.Setenv(conf.VMOnlyInCurrentSessionEnvName, gconv.String(sessionOnly))

			if len(args) == 0 || !strings.Contains(args[0], "@") {
				cmd.Help()
				return
			}
			sList := strings.Split(args[0], "@")
			if len(sList) != 2 {
				cmd.Help()
				return
			}
			threads, _ := cmd.Flags().GetInt("threads")
			os.Setenv(conf.VMDownloadThreadsEnvName, gconv.String(threads))

			if mirrorInChina {
				os.Setenv(conf.VMUseMirrorInChinaEnvName, "true")
			} else {
				os.Setenv(conf.VMUseMirrorInChinaEnvName, "false")
			}

			if ins, ok := register.VersionKeeper[sList[0]]; ok {
				ins.SetVersion(sList[1])
				register.RunInstaller(ins)
				if toLock {
					// lock the sdk version for a project.
					vlocker := locker.NewVLocker()
					vlocker.Save(args[0])
					// enable hook for command 'cd'.
					locker.AddCdHook()
				}
			} else {
				gprint.PrintError("Unsupported app: %s.", sList[0])
			}
		},
	}

	useCmd.Flags().BoolP("lock", "l", false, "To lock the sdk version for current project.")
	useCmd.Flags().BoolP("enable-locked-version", "E", false, "To enable the locked version for current project.")
	useCmd.Flags().IntP("threads", "t", 1, "Number of threads for downloading.")
	useCmd.Flags().BoolP("mirror_in_china", "c", false, "To downlowd from mirror sites in China.")
	useCmd.Flags().BoolP("session-only", "s", false, "To use a version only for the current terminal session.")
	c.rootCmd.AddCommand(useCmd)

	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "uninstall",
		Aliases: []string{"U"},
		GroupID: GroupID,
		Short:   "Uninstalls a version or an app.",
		Long:    "Example: 1. vmr U go@all; 2. vmr U go@1.22.1",
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) == 0 || !strings.Contains(args[0], "@") {
				cmd.Help()
				return
			}
			sList := strings.Split(args[0], "@")
			if len(sList) != 2 {
				cmd.Help()
				return
			}
			if ins, ok := register.VersionKeeper[sList[0]]; ok {
				ins.SetVersion(sList[1])
				register.RunUnInstaller(ins)
			} else {
				gprint.PrintError("Unsupported app: %s.", sList[0])
			}
		},
	})

	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "local",
		Aliases: []string{"L"},
		GroupID: GroupID,
		Short:   "Shows installed versions for an app.",
		Long:    "Example: vmr L go.",
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) == 0 {
				cmd.Help()
				return
			}
			register.ShowInstalled(args[0])
		},
	})

	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "set-proxy",
		Aliases: []string{"sp"},
		GroupID: GroupID,
		Short:   "Sets proxy for version manager.",
		Long:    "Example: vmr sp http://127.0.0.1:2023",
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) == 0 {
				cmd.Help()
				return
			}
			pUrl := args[0]
			conf.SaveConfigFile(&conf.Config{ProxyURI: pUrl})
		},
	})

	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "set-reverse-proxy",
		Aliases: []string{"sr", "srp"},
		GroupID: GroupID,
		Short:   "Sets reverse proxy for version manager.",
		Long:    "Example: vmr sr https://gvc.1710717.xyz/proxy/",
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) == 0 {
				cmd.Help()
				return
			}
			rPxy := args[0]
			conf.SaveConfigFile(&conf.Config{ReverseProxy: rPxy})
		},
	})

	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "install-self",
		Aliases: []string{"i", "is"},
		GroupID: GroupID,
		Short:   "Installs version manager.",
		Run: func(cmd *cobra.Command, args []string) {
			self.InstallVmr()
		},
	})

	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "clear-cache",
		Aliases: []string{"c", "cc"},
		GroupID: GroupID,
		Short:   "Clears cached zip files for an app.",
		Long:    "Example: vmr c <sdk-name>",
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) == 0 {
				cmd.Help()
				return
			}
			appName := args[0]
			if ins, ok := register.VersionKeeper[appName]; ok {
				register.RunClearCache(ins)
			}
		},
	})

	envHandler := &cobra.Command{
		Use:     "env",
		Aliases: []string{"e"},
		GroupID: GroupID,
		Short:   "Handles env manually.",
		Long:    "Example: vmr e <-r>",
		Run: func(cmd *cobra.Command, args []string) {
			enableRemove, _ := cmd.Flags().GetBool("remove")
			if enableRemove {
				envs.RemoveEnvManually()
			} else {
				envs.AddEnvManually()
			}
		},
	}
	envHandler.Flags().BoolP("remove", "r", false, "<false>(by default): adds env; <true>: removes env.")
	c.rootCmd.AddCommand(envHandler)

	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "version",
		Aliases: []string{"v"},
		GroupID: GroupID,
		Short:   "Shows version info of version-manager.",
		Run: func(cmd *cobra.Command, args []string) {
			if len(c.gitHash) > 7 {
				c.gitHash = c.gitHash[:7]
			}
			fmt.Printf("%s(%s)\n", c.gitTag, c.gitHash)
		},
	})
}

func (c *Cli) Run() {
	if err := c.rootCmd.Execute(); err != nil {
		gprint.PrintError("%+v", err)
	}
}
