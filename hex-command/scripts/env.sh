# Source from Bash for direct Cargo/Dune commands. No installation or profile edits.
# Project commands discover these tools automatically.
_hex_tool_exports=$(node "$(dirname "${BASH_SOURCE[0]}")/toolchain/toolchain-env.mjs" --shell) || return
eval "$_hex_tool_exports"
unset _hex_tool_exports
