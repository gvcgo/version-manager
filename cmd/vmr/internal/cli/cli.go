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

package cli

import (
	"fmt"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/spf13/cobra"
)

const (
	GroupID string = "vmr"
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
	c.rootCmd.AddCommand(searchCmd)
	c.rootCmd.AddCommand(listCmd)
	c.rootCmd.AddCommand(useCmd)
	c.rootCmd.AddCommand(uninstallCmd)
	c.rootCmd.AddCommand(localCmd)
	c.rootCmd.AddCommand(setProxyCmd)
	c.rootCmd.AddCommand(setReverseProxyCmd)
	c.rootCmd.AddCommand(installSelfCmd)
	c.rootCmd.AddCommand(clearCacheCmd)

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
