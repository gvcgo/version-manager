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

package main

import (
	"github.com/gvcgo/version-manager/internal/tui/cmds"
)

func main() {
	// os.Setenv(conf.VMReverseProxyEnvName, "https://gvc.1710717.xyz/proxy/")
	// register.RunInstaller(register.VersionKeeper["python"])
	// fmt.Println(os.Environ())
	// pt := terminal.NewPtyTerminal("go")
	// pt.AddEnv("Hello", "test-test-test")
	// pt.Run()
	// fmt.Println("----hello")

	// c := exec.Command("zsh", `cd`, `~`)

	// c.Env = os.Environ()
	// c.Stdin = os.Stdin
	// c.Stdout = os.Stdout
	// err := c.Run()
	// fmt.Println(err)

	// _, err := gutils.ExecuteSysCommand(true, "", "conda", "--help")
	// fmt.Println(err)
	// l, _ := os.Readlink(filepath.Join(conf.GetVMVersionsDir("python"), "python"))
	// fmt.Println(l)

	// l := table.NewList()
	// l.Run()
	// cmds.ShowSDKNameList()
	ll := cmds.NewVMRSDKList()
	ll.ShowSDKList()
}
