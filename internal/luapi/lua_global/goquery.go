package lua_global

import (
	"strings"

	"github.com/PuerkitoBio/goquery"
	"github.com/gogf/gf/v2/util/gconv"
	lua "github.com/yuin/gopher-lua"
)

func prepareResult(L *lua.LState, result interface{}) {
	ud := L.NewUserData()
	ud.Value = result
	L.Push(ud)
}

func checkSelection(L *lua.LState) *goquery.Selection {
	ud := L.ToUserData(1)
	if s, ok := ud.Value.(*goquery.Selection); ok {
		return s
	}
	L.ArgError(1, "Selection expected")
	return nil
}

func initDocument(resp string) *goquery.Document {
	if resp == "" {
		return nil
	}
	doc, _ := goquery.NewDocumentFromReader(strings.NewReader(resp))
	return doc
}

/*
lua:
url = "https://www.bing.com"
timeout = 10
headers = { ["User-Agent"] ="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"}
resp = vmrGetResponse(url, timeout, headers)
selection = vmrInitSelection(resp, "li")
*/
func InitSelection(L *lua.LState) int {
	resp := L.ToUserData(1)
	if resp == nil {
		return 0
	}
	doc := initDocument(gconv.String(resp.Value))
	if doc == nil {
		prepareResult(L, nil)
		return 0
	}
	selector := L.ToString(2)
	s := doc.Find(selector)
	prepareResult(L, s)
	return 1
}

/*
lua:
url = "https://www.bing.com"
timeout = 10
headers = { ["User-Agent"] ="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"}
resp = vmrGetResponse(url, timeout, headers)
selection = vmrInitSelection(resp, "li")
selection = vmrFind(selection, "a")
*/
func Find(L *lua.LState) int {
	s := checkSelection(L)
	if s == nil {
		prepareResult(L, nil)
		return 0
	}
	selector := L.ToString(2)
	s = s.Find(selector)
	prepareResult(L, s)
	return 1
}

/*
lua:
url = "https://www.bing.com"
timeout = 10
headers = { ["User-Agent"] ="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"}
resp = vmrGetResponse(url, timeout, headers)
selection = vmrInitSelection(resp, "li")
selection = vmrEq(selection, 0)
*/
func Eq(L *lua.LState) int {
	s := checkSelection(L)
	if s == nil {
		prepareResult(L, nil)
		return 0
	}
	index := L.ToInt(2)
	s = s.Eq(index)
	prepareResult(L, s)
	return 1
}

/*
lua:
url = "https://www.bing.com"
timeout = 10
headers = { ["User-Agent"] ="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"}
resp = vmrGetResponse(url, timeout, headers)
selection = vmrInitSelection(resp, "a")
string = vmrAttr(selection, "href")
*/
func Attr(L *lua.LState) int {
	s := checkSelection(L)
	if s == nil {
		L.Push(lua.LString(""))
		return 1
	}
	attrName := L.ToString(2)
	value := s.AttrOr(attrName, "")
	L.Push(lua.LString(value))
	return 1
}

/*
lua:
url = "https://www.bing.com"
timeout = 10
headers = { ["User-Agent"] ="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"}
resp = vmrGetResponse(url, timeout, headers)
selection = vmrInitSelection(resp, "a")
string = vmrText(selection)
*/
func Text(L *lua.LState) int {
	s := checkSelection(L)
	if s == nil {
		prepareResult(L, nil)
		return 0
	}
	value := s.Text()
	L.Push(lua.LString(value))
	return 1
}

/*
lua:
url = "https://www.bing.com"
timeout = 10
headers = { ["User-Agent"] ="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"}
resp = vmrGetResponse(url, timeout, headers)
s = initSelection(resp, "li")
function parseLiItem(i, ss)

	local node = vmrFind(ss, "a")
	local href = vmrAttr(node, "href")
	print(href)

end
vmrEach(s, parseLiItem)
*/
func Each(L *lua.LState) int {
	s := checkSelection(L)
	if s == nil {
		prepareResult(L, nil)
		return 0
	}

	cb := L.ToFunction(2)
	s.Each(func(i int, ss *goquery.Selection) {
		ud := L.NewUserData()
		ud.Value = ss
		if err := L.CallByParam(lua.P{
			Fn:      cb,
			NRet:    0,
			Protect: true,
		}, lua.LNumber(i), ud); err != nil {
			panic(err)
		}
	})
	return 0
}
