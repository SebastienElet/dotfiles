assert(vim.v.errmsg == "", "Neovim reported an error: " .. vim.v.errmsg)

local config = require("lazy.core.config")
local inspected = 0

-- Lazy tasks retain failures without failing :Lazy! sync; API at 85c7ff3: lua/lazy/manage/task/init.lua.
local function verify_plugins(plugins)
  assert(type(plugins) == "table", "Lazy plugin inventory unavailable")
  for name, plugin in pairs(plugins) do
    assert(type(plugin._) == "table", "Lazy plugin state unavailable: " .. tostring(name))
    if plugin._.tasks ~= nil then
      assert(type(plugin._.tasks) == "table", "Lazy task inventory unavailable: " .. tostring(name))
      for _, task in pairs(plugin._.tasks) do
        assert(type(task.has_errors) == "function" and type(task.running) == "function", "Lazy task API unavailable")
        assert(task:running() == false, "Lazy task running state is not clear: " .. tostring(name))
        assert(task:has_errors() == false, "Lazy task did not succeed: " .. tostring(name))
        inspected = inspected + 1
      end
    end
  end
end

verify_plugins(config.plugins)
verify_plugins(config.to_clean)
assert(inspected > 0, "Lazy sync task evidence unavailable")
