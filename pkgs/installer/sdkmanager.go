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

package installer

import "github.com/gvcgo/version-manager/pkgs/versions"

/*
Installs android sdk using sdkmanager.

1. cmdline-tools:
https://developer.android.google.cn/tools/releases/cmdline-tools?hl=en
2. build-tools
https://developer.android.com/tools/releases/build-tools?hl=en
3. platform-tools
https://developer.android.google.cn/tools/releases/platform-tools?hl=en
4. system-images
https://developer.android.google.cn/topic/generic-system-image/releases?hl=en
5. ndk
https://developer.android.google.cn/ndk?hl=en
6. add-ons
7. sources
https://developer.android.google.cn/reference/tools/gradle-api/8.5/com/android/build/api/variant/Sources?hl=en
8. extras
*/
type AndroidSDKInstaller struct {
	AppName   string
	Version   string
	Searcher  *SDKManagerSearcher
	V         *versions.VersionItem
	Install   func(appName, version, zipFilePath string)
	UnInstall func(appName, version string)
	HomePage  string
}

func NewAndroidSDKInstaller() (a *AndroidSDKInstaller) {
	a = &AndroidSDKInstaller{
		Searcher: NewSDKManagerSearcher(),
	}
	a.Install = a.InstallSDK
	a.UnInstall = a.UnInstallSDK
	return
}

func (a *AndroidSDKInstaller) InstallSDK(appName, version, zipFilePath string) {
}

func (a *AndroidSDKInstaller) UnInstallSDK(appName, version string) {
}
