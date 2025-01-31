package luapi

import (
	"github.com/PuerkitoBio/goquery"
	lua "github.com/yuin/gopher-lua"
)

func checkGoQuery(L *lua.LState) *Crawler {
	ud := L.ToUserData(1)
	if g, ok := ud.Value.(*Crawler); ok {
		return g
	}
	L.ArgError(1, "GoQuery expected")
	return nil
}

func checkSelection(L *lua.LState) *goquery.Selection {
	ud := L.ToUserData(1)
	if s, ok := ud.Value.(*goquery.Selection); ok {
		return s
	}
	L.ArgError(1, "Selection expected")
	return nil
}

func prepareResult(L *lua.LState, result interface{}) {
	ud := L.NewUserData()
	ud.Value = result
	L.Push(ud)
}

func InitSelection(L *lua.LState) int {
	q := checkGoQuery(L)
	if q == nil {
		prepareResult(L, nil)
		return 0
	}
	doc := GetDocument(q.Url, q.Timeout)
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
