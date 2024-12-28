package vcli

import (
	"github.com/gvcgo/version-manager/internal/completions"
	"github.com/spf13/cobra"
)

/*
Setup auto-completions for vmr commands.
*/

var SetupAutoCompletions = &cobra.Command{
	Use:     "add-completions",
	Aliases: []string{"ac"},
	GroupID: GroupID,
	Short:   "Add auto-completions for vmr to current shell profile.",
	Long:    "Example: vmr ac.",
	Run: func(cmd *cobra.Command, args []string) {
		completions.AddCompletionScriptToShellProfile()
	},
}
