-- ConstantFolder: arithmetic, unary, concat, comparisons, logic, tables,
-- parens and string-number extraction (all Lua 5.1-compatible so every
-- preset parses it).
local a = 1 + 2
local b = 3 - 4
local c = 5 * 6
local d = 7 / 2
local e = 8 % 3
local f = 2 ^ 10
local g = -5
local h = -2.5
local i = not true
local j = not false
local k = #"hello"
local l = "foo" .. "bar"
local m = "x" .. 1
local n = 1 < 2
local o = 2 <= 2
local p = 3 > 4
local q = 4 >= 4
local r = 1 == 1
local s = 1 ~= 2
local t = "a" == "a"
local u = "a" < "b"
local v = true == false
local w = true and 5
local x = false or 7
local y = nil or "default"
local z = (1 + 2) * 3
local aa = ((4))
local ac = ({ y = 2 }).y
local af = "10" + 5
local ah = "1e2" + 1
local ai = 1.5 + 2
local aj = x + 1
return a
