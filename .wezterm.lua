local wezterm = require("wezterm")

-- Crée une version de Catppuccin Latte avec meilleur contraste
local function create_custom_latte()
	local scheme = wezterm.color.get_builtin_schemes()["Catppuccin Latte"]

	-- Renforce significativement les gris/noirs pour meilleure lisibilité
	-- Les couleurs originales sont trop proches du fond blanc
	scheme.ansi[1] = "#4c4f69" -- black -> Text (le plus foncé)
	scheme.brights[1] = "#5c5f77" -- bright black -> Subtext1

	-- Renforce aussi les autres couleurs qui peuvent manquer de contraste
	scheme.ansi[8] = "#6c6f85" -- white (souvent utilisé pour texte secondaire) -> Subtext0
	scheme.brights[8] = "#4c4f69" -- bright white -> Text

	-- Foreground plus foncé pour le texte principal
	scheme.foreground = "#4c4f69" -- Text

	-- Sélection avec meilleur contraste
	scheme.selection_bg = "#bcc0cc" -- Surface1 - gris clair au lieu du marron
	scheme.selection_fg = "#4c4f69" -- Text - texte foncé

	return scheme
end

local function scheme_for_appearance(appearance)
	if appearance:find("Dark") then
		return "Catppuccin Mocha"
	else
		return "Catppuccin Latte Custom"
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
	color_schemes = {
		["Catppuccin Latte Custom"] = create_custom_latte(),
	},
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
