package conf

type Config struct {
	ProxyUrl   string `json:"proxy"`
	ReverseUrl string `json:"reverse"`
}

func NewConfig() *Config {
	return &Config{
		ReverseUrl: "https://gvc.1710717.xyz/proxy/",
	}
}
