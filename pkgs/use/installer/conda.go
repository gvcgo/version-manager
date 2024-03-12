package installer

/*
TODO: Use miniconda as installer.
*/
type CondaInstaller struct {
	AppName string
	Version string
}

func NewCondaInstaller() *CondaInstaller {
	c := &CondaInstaller{
		AppName: "python",
		Version: "3.12",
	}
	return c
}

func (c *CondaInstaller) Download() (zipFilePath string) {
	return ""
}

func (c *CondaInstaller) Unzip(zipFilePath string) {}

func (c *CondaInstaller) Copy() {}

func (c *CondaInstaller) CreateVersionSymbol() {}
func (c *CondaInstaller) CreateBinarySymbol()  {}

func (c *CondaInstaller) SetEnv() {}
func (c *CondaInstaller) GetInstall() func(appName, version, zipFilePath string) {
	return nil
}
func (c *CondaInstaller) InstallApp(zipFilePath string) {}
func (c *CondaInstaller) UnInstallApp()                 {}
func (c *CondaInstaller) DeleteVersion()                {}
func (c *CondaInstaller) DeleteAll()                    {}
