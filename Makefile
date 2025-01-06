BREW_BIN:=$(shell if [ "$(shell uname -p)" = "arm" ]; then echo "/opt/homebrew/bin"; else echo "/usr/local/bin"; fi)
BREW_GNU_BIN:=/opt/homebrew/opt/
NPM_BIN:=~/.volta/bin
APP_BIN:=/Applications

usage:
	@echo all - Setup dev env

utils: \
	alfred \
	bartender \
	cleanshot
all: \
	terminal \
	work \
	utils \
	personal

extra: \
	daisydisk \
	font-jetbrains-mono \
	mindnode \
	slack

################################################################################
# Terminal section
################################################################################
terminal: \
	~/.config \
	bottom \
	broot \
	eza \
	fd \
	fish \
	gnu-sed \
	htop \
	lazygit \
	nvim \
	opencommit \
	tldr \
	tmux \
	tokei \
	wezterm \
	zsh

~/.config:
	mkdir $@

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

fish: brew ${BREW_BIN}/fish ~/.config/fish
${BREW_BIN}/fish:
	brew install fish

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

opencommit: brew node ${BREW_BIN}/opencommit
${BREW_BIN}/opencommit:
	npm i -g opencommit

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
	docker \
	doppler \
	gh \
	javascript \
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
	gh extension install github/gh-copilot

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

################################################################################
# End of work section
################################################################################

################################################################################
# Personal section
################################################################################

personal: \
	calibre \
	doctl \
	obsidian \
	rust \
	spotify

# Local vault on 1password does not work with 1password
# app from the app store. We need to manually download
# 1password from the website
# 1password: /Applications/1password\ 7.app
# /Applications/1password\ 7.app:
#	brew install 1password

calibre: brew ${APP_BIN}/Calibre.app
${APP_BIN}/Calibre.app:
	brew install calibre

doctl: brew ${BREW_BIN}/doctl
${BREW_BIN}/doctl:
	brew install doctl

obsidian: brew /Applications/Obsidian.app
/Applications/Obsidian.app:
	brew install obsidian

todoist: brew ${APP_BIN}/Todoist.app
${APP_BIN}/Todoist.app:
	brew install todoist

rust: brew ~/.cargo
~/.cargo:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Faster linking for rust build
# This require xcode
zld:
	brew install michaeleisel/zld/zld

spotify: brew ${APP_BIN}/Spotify.app
${APP_BIN}/Spotify.app:
	brew install spotify

################################################################################
# End of personal section
################################################################################

################################################################################
# Utils section
################################################################################

alfred: brew /Applications/Alfred\ 5.app
/Applications/Alfred\ 5.app:
	brew install alfred

bartender: brew ${APP_BIN}/Bartender\ 4.app
${APP_BIN}/Bartender\ 4.app:
	brew install bartender

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

tldr: brew ${BREW_BIN}/tldr
${BREW_BIN}/tldr:
	brew install tldr

font-fira-code: ~/Library/Fonts/Fira\ Code\ Retina\ Nerd\ Font\ Complete.otf
~/Library/Fonts/Fira\ Code\ Retina\ Nerd\ Font\ Complete.otf:
	brew install font-fira-code-nerd-font
font-iosevka: ~/Library/Fonts/Iosevka\ Thin\ Nerd\ Font\ Complete.ttf
~/Library/Fonts/Iosevka\ Thin\ Nerd\ Font\ Complete.ttf:
	brew install font-iosevka-nerd-font
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

zsh: starship fzf ~/.zshrc ~/.zsh/zsh-autosuggestions ~/.zsh/zsh-syntax-highlighting ~/.zsh/zsh-completions
~/.zshrc:
	ln -s ~/.dotfiles/.zshrc $@
	@echo 'If you want to switch your shell to zsh, please run the following command'
	@echo '$> chsh -s /bin/zsh'
~/.zsh/zsh-autosuggestions:
	git clone https://github.com/zsh-users/zsh-autosuggestions $@
~/.zsh/zsh-syntax-highlighting:
	git clone https://github.com/zsh-users/zsh-syntax-highlighting.git $@
~/.zsh/zsh-completions:
	git clone https://github.com/zsh-users/zsh-completions $@

bash-language-server: node ${NPM_BIN}/bash-language-server
${NPM_BIN}/bash-language-server:
	@npm -g install bash-language-server

bat: brew ${BREW_BIN}/bat
${BREW_BIN}/bat:
	brew install bat

comby: brew ${BREW_BIN}/comby
${BREW_BIN}/comby:
	brew install comby

ansible: brew ${BREW_BIN}/molecule ${BREW_BIN}/ansible
${BREW_BIN}/ansible:
	brew install ansible

${BREW_BIN}/molecule:
	brew install molecule

brew: ${BREW_BIN}/brew
${BREW_BIN}/brew:
	curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh > /tmp/brew-installer.sh
	chmod +x /tmp/brew-installer.sh
	/tmp/brew-installer.sh
	brew tap gapple/services

bars: node ${NPM_BIN}/bars
${NPM_BIN}/bars:
	@npm install -g https://github.com/jez/bars.git

ctop: ${BREW_BIN}/ctop
${BREW_BIN}/ctop:
	brew install ctop

daisydisk: mas /Applications/DaisyDisk.app
/Applications/DaisyDisk.app:
	echo "Install DaisyDisk"
	mas install 411643860

dash: /Applications/Dash.app
/Applications/Dash.app:
	brew install dash

diff-so-fancy: brew ${BREW_BIN}/diff-so-fancy 
${BREW_BIN}/diff-so-fancy:
	brew install diff-so-fancy

emacs: brew ${BREW_BIN}/emacs ~/.emacs.d/custom.el ~/.emacs
${BREW_BIN}/emacs:
	brew install emacs
~/.emacs.d:
	mkdir $@
~/.emacs.d/custom.el: ~/.emacs.d
	touch $@
~/.emacs:
	ln -s ~/.dotfiles/.emacs ~/.emacs

git-heatmap: brew bars ${BREW_BIN}/git-heatmap
${BREW_BIN}/git-heatmap:
	brew install jez/formulae/git-heatmap

google-chrome: brew /Applications/Google\ Chrome.app
/Applications/Google\ Chrome.app:
	brew install google-chrome

# /usr/local/opt/gpg-agent/bin/gpg-agent:
#	brew install gpg-agent
${BREW_BIN}/pinentry-mac:
	brew install pinentry-mac

gpg-config:
	echo 'use-agent' > ~/.gnupg/gpg.conf
	echo 'use-standard-socket' > ~/.gnupg/gpg-agent.conf
	echo 'pinentry-program ${BREW_BIN}/pinentry-mac' >> ~/.gnupg/gpg-agent.conf

graphql-language-service-cli: node ${NPM_BIN}/graphql-language-service-cli
${NPM_BIN}/graphql-language-service-cli:
	@npm -g install graphql-language-service-cli

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

node: ${BREW_BIN}/volta
${BREW_BIN}/node:
	volta install latest

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

mindnode: mas /Applications/MindNode.app
/Applications/MindNode.app:
	mas install 992076693

prettyping: brew ${BREW_BIN}/prettyping 
${BREW_BIN}/prettyping:
	brew install prettyping

slack: mas /Applications/Slack.app
/Applications/Slack.app:
	echo "Install slack"
	mas install 803453959

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

vagrant: brew virtualbox ansible ${BREW_BIN}/vagrant
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

cloc: brew
	brew install cloc

jq: brew ${BREW_BIN}/jq
${BREW_BIN}/jq:
	brew install jq

highlight: brew ${BREW_BIN}/highlight
${BREW_BIN}/highlight:
	brew install highlight

tig: brew ${BREW_BIN}/tig
${BREW_BIN}/tig:
	brew install tig

cli-tools: how2

how2: node ${BREW_BIN}/how2
${BREW_BIN}/how2:
	@npm -g install how2

pgcli: brew ${BREW_BIN}/pgcli
${BREW_BIN}/pgcli:
	brew install pgcli

clif: ${BREW_BIN}/clif
${BREW_BIN}/clif:
	@npm install -g clif

jsonlint: node ${BREW_BIN}/jsonlint
${BREW_BIN}/jsonlint:
	@npm install -g jsonlint

js-yaml: ${BREW_BIN}/js-yaml
${BREW_BIN}/js-yaml:
	@npm install -g js-yaml

jsinspect: node ${BREW_BIN}/jsinspect
${BREW_BIN}/jsinspect:
	@npm install -g jsinspect

retire: node ${BREW_BIN}/retire
${BREW_BIN}/retire:
	@npm install -g retire

david: node ${BREW_BIN}/david
${BREW_BIN}/david:
	@npm install -g david

nsp: node ${BREW_BIN}/nsp
${BREW_BIN}/nsp:
	@npm install -g nsp

recess:
	@npm install -g recess

sqlint: ${BREW_BIN}/sqlint
${BREW_BIN}/sqlint:
	@gem install sqlint

wallpaper:
	# Set wallpaper
	# osascript -e "tell application \"Finder\" to set desktop picture \
	#	to POSIX file \"/Library/Desktop Pictures/Color Burst 2.jpg\""
	osascript -e "tell application \"System Events\" to set picture of every desktop to \"/Library/Desktop Pictures/Color Burst 2.jpg\""

xz: brew
${BREW_BIN}/xz:
	brew install xz


clean: 
	rm -rf ~/.config/nvim
	rm -rf ~/.local/share/nvim
	rm -rf ~/.cache/nvim
