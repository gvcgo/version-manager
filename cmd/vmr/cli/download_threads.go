package cli

import (
	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/version-manager/cmd/vmr/cli/vcli"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/spf13/cobra"
)

var setDownloadThreads = &cobra.Command{
	Use:     "set-download-threads",
	Aliases: []string{"sdt", "st"},
	GroupID: vcli.GroupID,
	Short:   "Set default threads number for downloadding.",
	Long:    "Example: vmr st 2",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			cmd.Help()
			return
		}
		cnf.DefaultConfig.SetDownloadThreadNum(gconv.Int(args[0]))
	},
}
