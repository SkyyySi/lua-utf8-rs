---@meta utf8_rs

---@class utf8_rs
local _M = {}

---@param str string
---@return true   | false  is_valid
---@return string | string str_or_error
function _M.check_is_valid(str) end

---@param str string
---@return string[] chars
function _M.get_chars(str) end

---@param str string
---@return string[] graphemes
function _M.get_graphemes(str) end

---@param str string
---@return integer[] codepoints
function _M.get_codepoints(str) end

---@param char string
---@return integer codepoint
function _M.char_to_codepoint(char) end

---@param codepoint integer
---@return string char
function _M.codepoint_to_char(codepoint) end

---@param str                  string
---@param ignore_graphemes?    boolean
---@param ignore_ansi_escapes? boolean
---@return integer display_width
function _M.calculate_display_width(str, ignore_graphemes, ignore_ansi_escapes) end

return _M
