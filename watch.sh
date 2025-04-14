#!/usr/bin/env bash
declare script_dir=''
script_dir="$(realpath --physical -- "$(dirname -- "${BASH_SOURCE[0]}")")"
cd "$script_dir" || exit 1

eval "$(luarocks --lua-version='5.1' path)" || exit 1

################################################################################

clear

printf '>>> Watching for changes...\n'

declare file=''
declare -i terminal_width=0
declare separator=''
while true; do
	file="$(realpath --relative-base "$script_dir" -- "$(inotifywait \
		--quiet \
		--event 'modify' \
		--format '%w/%f' \
		--recursive \
		--include '.*\.(rs|lua)' \
		-- './'
	)")"

	clear

	printf '>>> File: %q\n' "$file"

	terminal_width="$(( ${COLUMNS:-$(tput cols)} - 1 ))"
	separator="$(printf '\e[2m%*s\e[0m\n' "$terminal_width" '' | sed 's/ /─/g')"
	printf '%s\n' "$separator"

	if ! cargo build; then
		continue
	fi

	printf '%s\n' "$separator"
	luajit -- "${script_dir}/test.lua"
done
