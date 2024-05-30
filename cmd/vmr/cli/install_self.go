package cli

import (
	"github.com/gvcgo/version-manager/internal/self"
	"github.com/spf13/cobra"
)

/*
Install vmr.

This subcommand is used by the installation script.
*/
var installSelfCmd = &cobra.Command{
	Use:     "install-self",
	Aliases: []string{"i", "is"},
	GroupID: GroupID,
	Short:   "Installs version manager.",
	Run: func(cmd *cobra.Command, args []string) {
		self.InstallSelf()
	},
}
