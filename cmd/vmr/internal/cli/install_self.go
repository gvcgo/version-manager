package cli

import (
	"github.com/gvcgo/version-manager/pkgs/self"
	"github.com/spf13/cobra"
)

var installSelfCmd = &cobra.Command{
	Use:     "install-self",
	Aliases: []string{"i", "is"},
	GroupID: GroupID,
	Short:   "Installs version manager.",
	Run: func(cmd *cobra.Command, args []string) {
		self.InstallVmr()
	},
}
