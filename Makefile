BREW_BIN:=$(shell if [ "$(shell uname -p)" = "arm" ]; then echo "/opt/homebrew/bin"; else echo "/usr/local/bin"; fi)
BREW_GNU_BIN:=/opt/homebrew/opt/
NPM_BIN:=~/.volta/bin
APP_BIN:=/Applications

usage:
	@echo all - Setup dev env

utils: \
	cleanshot
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
	eza \
	fd \
	fish \
	fzf \
	gnu-sed \
	htop \
	jq \
	lazygit \
	nvim \
	tmux \
	tokei \
	wezterm

~/.config:
	mkdir $@

bat: brew ${BREW_BIN}/bat
${BREW_BIN}/bat:
	brew install bat

bottom: brew ${BREW_BIN}/btm
${BREW_BIN}/btm:
	brew install bottom

broot: brew ${BREW_BIN}/broot
${BREW_BIN}/broot:
	brew install broot

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

~/.config/fish:
	ln -s ~/.dotfiles/fish $@

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

wezterm: brew font-jetbrains-mono /Applications/WezTerm.app ~/.wezterm.lua
/Applications/WezTerm.app:
	brew install --cask wez/wezterm/wezterm
~/.wezterm.lua:
	ln -s ~/.dotfiles/.wezterm.lua $@

################################################################################
# End of the terminal section
################################################################################

################################################################################
# Work section
################################################################################
work: \
	aws \
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
	terraform

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

mosh: brew ${BREW_BIN}/mosh/
${BREW_BIN}/mosh/:
	brew install mosh

postgresql: brew ${BREW_BIN}/psql ~/.psqlrc
${BREW_BIN}/psql:
	brew install postgresql
~/.psqlrc:
	ln -s ~/.dotfiles/.psqlrc $@

renovate: brew node ${NPM_BIN}/renovate
${NPM_BIN}/renovate:
	npm i -g renovate

tableplus: brew ${APP_BIN}/TablePlus.app
${APP_BIN}/TablePlus.app:
	brew install tableplus

terraform: brew ${BREW_BIN}/terraform
${BREW_BIN}/terraform:
	brew tap hashicorp/tap
	brew install hashicorp/tap/terraform

cursor: brew ${APP_BIN}/Cursor.app ~/.local/bin/cursor-agent ~/.config/Cursor/User/settings.json ~/.config/Cursor/User/extensions.json ~/.config/Cursor/User/keybindings.json
${APP_BIN}/Cursor.app:
	brew install --cask cursor
~/.local/bin/cursor-agent:
	curl https://cursor.com/install -fsS | bash
~/.config/Cursor/User/settings.json:
	mkdir -p ~/.config/Cursor/User
	ln -s ~/.dotfiles/cursor/settings.json $@
~/.config/Cursor/User/extensions.json:
	mkdir -p ~/.config/Cursor/User
	ln -s ~/.dotfiles/cursor/extensions.json $@
~/.config/Cursor/User/keybindings.json:
	mkdir -p ~/.config/Cursor/User
	ln -s ~/.dotfiles/cursor/keybindings.json $@

################################################################################
# End of work section
################################################################################

################################################################################
# Personal section
################################################################################

personal: \
	calibre

# Local vault on 1password does not work with 1password
# app from the app store. We need to manually download
# 1password from the website
# 1password: /Applications/1password\ 7.app
# /Applications/1password\ 7.app:
#	brew install 1password

calibre: brew ${APP_BIN}/Calibre.app
${APP_BIN}/Calibre.app:
	brew install calibre

################################################################################
# End of personal section
################################################################################

################################################################################
# Utils section
################################################################################

cleanshot: brew ${APP_BIN}/CleanShot\ X.app
${APP_BIN}/CleanShot\ X.app:
	brew install cleanshot

################################################################################
# End of utils section
################################################################################

javascript: prettier
prettier: node ${NPM_BIN}/prettier
${NPM_BIN}/prettier:
	npm i -g prettier @fsouza/prettierd

nvim : ripgrep node brew ${BREW_BIN}/nvim ~/.config/nvim ~/cspell.json
${BREW_BIN}/nvim:
	brew install neovim
	npm i -g neovim
~/.config/nvim:
	ln -s ~/.dotfiles/nvim ~/.config/nvim
~/cspell.json:
	ln -s ~/.dotfiles/cspell.json $@

font-jetbrains-mono: ~/Library/Fonts/JetBrains\ Mono\ Regular\ Nerd\ Font\ Complete.ttf
~/Library/Fonts/JetBrains\ Mono\ Regular\ Nerd\ Font\ Complete.ttf:
	brew install font-jetbrains-mono-nerd-font

fzf: ~/.fzf
~/.fzf:
	git clone https://github.com/junegunn/fzf.git ~/.fzf
	~/.fzf/install --no-update-rc

ripgrep: brew ${BREW_BIN}/rg
${BREW_BIN}/rg:
	brew install ripgrep

starship: brew ${BREW_BIN}/starship ~/.config/starship.toml
${BREW_BIN}/starship:
	brew install starship
~/.config/starship.toml:
	ln -s ~/.dotfiles/.config/starship.toml $@

tmux: brew ${BREW_BIN}/tmux ~/.tmux.conf
${BREW_BIN}/tmux:
	brew install tmux
~/.tmux.conf:
	ln -s ~/.dotfiles/tmux/.tmux.conf ~/.tmux.conf

yabai: ~/.yabairc ${BREW_BIN}/yabai
${BREW_BIN}/yabai:
	brew install koekeishiya/formulae/yabai
~/.yabairc:
	ln -s ~/.dotfiles/.yabairc $@

brew: ${BREW_BIN}/brew
${BREW_BIN}/brew:
	curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh > /tmp/brew-installer.sh
	chmod +x /tmp/brew-installer.sh
	/tmp/brew-installer.sh
	brew tap gapple/services

daisydisk: mas /Applications/DaisyDisk.app
/Applications/DaisyDisk.app:
	echo "Install DaisyDisk"
	mas install 411643860

# /usr/local/opt/gpg-agent/bin/gpg-agent:
#	brew install gpg-agent
${BREW_BIN}/pinentry-mac:
	brew install pinentry-mac

jscpd: node ${BREW_BIN}/jscpd
${BREW_BIN}/jscpd:
	@npm install -g jscpd

kap: brew /Applications/Kap.app
/Applications/Kap.app:
	brew install kap

libjpeg: brew ${BREW_BIN}/libjpeg
${BREW_BIN}/libjpeg:
	brew install libjpeg

lftp: brew ${BREW_BIN}/lftp
${BREW_BIN}/lftp:
	brew install lftp

mas: brew ${BREW_BIN}/mas/
${BREW_BIN}/mas/:
	brew install mas

ncdu: brew ${BREW_BIN}/ncdu
${BREW_BIN}/ncdu:
	brew install ncdu

node: volta ${NPM_BIN}/node ${NPM_BIN}/pnpm
${NPM_BIN}/node:
	${BREW_BIN}/volta install node@lts
${NPM_BIN}/pnpm:
	${BREW_BIN}/volta install pnpm

notion: brew /Applications/Notion.app
/Applications/Notion.app:
	brew cask install notion

numbers: mas /Applications/Numbers.app
/Applications/Numbers.app:
	mas install 409203825

nvm: /usr/local/opt/nvm/nvm.sh ~/.nvm
/usr/local/opt/nvm/nvm.sh:
	brew install nvm
~/.nvm:
	mkdir $@

prettyping: brew ${BREW_BIN}/prettyping 
${BREW_BIN}/prettyping:
	brew install prettyping

shellcheck: brew ${BREW_BIN}/shellcheck
${BREW_BIN}/shellcheck: 
	brew install shellcheck

tmate: brew ${BREW_BIN}/tmate
${BREW_BIN}/tmate:
	brew install tmate

trymodule: node ${BREW_BIN}/trymodule
${BREW_BIN}/trymodule:
	npm install -g trymodule
	
translate-shell: ${BREW_BIN}/trans
${BREW_BIN}/trans:
	brew install translate-shell

typescript: node ${BREW_BIN}/tsc
${BREW_BIN}/tsc:
	npm install -g typescript

vagrant: brew virtualbox ${BREW_BIN}/vagrant
${BREW_BIN}/vagrant:
	brew install vagrant

virtualbox: brew ${BREW_BIN}/VBoxHeadless
${BREW_BIN}/VBoxHeadless:
	brew install virtualbox

volta: brew ${BREW_BIN}/volta
${BREW_BIN}/volta:
	brew install volta


yarn: node ${BREW_BIN}/yarn
${BREW_BIN}/yarn:
	npm install -g yarn


visidata: brew ${BREW_BIN}/vd
${BREW_BIN}/vd:
	brew install saulpw/vd/visidata

youtube-dl: brew ${BREW_BIN}/youtube-dl
${BREW_BIN}/youtube-dl:
	brew install youtube-dl

watch: brew ${BREW_BIN}/watch
${BREW_BIN}/watch:
	brew install watch

mtr: brew ${BREW_BIN}/mtr
${BREW_BIN}/mtr:
	brew install mtr

travis:
	gem install travis

vlc: brew ~/Applications/VLC.app
~/Applications/VLC.app:
	brew install vlc

postman: brew ~/Applications/Postman.app
~/Applications/Postman.app:
	brew install postman

packer: brew ${BREW_BIN}/packer
${BREW_BIN}/packer:
	brew install packer

jq: brew ${BREW_BIN}/jq
${BREW_BIN}/jq:
	brew install jq

highlight: brew ${BREW_BIN}/highlight
${BREW_BIN}/highlight:
	brew install highlight

tig: brew ${BREW_BIN}/tig
${BREW_BIN}/tig:
	brew install tig

how2: node ${BREW_BIN}/how2
${BREW_BIN}/how2:
	@npm -g install how2

pgcli: brew ${BREW_BIN}/pgcli
${BREW_BIN}/pgcli:
	brew install pgcli

jsonlint: node ${BREW_BIN}/jsonlint
${BREW_BIN}/jsonlint:
	@npm install -g jsonlint

js-yaml: node ${BREW_BIN}/js-yaml
${BREW_BIN}/js-yaml:
	@npm install -g js-yaml

jsinspect: node ${BREW_BIN}/jsinspect
${BREW_BIN}/jsinspect:
	@npm install -g jsinspect

retire: node ${BREW_BIN}/retire
${BREW_BIN}/retire:
	@npm install -g retire

nsp: node ${BREW_BIN}/nsp
${BREW_BIN}/nsp:
	@npm install -g nsp

recess: node
	@npm install -g recess

sqlint: ${BREW_BIN}/sqlint
${BREW_BIN}/sqlint:
	@gem install sqlint

xz: brew
${BREW_BIN}/xz:
	brew install xz


clean: 
	rm -rf ~/.config/nvim
	rm -rf ~/.local/share/nvim
	rm -rf ~/.cache/nvim
	rm -rf ~/.config/Cursor/User/settings.json
	rm -rf ~/.config/Cursor/User/extensions.json
	rm -rf ~/.config/Cursor/User/keybindings.json
