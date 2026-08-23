# ADR-013 — WezTerm comme émulateur de terminal

- **Statut** : accepté
- **Date** : 2026-01
- **Commits** : `dc58857`, `c1ce30c`, `a372ba5`, `61d847b`

## Contexte

iTerm2 puis Alacritty ont précédé. iTerm2 se configure par interface graphique
et par un plist difficile à versionner ; Alacritty, versionnable, ne gère ni
les ligatures ni certains besoins de rendu, et son format de configuration a
changé plusieurs fois.

## Décision

Adopter WezTerm, configuré par un `home/.config/wezterm/wezterm.lua` versionné
et symlinké. Le cask `nightly` est installé (`a372ba5`), la version stable
accusant un retard important sur les correctifs de rendu.

## Conséquences

- Configuration en Lua, versionnée, cohérente avec celle de Neovim.
- Le canal `nightly` expose à des régressions : c'est le prix des correctifs de
  rendu, ajustés dans `61d847b`.
- Le thème est piloté depuis le dépôt, ce qui rend possible la bascule
  automatique clair/sombre ([ADR-014](014-theme-catppuccin-unique.md)).

## Alternatives écartées

- iTerm2 : configuration non versionnable.
- Alacritty : format de configuration instable, fonctionnalités de rendu
  manquantes.
- Terminal.app : insuffisant sur les couleurs et les polices.
