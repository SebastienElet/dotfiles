local wezterm = require("wezterm")

local function scheme_for_appearance(appearance)
	if appearance:find("Dark") then
		return "Catppuccin Mocha"
	else
		return "Catppuccin Latte"
	end
end

wezterm.on("window-config-reloaded", function(window)
	local overrides = window:get_config_overrides() or {}
	local appearance = window:get_appearance()
	local scheme = scheme_for_appearance(appearance)
	if overrides.color_scheme ~= scheme then
		overrides.color_scheme = scheme
		window:set_config_overrides(overrides)
	end
end)

return {
	font = wezterm.font("Iosevka Nerd Font"),
	font_size = 13.0,
	color_scheme = "Catppuccin Mocha",
	hide_tab_bar_if_only_one_tab = true,
	scrollback_lines = 200000,
	window_padding = {
		left = 4,
		right = 4,
		top = 4,
		bottom = 4,
	},
	default_cursor_style = "BlinkingBlock",
	cursor_blink_rate = 500,
	enable_wayland = false,
}
