package utils

import (
	"bytes"
	"context"
	"os"
	"os/exec"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
)

/*
Run a system command.
*/
type SysCommandRunner struct {
	cmd           *exec.Cmd
	collectOutput bool
	workDir       string
	args          []string
	output        *bytes.Buffer
	cancel        context.CancelFunc
}

func NewSysCommandRunner(collectOutput bool, workDir string, args ...string) *SysCommandRunner {
	return &SysCommandRunner{
		collectOutput: collectOutput,
		workDir:       workDir,
		args:          args,
	}
}

func (s *SysCommandRunner) Run() error {
	ctx := context.Background()
	ctx, s.cancel = context.WithCancel(ctx)
	if runtime.GOOS == gutils.Windows {
		s.args = append([]string{"/c"}, s.args...)
		s.cmd = exec.CommandContext(ctx, "cmd", s.args...)
	} else {
		gutils.FlushPathEnvForUnix()
		s.cmd = exec.CommandContext(ctx, s.args[0], s.args[1:]...)
	}

	s.cmd.Env = os.Environ()
	s.output = &bytes.Buffer{}
	if s.collectOutput {
		s.cmd.Stdout = s.output
	} else {
		s.cmd.Stdout = os.Stdout
	}
	if s.workDir != "" {
		s.cmd.Dir = s.workDir
	}
	s.cmd.Stderr = os.Stderr
	s.cmd.Stdin = os.Stdin
	return s.cmd.Run()
}

func (s *SysCommandRunner) Cancel() {
	if s.cancel != nil {
		s.cancel()
	}
}

func (s *SysCommandRunner) GetOutput() string {
	if s.output == nil {
		return ""
	}
	return s.output.String()
}
