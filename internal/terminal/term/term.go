package term

type Terminal interface {
	Record(command string, envs ...string) error
}
