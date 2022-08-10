BREW_BIN:=$(shell if [ "$(shell uname -p)" = "arm" ]; then echo "/opt/homebrew/bin"; else echo "/usr/local/bin"; fi)
NPM_BIN:=/usr/local/bin

usage:
	@echo all - Setup dev env

all: \
	obsidian \
	wezterm \
	zsh

extra: \
	daisydisk \
	font-jetbrains-mono \
	mindnode \
	slack

obsidian: brew /Applications/Obsidian.app
/Applications/Obsidian.app:
	brew install obsidian

font-jetbrains-mono: ~/Library/Fonts/JetBrains\ Mono\ Regular\ Nerd\ Font\ Complete.ttf
~/Library/Fonts/JetBrains\ Mono\ Regular\ Nerd\ Font\ Complete.ttf:
	brew tap homebrew/cask-fonts
	brew install font-jetbrains-mono-nerd-font

wezterm: brew font-jetbrains-mono /Applications/WezTerm.app ~/.wezterm.lua
/Applications/WezTerm.app:
	brew install --cask wez/wezterm/wezterm
~/.wezterm.lua:
	ln -s ~/.dotfiles/.wezterm.lua $@

zsh: ~/.zshrc
~/.zshrc:
	ln -s ~/.dotfiles/zsh/zshrc $@
	chsh -s /bin/zsh


1password: /Applications/1password\ 7.app
/Applications/1password\ 7.app:
	brew install 1password

alfred: brew /Applications/Alfred\ 4.app
/Applications/Alfred\ 4.app:
	brew install alfred

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
	brew tap homebrew/cask-fonts

bars: node ${NPM_BIN}/bars
${NPM_BIN}/bars:
	@npm install -g https://github.com/jez/bars.git

yabai: ~/.yabairc ${BREW_BIN}/yabai
${BREW_BIN}/yabai:
	brew install koekeishiya/formulae/yabai
~/.yabairc:
	ln -s ~/.dotfiles/.yabairc $@

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

docker: brew /Applications/Docker.app
/Applications/Docker.app:
	brew install docker

emacs: brew ${BREW_BIN}/emacs ~/.emacs.d/custom.el ~/.emacs
${BREW_BIN}/emacs:
	brew install emacs
~/.emacs.d:
	mkdir $@
~/.emacs.d/custom.el: ~/.emacs.d
	touch $@
~/.emacs:
	ln -s ~/.dotfiles/.emacs ~/.emacs


fd: brew ${BREW_BIN}/fd 
${BREW_BIN}/fd:
	brew install fd

fzf: ~/.fzf
~/.fzf:
	git clone https://github.com/junegunn/fzf.git ~/.fzf
	~/.fzf/install --no-update-rc

git-heatmap: brew bars ${BREW_BIN}/git-heatmap
${BREW_BIN}/git-heatmap:
	brew install jez/formulae/git-heatmap

google-chrome: brew /Applications/Google\ Chrome.app
/Applications/Google\ Chrome.app:
	brew install google-chrome

gpg: brew ${BREW_BIN}/gpg ${BREW_BIN}/pinentry-mac
${BREW_BIN}/gpg:
	brew install gpg
# /usr/local/opt/gpg-agent/bin/gpg-agent:
#	brew install gpg-agent
${BREW_BIN}/pinentry-mac:
	brew install pinentry-mac

gh: brew ${BREW_BIN}/gh
${BREW_BIN}/gh:
	brew install github/gh/gh

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

mosh: brew ${BREW_BIN}/mosh/
${BREW_BIN}/mosh/:
	brew install mosh

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

prettier: node ${NPM_BIN}/prettier
${NPM_BIN}/prettier:
	npm i -g prettier

ripgrep: brew
	brew install ripgrep

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

tmux: brew ${BREW_BIN}/tmux ~/.tmux.conf
${BREW_BIN}/tmux:
	brew install reattach-to-user-namespace
	brew install tmux
~/.tmux.conf:
	ln -s ~/.dotfiles/tmux/.tmux.conf ~/.tmux.conf

trymodule: node ${BREW_BIN}/trymodule
${BREW_BIN}/trymodule:
	npm install -g trymodule
	
tldr: brew ${BREW_BIN}/tldr
${BREW_BIN}/tldr:
	brew install tldr

translate-shell: ${BREW_BIN}/trans
${BREW_BIN}/trans:
	brew install translate-shell

typescript: node ${BREW_BIN}/tsc
${BREW_BIN}/tsc:
	npm install -g typescript

vagrant: brew virtualbox ansible ${BREW_BIN}/vagrant
${BREW_BIN}/vagrant:
	brew install vagrant

nvim : node brew ${BREW_BIN}/nvim
${BREW_BIN}/nvim:
	# To use neovim 0.5
	# Install cmake luarocks
	# brew install --HEAD neovim
	brew install neovim
	npm i -g neovim
~/.config:
	mkdir $@
~/.config/nvim:
	ln -s ~/.dotfiles/nvim ~/.config/nvim

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

