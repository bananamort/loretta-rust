-- ConstantFolder hex-string extraction: the C# long.TryParse with
-- AllowLeadingSign | AllowHexSpecifier throws ArgumentException on .NET 10
-- (pinned behavior — the reference errors the whole op).
local ag = "0x10" + 1
