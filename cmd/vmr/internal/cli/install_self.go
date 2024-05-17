package cli

import (
	"github.com/gvcgo/version-manager/internal/shell"
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
		// migrate from old shell configration file to the new one.
		m := shell.NewShellMigrator()
		m.Migrate()
	},
}
