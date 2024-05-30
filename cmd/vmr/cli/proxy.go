package cli

import (
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/spf13/cobra"
)

var setProxyCmd = &cobra.Command{
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

		cnf.DefaultConfig.SetProxyUri(pUrl)
	},
}

var setReverseProxyCmd = &cobra.Command{
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
		cnf.DefaultConfig.SetReverseProxy(rPxy)
	},
}
