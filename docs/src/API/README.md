# Lua API Reference

The following chapters contain a complete listing of the afseq Lua scripting API. The content has been auto-generated from the [LuaLS Type Definitions](https://github.com/emuell/afseq/tree/master/types/nerdo), so you can read and use the definition files directly too.

---

You can also use the LuaLS type definitions directly for autocompletion and to view the API documentation in e.g. vscode and other editors that support the [LuaLS language server](https://luals.github.io/). 

First install the [sumneko.lua vscode extension](https://luals.github.io/#vscode-install).

Then download a copy of the afseq type definitions folder and configure your workspace to use the files in your project. To do this, add the following to your project's `/.vscode/settings.json` file

```json
{
    "Lua.workspace.library": ["PATH/TO/RENOISE_DEFINITION_FOLDER"],
    "Lua.runtime.plugin": "PATH/TO/RENOISE_DEFINITION_FOLDER/plugin.lua"
}
```