pcall(require, "startup")

---@class utf8_rs
---@field check_is_valid          fun(str: string): (is_valid: boolean, str_or_error: string)
---@field get_chars               fun(str: string): chars: string[]
---@field get_graphemes           fun(str: string): graphemes: string[]
---@field get_codepoints          fun(str: string): codepoints: integer[]
---@field char_to_codepoint       fun(char: string): codepoint: integer
---@field codepoint_to_char       fun(codepoint: integer): char: string
---@field calculate_display_width fun(str: string, ignore_graphemes?: boolean, ignore_ansi_escapes?: boolean): display_width: integer
local utf8_rs = require("utf8_rs")
print(utf8_rs)

---@type string[]
local maybe_invalid_utf8 = {
	-- Lone continuation bytes (not valid as first byte)
	"\128", -- \x80
	"\129", "\130", "\131", "\132", "\133", "\134", "\135",
	"\136", "\137", "\138", "\139", "\140", "\141", "\142", "\143",
	"\144", "\145", "\146", "\147", "\148", "\149", "\150", "\151",
	"\152", "\153", "\154", "\155", "\156", "\157", "\158", "\159",
	"\160", "\161", "\162", "\163", "\164", "\165", "\166", "\167",
	"\168", "\169", "\170", "\171", "\172", "\173", "\174", "\175",
	"\176", "\177", "\178", "\179", "\180", "\181", "\182", "\183",
	"\184", "\185", "\186", "\187", "\188", "\189", "\190", "\191",

	-- Overlong encodings
	"\192\128",         -- overlong encoding of U+0000
	"\193\191",         -- overlong encoding, illegal
	"\224\160\128",     -- overlong encoding of U+0800
	"\240\144\128\128", -- overlong encoding of U+10000

	-- Invalid starting bytes
	"\255", -- \xFF is never valid in UTF-8
	"\254", -- \xFE is also never valid

	-- Truncated sequences
	"\194",         -- valid lead byte of 2-byte seq, no continuation
	"\224\160",     -- valid start of 3-byte seq, missing one byte
	"\240\144\128", -- valid start of 4-byte seq, missing one byte

	-- Invalid continuation bytes after valid starters
	"\194\255",         -- \xFF is not a valid continuation byte
	"\224\255\128",     -- \xFF in 3-byte sequence
	"\240\128\255\128", -- \xFF in 4-byte sequence

	"A\000B\000C\000D",

	"Hello, world!",
}

local char_map = {
	["\\"] = "\\\\",
	["\""] = "\\\"",
	["\'"] = "\\\'",
	["\a"] = "\\a",
	["\b"] = "\\b",
	["\f"] = "\\f",
	["\n"] = "\\n",
	["\r"] = "\\r",
	["\t"] = "\\t",
	["\v"] = "\\v",
}

---@param str string
---@return string
---@nodiscard
local function quote(str)
	local result = str:gsub("[%z\001-\031\127-\255\\\"\'\a\b\f\n\r\t\v]", function(char) ---@param char string
		local mapped_char = char_map[char]

		if mapped_char ~= nil then
			return mapped_char
		end

		local byte = char:byte()
		return ("\\%03d"):format(byte)
	end)

	return "\"" .. result .. "\""
end

--[ [
for _, bytes in ipairs(maybe_invalid_utf8) do
	print(quote(bytes), utf8_rs.check_is_valid(bytes))
end
--]]

do
	local str1 = "😀🤟🏼❤️👋👋🏻👋🏼👋🏽👋🏾👋🏿🍎🏳️‍🌈🏳️🏴🧑🏻‍🤝‍🧑🏿"
	print(utf8_rs.get_chars(str1))
	print(utf8_rs.get_graphemes(str1))
	print(utf8_rs.get_codepoints(str1))
	print(utf8_rs.char_to_codepoint(str1) == 128512)
	print(utf8_rs.codepoint_to_char(128512))
	print(utf8_rs.codepoint_to_char(0x10FFFF))
	--print(utf8_rs.codepoint_to_char(0x10FFFF + 1))

	local str2 = "Foo\027[1;31mBar🧑🏻‍🤝‍🧑🏿\027[0m"
	print(utf8_rs.calculate_display_width(str2) == 7)
	print(utf8_rs.calculate_display_width(str2, false) == 7)
	print(utf8_rs.calculate_display_width(str2, false, false) == 7)
	print(utf8_rs.calculate_display_width(str2, true,  false) == 13)
	print(utf8_rs.calculate_display_width(str2, false, true) == 18)
	print(utf8_rs.calculate_display_width(str2, true,  true) == 24)
end
