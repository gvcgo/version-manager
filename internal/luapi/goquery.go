package luapi

import (
	"strings"

	"github.com/PuerkitoBio/goquery"
	"github.com/gogf/gf/v2/util/gconv"
	lua "github.com/yuin/gopher-lua"
)

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

func Attr(L *lua.LState) int {
	s := checkSelection(L)
	if s == nil {
		prepareResult(L, nil)
		return 0
	}
	attrName := L.ToString(2)
	value := s.AttrOr(attrName, "")
	L.Push(lua.LString(value))
	return 1
}

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
