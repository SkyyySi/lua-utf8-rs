use mlua::prelude::*;
use regex::Regex;
use once_cell::sync::Lazy;
use unicode_segmentation::UnicodeSegmentation;

macro_rules! namespace {
	($lua:expr, [ $($name:ident),* $(,)? ] $(,)?) => {
		{
			let ns: ::mlua::Table = ($lua).create_table()?;

			$(
				ns.set(stringify!($name), ($lua).create_function($name)?)?;
			)*

			::mlua::Result::<::mlua::Table>::Ok(ns)
		}
	};
}

macro_rules! multi_value {
	($lua:expr, [ $($e:expr),* $(,)? ] $(,)?) => {
		{
			let mut values = ::std::vec::Vec::<::mlua::Value>::new();

			$(
				values.push(($e).into_lua($lua)?);
			)*

			::mlua::Result::<::mlua::MultiValue>::Ok(::mlua::MultiValue::from_vec(values))
		}
	};
}

#[allow(unused)]
macro_rules! match_or_bail {
	() => {
		todo!()
	};
}

#[allow(unused, non_snake_case)]
fn check_is_valid_NEW_TEMP<I: Iterator<Item = u8>>(mut iter: I) -> bool {
	let mut needed = 0;

	while let Some(byte) = iter.next() {
		if needed == 0 {
			match byte {
				0x00..=0x7F => continue, // ASCII
				0xC2..=0xDF => needed = 1,
				0xE0..=0xEF => {
					needed = 2;
					// Check for overlongs and surrogates
					let next = match iter.next() {
						Some(b) => b,
						None => return false,
					};
					match byte {
						0xE0 if next < 0xA0 || next > 0xBF => return false,
						0xED if next > 0x9F => return false,
						0xE1..=0xEC | 0xEE..=0xEF if next < 0x80 || next > 0xBF => return false,
						_ => {}
					}
					needed = 1;
					continue;
				}
				0xF0..=0xF4 => {
					needed = 3;
					let next = match iter.next() {
						Some(b) => b,
						None => return false,
					};
					match byte {
						0xF0 if next < 0x90 || next > 0xBF => return false,
						0xF4 if next > 0x8F => return false,
						0xF1..=0xF3 if next < 0x80 || next > 0xBF => return false,
						_ => {}
					}
					needed = 2;
					continue;
				}
				_ => return false, // invalid first byte
			}
		} else {
			if byte < 0x80 || byte > 0xBF {
				return false;
			}
			needed -= 1;
		}
	}

	needed == 0
}

fn check_is_valid(lua: &Lua, string: LuaString) -> LuaResult<LuaMultiValue> {
	let result: LuaResult<mlua::BorrowedStr<'_>> = string.to_str();

	multi_value![
		lua,
		[result.is_ok(),
		match result {
			Ok(_) => string.into_lua(lua)?,
			Err(error) => error.to_string().into_lua(lua)?,
		},]
	]
}

fn lua_string_format<T: IntoLua>(lua: &Lua, string: &str, args: impl IntoIterator<Item = T>) -> LuaResult<String> {
	let string_format: LuaFunction = lua
		.globals()
		.raw_get::<LuaTable>("string")?
		.raw_get::<LuaFunction>("format")?;

	let mut full_args: Vec<LuaValue> = args
		.into_iter()
		.map(|i| i.into_lua(lua))
		.collect::<LuaResult<Vec<LuaValue>>>()?;

	full_args.insert(0, string.to_string().into_lua(lua)?);

	Ok(string_format.call::<String>(LuaMultiValue::from_vec(full_args))?)
}

fn new_lua_array<T: IntoLua>(lua: &Lua, data: impl IntoIterator<Item = T>) -> LuaResult<LuaTable> {
	let result: LuaTable = lua.create_sequence_from(data)?;

	let meta: LuaTable = lua.create_table()?;
	meta.set("__tostring", lua.create_function(|lua: &Lua, table: LuaTable| -> LuaResult<String> {
		if table.is_empty() {
			return Ok("{}".to_string());
		}

		let mut values: Vec<String> = Vec::new();
		table.for_each(|_: LuaValue, value: LuaValue| {
			values.push(match value {
				LuaValue::String(s) => lua_string_format(lua, "%q", vec![s])?,
				v => v.to_string()?,
			});
			Ok(())
		})?;

		Ok(format!("{{ {} }}", values.join(", ")))
	})?)?;
	result.set_metatable(Some(meta));

	Ok(result)
}

fn get_chars(lua: &Lua, string: String) -> LuaResult<LuaTable> {
	new_lua_array(lua, string.chars().map(|c| c.to_string()))
}

fn get_graphemes(lua: &Lua, string: String) -> LuaResult<LuaTable> {
	new_lua_array(lua, string.graphemes(true).map(|g| g.to_owned()))
}

fn get_codepoints(lua: &Lua, string: String) -> LuaResult<LuaTable> {
	new_lua_array(lua, string.chars().map(|c| c as u32))
}

fn char_to_codepoint(_lua: &Lua, string: String) -> LuaResult<LuaInteger> {
	match string.chars().collect::<Vec<char>>().get(0) {
		Some(value) => Ok((*value as u32).into()),
		None => Err(LuaError::runtime(format!("Got an empty string!"))),
	}
}

fn codepoint_to_char(_lua: &Lua, codepoint: LuaInteger) -> LuaResult<String> {
	match char::from_u32(codepoint as u32) {
		Some(c) => Ok(c.to_string()),
		None => Err(LuaError::runtime(format!("Invalid Unicode codepoint! (0..1114111 expected, got: {codepoint})"))),
	}
}

static ANSI_ESCAPE_PATTERN: Lazy<Regex> = Lazy::new(|| {
	Regex::new(r"\x1b\[[0-9]+(?:;[0-9]+)*[a-zA-Z]")
		.expect("Failed to compile Regex pattern for matching ANSI escape sequences!")
});

fn calculate_display_width(_lua: &Lua, args: (String, Option<bool>, Option<bool>)) -> LuaResult<LuaInteger> {
	let string: String = args.0;
	let ignore_graphemes: bool = args.1.unwrap_or(false);
	let ignore_ansi_escapes: bool = args.2.unwrap_or(false);

	let mut result: LuaInteger = (if ignore_graphemes {
		string.chars().count()
	} else {
		string.graphemes(true).count()
	}) as i64;

	if !ignore_ansi_escapes {
		if let Some(combined_ansi_escape_lengths) = ANSI_ESCAPE_PATTERN.find_iter(string.as_str()).map(|m| m.len()).reduce(|acc, e| acc + e) {
			result -= combined_ansi_escape_lengths as LuaInteger;
		}
	}

	Ok(result)
}

#[mlua::lua_module]
fn utf8_rs(lua: &Lua) -> LuaResult<LuaTable> {
	let ns = namespace!(
		lua,
		[check_is_valid,
		get_chars,
		get_graphemes,
		get_codepoints,
		char_to_codepoint,
		codepoint_to_char,
		calculate_display_width,]
	)?;

	Ok(ns)
}
