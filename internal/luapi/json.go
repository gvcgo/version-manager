package luapi

import (
	"github.com/gogf/gf/v2/encoding/gjson"
	"github.com/gogf/gf/v2/util/gconv"
	lua "github.com/yuin/gopher-lua"
)

func checkGJson(L *lua.LState) *gjson.Json {
	ud := L.CheckUserData(1)
	if j, ok := ud.Value.(*gjson.Json); ok {
		return j
	}
	L.ArgError(1, "gjson expected")
	return nil
}

func InitGJson(L *lua.LState) int {
	arg := L.ToUserData(1)
	if j, err := gjson.LoadJson(gconv.String(arg.Value)); err != nil {
		prepareResult(L, nil)
		return 0
	} else {
		j.SetViolenceCheck(true)
		prepareResult(L, j)
		return 1
	}
}

func GetGJsonString(L *lua.LState) int {
	j := checkGJson(L)
	if j == nil {
		return 0
	}

	jPath := L.ToString(2)
	if jPath == "" {
		return 0
	}
	res := j.Get(jPath).String()
	L.Push(lua.LString(res))
	return 1
}

func GetGJsonInt(L *lua.LState) int {
	j := checkGJson(L)
	if j == nil {
		return 0
	}

	jPath := L.ToString(2)
	if jPath == "" {
		return 0
	}
	res := j.Get(jPath).Int()
	L.Push(lua.LNumber(res))
	return 1
}

func GetGJsonMapEach(L *lua.LState) int {
	j := checkGJson(L)
	if j == nil {
		return 0
	}
	jPath := L.ToString(2)
	if jPath == "" {
		return 0
	}
	res := j.Get(jPath).Map()
	if res == nil {
		return 0
	}

	cb := L.ToFunction(3)
	for k, v := range res {
		ud := L.NewUserData()
		ud.Value = v
		if err := L.CallByParam(lua.P{
			Fn:      cb,
			NRet:    0,
			Protect: true,
		}, lua.LString(k), ud); err != nil {
			panic(err)
		}
	}
	return 0
}

func GetGJsonFromMapByKey(L *lua.LState) int {
	j := checkGJson(L)
	if j == nil {
		return 0
	}
	jPath := L.ToString(2)
	if jPath == "" {
		return 0
	}
	res := j.Get(jPath).Map()
	if res == nil {
		return 0
	}

	key := L.ToString(3)
	val := res[key]
	L.Push(lua.LString(gconv.String(val)))
	return 1
}

func GetGJsonSliceEach(L *lua.LState) int {
	j := checkGJson(L)
	if j == nil {
		return 0
	}

	jPath := L.ToString(2)
	if jPath == "" {
		return 0
	}

	res := j.Get(jPath).Array()

	if res == nil {
		return 0
	}

	cb := L.ToFunction(3)
	for idx, v := range res {
		if v == nil {
			continue
		}
		ud := L.NewUserData()
		ud.Value = v
		if err := L.CallByParam(lua.P{
			Fn:      cb,
			NRet:    0,
			Protect: true,
		}, lua.LNumber(idx+1), ud); err != nil {
			panic(err)
		}
	}
	return 0
}

func GetGJsonFromSliceByIndex(L *lua.LState) int {
	j := checkGJson(L)
	if j == nil {
		return 0
	}
	jPath := L.ToString(2)
	if jPath == "" {
		return 0
	}
	res := j.Get(jPath).Array()
	if res == nil {
		return 0
	}

	index := L.ToInt(3)
	if index < 1 || index > len(res) {
		return 0
	}
	val := res[index-1]
	L.Push(lua.LString(gconv.String(val)))
	return 1
}
