#!/bin/bash
set -e -o pipefail

pushd "$(dirname "$0")"

LUA_LS_VERSION="3.9.3"
LUA_LS_RELEASES_URL="https://github.com/LuaLS/lua-language-server/releases/download"

echo "downloading lua-language-server-${LUA_LS_VERSION}..."
rm -rf lua-language-server && mkdir lua-language-server

pushd lua-language-server

# windows (wsl)
if [[ $(grep -i Microsoft /proc/version) ]]; then
  wget -q "${LUA_LS_RELEASES_URL}/${LUA_LS_VERSION}/lua-language-server-${LUA_LS_VERSION}-win32-x64.zip" -O temp.zip
  unzip temp.zip && rm temp.zip
# macOS
elif [ "$(uname)" = 'Darwin' ]; then
  wget -q -O - "${LUA_LS_RELEASES_URL}/${LUA_LS_VERSION}/lua-language-server-${LUA_LS_VERSION}-darwin-arm64.tar.gz" | tar xz
# linux
else
  wget -q -O - "${LUA_LS_RELEASES_URL}/${LUA_LS_VERSION}/lua-language-server-${LUA_LS_VERSION}-linux-x64.tar.gz" | tar xz
fi

echo "patching files..."
sed -i.bak "s/\(\['Lua.hover.enumsLimit'\]\s*=\s*Type.Integer\s*>>\s*\)5\(,\)/\1100\2/" "script/config/template.lua"
sed -i.bak -e "s/\(\['Lua.hover.expandAlias'\]\s*=\s*Type.Boolean\s*>>\s*\)true\(,\)/\1false\2/" "script/config/template.lua"
sed -i.bak -e '/if \#view > 200 then/,/end/s/^/-- /' "script/vm/infer.lua"

popd # lua-language-server

popd # docs
