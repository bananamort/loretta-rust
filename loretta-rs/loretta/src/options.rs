// Ported from Loretta.CodeAnalysis.Lua LuaParseOptions / LuaSyntaxOptions / Operations (b767b4e):
// ADAPT to full_moon::ast::LuaVersion.
// NOTE: GMod (GLua &&/||/!=/! and //, /* */) is intentionally DROP per docs/AGENTS.md:24 — no local parser maintenance.
// Mapping: Lua51->LuaVersion::lua51(), Lua52->lua52(), Lua53->lua53(), Lua54->lua54(), LuaJIT->luajit(), Luau/Roblox->luau(), FiveM/CfxLua->cfxlua(), GMod/All -> unsupported (use luau+cfxlua or error).
