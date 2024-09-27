package cli

import (
	"github.com/gvcgo/version-manager/cmd/vmr/cli/vcli"
	"github.com/gvcgo/version-manager/internal/self"
	"github.com/spf13/cobra"
)

/*
Install vmr.

This subcommand is used by the installation script.
*/
var unInstallSelfCmd = &cobra.Command{
	Use:     "uninstall-self",
	Aliases: []string{"Uins"},
	GroupID: vcli.GroupID,
	Short:   "Uninstalls version manager, Only for script.",
	Run: func(cmd *cobra.Command, args []string) {
		self.RemoveCurrentVersion()
	},
}
