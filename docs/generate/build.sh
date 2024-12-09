#!/bin/bash
set -e

LUA_LS_VERSION=$1

pushd generate
echo "downloading lua-language-server-${LUA_LS_VERSION}..."
rm -rf lua-language-server && mkdir lua-language-server

pushd lua-language-server
rm -rf ./*
wget -q -O - "https://github.com/LuaLS/lua-language-server/releases/download/${LUA_LS_VERSION}/lua-language-server-${LUA_LS_VERSION}-linux-x64.tar.gz" | tar xz

echo "patching files..."
sed -i.bak "s/\(\['Lua.hover.enumsLimit'\]\s*=\s*Type.Integer\s*>>\s*\)5\(,\)/\1100\2/" "script/config/template.lua"
sed -i.bak -e "s/\(\['Lua.hover.expandAlias'\]\s*=\s*Type.Boolean\s*>>\s*\)true\(,\)/\1false\2/" "script/config/template.lua"
sed -i.bak -e '/if \#view > 200 then/,/end/s/^/-- /' "script/vm/infer.lua"
popd # lua-language-server

echo "generating api docs..."
cargo run -- "../../types/nerdo/library" "../src"