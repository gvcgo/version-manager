package lua_global

import "testing"

var githubScript = `print("------------github------------")
getGithubRelease("oven-sh/bun")`

func TestGithub(t *testing.T) {
	ll := NewLua()
	defer ll.Close()
	L := ll.GetLState()

	if err := L.DoString(githubScript); err != nil {
		t.Error(err)
	}
}
