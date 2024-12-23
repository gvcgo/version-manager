package cli

import (
	"fmt"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/cmd/vmr/cli/vcli"
	"github.com/gvcgo/version-manager/internal/tui/cmds"
	"github.com/spf13/cobra"
)

// Cli is a commander
type Cli struct {
	rootCmd *cobra.Command
	groupID string
	gitTag  string
	gitHash string
}

func New(gitTag, gitHash string) (c *Cli) {
	c = &Cli{
		rootCmd: &cobra.Command{
			Use:   "vmr",
			Short: "version manager",
			Long:  "vmr <Command> <SubCommand> --flags args...",
			Run: func(cmd *cobra.Command, args []string) {
				ll := cmds.NewTUI()
				ll.ListSDKName()
			},
		},
		groupID: vcli.GroupID,
		gitTag:  gitTag,
		gitHash: gitHash,
	}
	c.rootCmd.AddGroup(&cobra.Group{ID: c.groupID, Title: "Command list: "})
	c.initiate()
	return
}

func (c *Cli) initiate() {
	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "version",
		Aliases: []string{"v"},
		GroupID: vcli.GroupID,
		Short:   "Shows version info of version-manager.",
		Run: func(cmd *cobra.Command, args []string) {
			if len(c.gitHash) > 7 {
				c.gitHash = c.gitHash[:7]
			}
			fmt.Printf("%s(%s)\n", c.gitTag, c.gitHash)
		},
	})

	c.rootCmd.AddCommand(setProxyCmd)
	c.rootCmd.AddCommand(setReverseProxyCmd)
	c.rootCmd.AddCommand(useCmd)
	c.rootCmd.AddCommand(installSelfCmd)
	c.rootCmd.AddCommand(unInstallSelfCmd)
	c.rootCmd.AddCommand(setDownloadThreads)
	c.rootCmd.AddCommand(toggleCustomedMirrors)

	// For CLIs
	c.rootCmd.AddCommand(vcli.ShowSDKCmd)
	c.rootCmd.AddCommand(vcli.SearchVersionsCmd)
	c.rootCmd.AddCommand(vcli.ShowInstalledCmd)
	c.rootCmd.AddCommand(vcli.ShowInstalledSDKs)
	c.rootCmd.AddCommand(vcli.ShowInstalledSDKInfo)
	c.rootCmd.AddCommand(vcli.UninstallVersionCmd)
}

func (c *Cli) Run() {
	if err := c.rootCmd.Execute(); err != nil {
		gprint.PrintError("%+v", err)
	}
}
