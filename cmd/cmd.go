package cmd

import (
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/pkgs/search"
	"github.com/spf13/cobra"
)

const (
	GroupID string = "vm"
)

/*
CLIs
*/
type Cli struct {
	rootCmd *cobra.Command
	groupID string
}

func New() (c *Cli) {
	c = &Cli{
		rootCmd: &cobra.Command{
			Short: "version manager",
			Long:  "vm <Command> <SubCommand> --flags args...",
		},
		groupID: GroupID,
	}
	c.rootCmd.AddGroup(&cobra.Group{ID: c.groupID, Title: "Command list: "})
	c.initiate()
	return
}

func (c *Cli) initiate() {
	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "search",
		Aliases: []string{"s"},
		Short:   "Shows the available version list of an app.",
		Long:    "Example: vm search go.",
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) == 0 {
				cmd.Help()
				return
			}
			sch := search.NewSearcher()
			sch.Search(args[0])
		},
	})

	c.rootCmd.AddCommand(&cobra.Command{
		Use:     "show",
		Aliases: []string{"sh", "S"},
		Short:   "Shows the supported app list.",
		Run: func(cmd *cobra.Command, args []string) {
			search.ShowAppList()
		},
	})
}

func (that *Cli) Run() {
	if err := that.rootCmd.Execute(); err != nil {
		gprint.PrintError("%+v", err)
	}
}
