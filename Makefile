BREW_BIN:=$(shell if [ "$(shell uname -p)" = "arm" ]; then echo "/opt/homebrew/bin"; else echo "/usr/local/bin"; fi)
BREW_GNU_BIN:=$(shell if [ "$(shell uname -p)" = "arm" ]; then echo "/opt/homebrew/opt"; else echo "/usr/local/opt"; fi)
NPM_BIN:=$(HOME)/.volta/bin
APP_BIN:=/Applications
# DOTFILES_PATH should be ~/.dotfiles when installed normally
DOTFILES_PATH:=$(shell pwd)
# SKIP_PAID_APPS: set to 1 to skip paid Mac App Store apps (useful for CI)
SKIP_PAID_APPS?=0

.PHONY: usage all extra terminal work personal utils clean brew node volta javascript mas

usage:
	@echo all - Setup dev env

utils: \
	cleanshot \
	opensuperwhisper \
	rectangle-pro \
	superwhisper \
	things-3
all: \
	extra \
	terminal \
	work \
	utils \
	personal

extra: \
	daisydisk \
	font-jetbrains-mono

################################################################################
# Terminal section
################################################################################
terminal: \
	~/.config \
	bat \
	bottom \
	broot \
	procs \
	eza \
	fd \
	fish \
	fzf \
	git-delta \
	gnu-sed \
	htop \
	jq \
	jscpd \
	lazygit \
	mtr \
	nvim \
	ripgrep \
	tmux \
	tokei \
	wezterm \
	zoxide \
	zsh

~/.config:
	mkdir -p $@

bat: brew ${BREW_BIN}/bat ~/.config/bat/themes/Catppuccin\ Latte.tmTheme
${BREW_BIN}/bat:
	brew install bat
~/.config/bat/themes:
	mkdir -p $@
~/.config/bat/themes/Catppuccin\ Latte.tmTheme: | ~/.config/bat/themes
	curl -L -o ~/.config/bat/themes/Catppuccin\ Latte.tmTheme https://github.com/catppuccin/bat/raw/main/themes/Catppuccin%20Latte.tmTheme
	curl -L -o ~/.config/bat/themes/Catppuccin\ Mocha.tmTheme https://github.com/catppuccin/bat/raw/main/themes/Catppuccin%20Mocha.tmTheme
	bat cache --build

bottom: brew ${BREW_BIN}/btm
${BREW_BIN}/btm:
	brew install bottom

broot: brew ${BREW_BIN}/broot
${BREW_BIN}/broot:
	brew install broot

procs: brew ${BREW_BIN}/procs
${BREW_BIN}/procs:
	brew install procs

eza: brew ${BREW_BIN}/eza
${BREW_BIN}/eza:
	brew install eza

fd: brew ${BREW_BIN}/fd
${BREW_BIN}/fd:
	brew install fd

fish: brew starship ~/.config/fish ${BREW_BIN}/fish
${BREW_BIN}/fish:
	brew install fish fisher
	fish -c 'fisher install PatrickF1/fzf.fish'
	@echo 'If you want to switch your shell to fish, please run the following command'
	@echo '$> sudo chpass -s ${BREW_BIN}/fish ${USER}'

~/.config/fish: ${DOTFILES_PATH}/fish | ~/.config
	ln -s ${DOTFILES_PATH}/fish $@

gnu-sed: brew ${BREW_GNU_BIN}/gnu-sed
${BREW_GNU_BIN}/gnu-sed:
	brew install gnu-sed

htop: brew ${BREW_BIN}/htop
${BREW_BIN}/htop:
	brew install htop

lazygit: brew ${BREW_BIN}/lazygit
${BREW_BIN}/lazygit:
	brew install lazygit

tokei: brew ${BREW_BIN}/tokei
${BREW_BIN}/tokei:
	brew install tokei

wezterm: brew font-jetbrains-mono font-iosevka-nerd-font /Applications/WezTerm.app ~/.wezterm.lua
/Applications/WezTerm.app:
	brew install --cask wez/wezterm/wezterm
~/.wezterm.lua: ${DOTFILES_PATH}/.wezterm.lua
	ln -s ${DOTFILES_PATH}/.wezterm.lua $@

################################################################################
# End of the terminal section
################################################################################

################################################################################
# Work section
################################################################################
work: \
	arc \
	aws \
	claude-code \
	codex \
	cursor \
	docker \
	doppler \
	gh \
	javascript \
	k9s \
	lazydocker \
	mosh \
	postgresql \
	renovate \
	tableplus \
	terraform \
	1password

arc: brew ${APP_BIN}/Arc.app
${APP_BIN}/Arc.app:
	brew install --cask arc

aws: brew ${BREW_BIN}/aws
${BREW_BIN}/aws:
	brew install awscli

docker: brew lazydocker /Applications/Orbstack.app
/Applications/Orbstack.app:
	brew install orbstack

doppler: gnupg ${BREW_BIN}/doppler
${BREW_BIN}/doppler:
	brew install dopplerhq/cli/doppler

gnupg: brew ${BREW_BIN}/gpg
${BREW_BIN}/gpg:
	brew install gnupg

gh: brew ${BREW_BIN}/gh
${BREW_BIN}/gh:
	brew install gh

k9s: brew ${BREW_BIN}/k9s
${BREW_BIN}/k9s:
	brew install k9s

lazydocker: brew ${BREW_BIN}/lazydocker
${BREW_BIN}/lazydocker:
	brew install jesseduffield/lazydocker/lazydocker

mosh: brew ${BREW_BIN}/mosh
${BREW_BIN}/mosh:
	brew install mosh

postgresql: brew ${BREW_GNU_BIN}/postgresql@16/bin/psql ~/.psqlrc
${BREW_GNU_BIN}/postgresql@16/bin/psql:
	brew install postgresql@16
~/.psqlrc: ${DOTFILES_PATH}/.psqlrc
	ln -s ${DOTFILES_PATH}/.psqlrc $@

renovate: brew node ${NPM_BIN}/renovate
${NPM_BIN}/renovate: ${NPM_BIN}/pnpm
	${NPM_BIN}/pnpm add -g renovate

tableplus: brew ${APP_BIN}/TablePlus.app
${APP_BIN}/TablePlus.app:
	brew install tableplus

terraform: brew ${BREW_BIN}/terraform
${BREW_BIN}/terraform:
	brew tap hashicorp/tap
	brew install hashicorp/tap/terraform

1password: brew ${APP_BIN}/1Password.app
${APP_BIN}/1Password.app:
	brew install --cask 1password

cursor: brew ${APP_BIN}/Cursor.app ~/.local/bin/cursor-agent ~/.config/Cursor/User/settings.json ~/.config/Cursor/User/extensions.json ~/.config/Cursor/User/keybindings.json
${APP_BIN}/Cursor.app:
	brew install --cask cursor
~/.local/bin/cursor-agent:
	curl https://cursor.com/install -fsS | bash
~/.config/Cursor/User: | ~/.config
	mkdir -p $@
~/.config/Cursor/User/settings.json: ${DOTFILES_PATH}/cursor/settings.json | ~/.config/Cursor/User
	ln -s ${DOTFILES_PATH}/cursor/settings.json $@
~/.config/Cursor/User/extensions.json: ${DOTFILES_PATH}/cursor/extensions.json | ~/.config/Cursor/User
	ln -s ${DOTFILES_PATH}/cursor/extensions.json $@
~/.config/Cursor/User/keybindings.json: ${DOTFILES_PATH}/cursor/keybindings.json | ~/.config/Cursor/User
	ln -s ${DOTFILES_PATH}/cursor/keybindings.json $@

claude-code: brew ${BREW_BIN}/claude ~/.claude/CLAUDE.md
${BREW_BIN}/claude:
	brew install --cask claude-code
~/.claude:
	mkdir -p $@
~/.claude/CLAUDE.md: ${DOTFILES_PATH}/ai/AGENTS.md | ~/.claude
	ln -s ${DOTFILES_PATH}/ai/AGENTS.md $@

codex: node $(HOME)/Library/pnpm/codex ~/.codex/AGENTS.md
$(HOME)/Library/pnpm/codex: ${NPM_BIN}/pnpm
	${NPM_BIN}/pnpm add -g @openai/codex
~/.codex:
	mkdir -p $@
~/.codex/AGENTS.md: ${DOTFILES_PATH}/ai/AGENTS.md | ~/.codex
	ln -s ${DOTFILES_PATH}/ai/AGENTS.md $@

################################################################################
# End of work section
################################################################################

################################################################################
# Personal section
################################################################################

personal: \
	calibre \
	chatgpt \
	claude \
	flow \
	language-tool \
	whatsapp

# Local vault on 1password does not work with 1password
# app from the app store. We need to manually download
# 1password from the website
# 1password: /Applications/1password\ 7.app
# /Applications/1password\ 7.app:
#	brew install 1password

calibre: brew ${APP_BIN}/Calibre.app
${APP_BIN}/Calibre.app:
	brew install calibre

chatgpt: brew ${APP_BIN}/ChatGPT.app
${APP_BIN}/ChatGPT.app:
	brew install --cask chatgpt

claude: brew ${APP_BIN}/Claude.app
${APP_BIN}/Claude.app:
	brew install --cask claude

flow: mas /Applications/Flow.app
/Applications/Flow.app:
	@if [ "$(SKIP_PAID_APPS)" = "1" ]; then \
		echo "Skipping Flow installation (SKIP_PAID_APPS=1)"; \
		mkdir -p "$@"; \
	else \
		echo "Installing Flow"; \
		mas install 1423210932 || echo "Warning: Failed to install Flow (may not be purchased on this Apple account)"; \
	fi

language-tool: brew ${APP_BIN}/LanguageTool.app
${APP_BIN}/LanguageTool.app:
	brew install --cask languagetool

whatsapp: brew ${APP_BIN}/WhatsApp.app
${APP_BIN}/WhatsApp.app:
	brew install --cask whatsapp

################################################################################
# End of personal section
################################################################################

################################################################################
# Utils section
################################################################################

cleanshot: brew ${APP_BIN}/CleanShot\ X.app
${APP_BIN}/CleanShot\ X.app:
	brew install cleanshot

rectangle-pro: brew /Applications/Rectangle\ Pro.app
/Applications/Rectangle\ Pro.app:
	brew install --cask rectangle-pro

opensuperwhisper: brew ${APP_BIN}/OpenSuperWhisper.app
${APP_BIN}/OpenSuperWhisper.app:
	brew install --cask opensuperwhisper

superwhisper: brew ${APP_BIN}/SuperWhisper.app
${APP_BIN}/SuperWhisper.app:
	brew install --cask superwhisper

things-3: mas /Applications/Things3.app
/Applications/Things3.app:
	@if [ "$(SKIP_PAID_APPS)" = "1" ]; then \
		echo "Skipping Things 3 installation (SKIP_PAID_APPS=1)"; \
		mkdir -p "$@"; \
	else \
		echo "Installing Things 3"; \
		mas install 904280696 || echo "Warning: Failed to install Things 3 (may not be purchased on this Apple account)"; \
	fi

################################################################################
# End of utils section
################################################################################

javascript: prettier cspell
prettier: node ${NPM_BIN}/prettier
${NPM_BIN}/prettier: ${NPM_BIN}/pnpm
	${NPM_BIN}/pnpm add -g prettier @fsouza/prettierd
cspell: node ${NPM_BIN}/cspell
${NPM_BIN}/cspell: ${NPM_BIN}/pnpm
	${NPM_BIN}/pnpm add -g cspell

nvim: ripgrep node brew ${BREW_BIN}/nvim ~/.config/nvim ~/cspell.json
${BREW_BIN}/nvim: ${NPM_BIN}/pnpm
	brew install neovim
	${NPM_BIN}/pnpm add -g neovim
~/.config/nvim: ${DOTFILES_PATH}/nvim | ~/.config
	ln -s ${DOTFILES_PATH}/nvim ~/.config/nvim
~/cspell.json: ${DOTFILES_PATH}/cspell.json
	ln -s ${DOTFILES_PATH}/cspell.json $@

font-jetbrains-mono: ~/Library/Fonts/JetBrainsMonoNLNerdFont-Regular.ttf
~/Library/Fonts/JetBrainsMonoNLNerdFont-Regular.ttf:
	brew install font-jetbrains-mono-nerd-font

font-iosevka-nerd-font: ~/Library/Fonts/IosevkaNerdFont-Regular.ttf
~/Library/Fonts/IosevkaNerdFont-Regular.ttf:
	brew install font-iosevka-nerd-font

fzf: ~/.fzf
~/.fzf:
	git clone https://github.com/junegunn/fzf.git ~/.fzf
	~/.fzf/install --no-update-rc

ripgrep: brew ${BREW_BIN}/rg
${BREW_BIN}/rg:
	brew install ripgrep

zoxide: brew ${BREW_BIN}/zoxide
${BREW_BIN}/zoxide:
	brew install zoxide

zsh: ~/.zshrc
~/.zshrc: ${DOTFILES_PATH}/.zshrc
	ln -s ${DOTFILES_PATH}/.zshrc $@

git-delta: brew ${BREW_BIN}/delta ~/.gitconfig.delta
${BREW_BIN}/delta:
	brew install git-delta
~/.gitconfig.delta: ${DOTFILES_PATH}/.gitconfig.delta
	ln -s ${DOTFILES_PATH}/.gitconfig.delta ~/.gitconfig.delta
	@if ! git config --global --get include.path | grep -q "\.gitconfig\.delta"; then \
		git config --global include.path "~/.gitconfig.delta"; \
		echo "Added include.path to ~/.gitconfig"; \
	fi

starship: brew ${BREW_BIN}/starship ~/.config/starship.toml
${BREW_BIN}/starship:
	brew install starship
~/.config/starship.toml: | ~/.config
	@if [ -f ${DOTFILES_PATH}/.config/starship.toml ]; then \
		ln -sf ${DOTFILES_PATH}/.config/starship.toml $@; \
		echo "Created starship.toml symlink"; \
	else \
		echo "Skipping starship.toml symlink: source file ${DOTFILES_PATH}/.config/starship.toml not found"; \
		touch $@; \
	fi

tmux: brew ${BREW_BIN}/tmux ~/.tmux.conf
${BREW_BIN}/tmux:
	brew install tmux
~/.tmux.conf: ${DOTFILES_PATH}/tmux/.tmux.conf
	ln -s ${DOTFILES_PATH}/tmux/.tmux.conf ~/.tmux.conf

brew: ${BREW_BIN}/brew
${BREW_BIN}/brew:
	curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh > /tmp/brew-installer.sh
	chmod +x /tmp/brew-installer.sh
	/tmp/brew-installer.sh
	brew tap gapple/services

daisydisk: mas /Applications/DaisyDisk.app
/Applications/DaisyDisk.app:
	@if [ "$(SKIP_PAID_APPS)" = "1" ]; then \
		echo "Skipping DaisyDisk installation (SKIP_PAID_APPS=1)"; \
		mkdir -p "$@"; \
	else \
		echo "Installing DaisyDisk"; \
		mas install 411643860 || echo "Warning: Failed to install DaisyDisk (may not be purchased on this Apple account)"; \
	fi

${BREW_BIN}/pinentry-mac:
	brew install pinentry-mac

jscpd: node ${NPM_BIN}/jscpd
${NPM_BIN}/jscpd: ${NPM_BIN}/pnpm
	@${NPM_BIN}/pnpm add -g jscpd

mas: brew ${BREW_BIN}/mas
${BREW_BIN}/mas:
	brew install mas

node: volta ${NPM_BIN}/node
${NPM_BIN}/node:
	${BREW_BIN}/volta install node@lts
${NPM_BIN}/pnpm: ${NPM_BIN}/node
	${BREW_BIN}/volta install pnpm

volta: brew ${BREW_BIN}/volta
${BREW_BIN}/volta:
	brew install volta

mtr: brew ${BREW_BIN}/../sbin/mtr
${BREW_BIN}/../sbin/mtr:
	brew install mtr

jq: brew ${BREW_BIN}/jq
${BREW_BIN}/jq:
	brew install jq

clean:
	rm -rf ~/.config/nvim
	rm -rf ~/.local/share/nvim
	rm -rf ~/.cache/nvim
	rm -rf ~/.config/Cursor/User/settings.json
	rm -rf ~/.config/Cursor/User/extensions.json
	rm -rf ~/.config/Cursor/User/keybindings.json
