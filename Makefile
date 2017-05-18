BREW_BIN:=/usr/local/bin
NPM_BIN:=/usr/local/bin

usage:
	@echo all - Setup dev env

all: \
	1password \
	daisydisk \
	docker \
	emacs \
	fantastical \
	font-firacode \
	fzf \
	google-chrome \
	gpg \
	hub \
	hyper \
	iterm2 \
	lftp \
	ncdu \
	node \
	nvm \
	prettier \
	prezto \
	ranger \
	slack \
	slate \
	spotify \
	shellcheck \
	tadam \
	the_silver_searcher \
	tmux \
	trymodule \
	vagrant \
	vim \
	xcode \
	yarn

1password: mas /Applications/1password.app
/Applications/1password.app:
	mas install 443987910

ansible: brew ${BREW_BIN}/ansible
${BREW_BIN}/ansible:
	brew install ansible

brew: ${BREW_BIN}/brew
${BREW_BIN}/brew:
	curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install > /tmp/brew-installer.sh
	chmod +x /tmp/brew-installer.sh
	/tmp/brew-installer.sh
	brew install caskroom/cask/brew-cask
	brew tap gapple/services
	brew tap caskroom/fonts
	brew tap homebrew/versions

daisydisk: mas /Applications/DaisyDisk.app
/Applications/DaisyDisk.app:
	mas install 411643860

docker: brew /Applications/Docker.app
/Applications/Docker.app:
	brew cask install docker-beta

emacs: brew ${BREW_BIN}/emacs ~/.emacs.d/custom.el ~/.emacs
${BREW_BIN}/emacs:
	brew install emacs
~/.emacs.d:
	mkdir $@
~/.emacs.d/custom.el: ~/.emacs.d
	touch $@
~/.emacs:
	ln -s ~/.dotfiles/.emacs ~/.emacs


fantastical: mas /Applications/Fantastical\ 2.app
/Applications/Fantastical\ 2.app:
	mas install 975937182

font-firacode: ~/Library/Fonts/FiraCode-bold.otf
~/Library/Fonts/FiraCode-bold.otf:
	brew cask install font-fira-code

fzf: ~/.fzf
~/.fzf:
	git clone https://github.com/junegunn/fzf.git ~/.fzf
	~/.fzf/install --no-update-rc

google-chrome: brew /Applications/Google\ Chrome.app
/Applications/Google\ Chrome.app:
	brew cask install google-chrome

gpg: brew /usr/local/opt/gpg-agent/bin/gpg-agent ${BREW_BIN}/gpg ${BREW_BIN}/pinentry-mac
${BREW_BIN}/gpg:
	brew install gpg
/usr/local/opt/gpg-agent/bin/gpg-agent:
	brew install gpg-agent
${BREW_BIN}/pinentry-mac:
	brew install pinentry-mac

hub: brew ${BREW_BIN}/hub
${BREW_BIN}/hub:
	brew install hub

hyper: brew /Applications/Hyper.app
/Applications/Hyper.app:
	brew cask install hyper

gpg-config:
	echo 'use-agent' > ~/.gnupg/gpg.conf
	echo 'use-standard-socket' > ~/.gnupg/gpg-agent.conf
	echo 'pinentry-program ${BREW_BIN}/pinentry-mac' >> ~/.gnupg/gpg-agent.conf

iterm2: brew /Applications/iTerm.app
/Applications/iTerm.app:
	brew install Caskroom/versions/iterm2-nightly

kwm: brew ${BREW_BIN}/kwmc ~/.kwm
${BREW_BIN}/kwmc:
	brew install koekeishiya/formulae/kwm
~/.kwm:
	ln -s ~/.dotfiles/kwm $@

khd: brew ${BREW_BIN}/khd
${BREW_BIN}/khd:
	brew install koekeishiya/formulae/khd

lftp: brew ${BREW_BIN}/lftp
${BREW_BIN}/lftp:
	brew install homebrew/boneyard/lftp

mas: brew ${BREW_BIN}/mas/
${BREW_BIN}/mas/:
	brew install mas

ncdu: brew ${BREW_BIN}/ncdu
${BREW_BIN}/ncdu:
	brew install ncdu

node: ${BREW_BIN}/node
${BREW_BIN}/node:
	brew install node

nvm: /usr/local/opt/nvm/nvm.sh ~/.nvm
/usr/local/opt/nvm/nvm.sh:
	brew install nvm
~/.nvm:
	mkdir $@

prettier: node ${NPM_BIN}/prettier
${NPM_BIN}/prettier:
	npm i -g prettier

prezto: ~/.zprezto ~/.zpreztorc ~/.zlogin ~/.zlogout ~/.zprofile ~/.zshenv ~/.zshrc \
	~/.zprezto/modules/prompt/functions/prompt_seb_mini_setup \
	~/.zprezto/modules/prompt/functions/prompt_seb_setup
~/.zprezto:
	git clone --recursive https://github.com/sorin-ionescu/prezto.git $@
~/.zlogin ~/.zlogout ~/.zprofile ~/.zshenv:
	ln -s ~/.zprezto/runcoms/$(subst .,,$(notdir $@)) $@
	chsh -s /bin/zsh
~/.zshrc:
	ln -s ~/.dotfiles/zsh/zshrc $@
~/.zpreztorc:
	ln -s ~/.dotfiles/zsh/zpreztorc $@
~/.zprezto/modules/prompt/functions/prompt_seb_mini_setup:
	ln -s ~/.dotfiles/zsh/prompts/prompt_seb_mini_setup $@
~/.zprezto/modules/prompt/functions/prompt_seb_setup:
	ln -s ~/.dotfiles/zsh/prompts/prompt_seb_setup $@

ranger: brew ~/.config/ranger ${BREW_BIN}/ranger
${BREW_BIN}/ranger:
	brew install ranger
~/.config/ranger: ~/.config
	ln -s ~/.dotfiles/ranger $@

slack: mas /Applications/Slack.app
/Applications/Slack.app:
	mas install 803453959

slate: brew /Applications/Slate.app ~/.slate.js
/Applications/Slate.app:
	brew cask install mattr-slate
~/.slate.js:
	ln -s ~/.dotfiles/slate/.slate.js ~/.slate.js

spotify: brew /Applications/Spotify.app
/Applications/Spotify.app:
	brew cask install spotify

shellcheck: brew ${BREW_BIN}/shellcheck
${BREW_BIN}/shellcheck: 
	brew install shellcheck

tadam: mas /Applications/Tadam.app
/Applications/Tadam.app:
	mas install 531349534

tmux: brew ${BREW_BIN}/tmux ~/.tmux.conf
${BREW_BIN}/tmux:
	brew install reattach-to-user-namespace
	brew install tmux
~/.tmux.conf:
	ln -s ~/.dotfiles/tmux/.tmux.conf ~/.tmux.conf

trymodule: node ${BREW_BIN}/trymodule
${BREW_BIN}/trymodule:
	npm install -g trymodule
	
the_silver_searcher: brew ${BREW_BIN}/ag
${BREW_BIN}/ag:
	brew install the_silver_searcher

vagrant: brew virtualbox ansible ${BREW_BIN}/vagrant
${BREW_BIN}/vagrant:
	brew cask install vagrant

vim: brew ${BREW_BIN}/vim ${BREW_BIN}/nvim ~/.vimrc prettier
${BREW_BIN}/vim:
	brew install vim

${BREW_BIN}/nvim:
	brew install neovim/neovim/neovim
~/.config:
	mkdir $@
~/.vimrc:
	ln -s ~/.dotfiles/vim ~/.vim
	ln -s ~/.dotfiles/vim/.vimrc ~/.vimrc
	ln -s ~/.vim ~/.config/nvim
	cd ~/.vim && make

virtualbox: brew ${BREW_BIN}/VBoxHeadless
${BREW_BIN}/VBoxHeadless:
	brew cask install virtualbox

xcode: mas /Applications/XCode.app
/Applications/XCode.app:
	mas install 497799835

yarn: node ${BREW_BIN}/yarn
${BREW_BIN}/yarn:
	npm install -g yarn



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
	brew cask install vlc

postman: brew ~/Applications/Postman.app
~/Applications/Postman.app:
	brew install Caskroom/cask/postman

packer: brew ${BREW_BIN}/packer
${BREW_BIN}/packer:
	brew cask install packer

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

mongohacker: node /usr/local/lib/node_modules/mongo-hacker
/usr/local/lib/node_modules/mongo-hacker:
	@npm install -g mongo-hacker

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

