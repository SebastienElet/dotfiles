usage:
	@echo all - Setup dev env

all: \
	1password \
	daisydisk \
	fantastical \
	flux \
	font-firacode \
	fzf \
	google-chrome \
	gpg \
	iterm2 \
	lftp \
	node \
	prezto \
	ranger \
	slack \
	slate \
	spotify \
	tadam \
	tmux \
	the_silver_searcher \
	vagrant \
	vim


1password: mas /Applications/1password.app
/Applications/1password.app:
	mas install 443987910

ansible: brew /usr/local/bin/ansible
/usr/local/bin/ansible:
	brew install ansible

brew: /usr/local/bin/brew
/usr/local/bin/brew:
	curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install > /tmp/brew-installer.sh
	chmod +x /tmp/brew-installer.sh
	/tmp/brew-installer.sh
	brew install caskroom/cask/brew-cask
	brew tap gapple/services
	brew tap caskroom/fonts

daisydisk: mas /Applications/DaisyDisk.app
/Applications/DaisyDisk.app:
	mas install 411643860

fantastical: mas /Applications/Fantastical\ 2.app
/Applications/Fantastical\ 2.app:
	mas install 975937182

flux: brew /Applications/Flux.app
/Applications/Flux.app:
	brew cask install flux

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

gpg: brew /usr/local/bin/gpg-agent /usr/local/bin/gpg /usr/local/bin/pinentry-mac
/usr/local/bin/gpg:
	brew install gpg
/usr/local/bin/gpg-agent:
	brew install gpg-agent
/usr/local/bin/pinentry-mac:
	brew install pinentry-mac

gpg-config:
	echo 'use-agent' > ~/.gnupg/gpg.conf
	echo 'use-standard-socket' > ~/.gnupg/gpg-agent.conf
	echo 'pinentry-program /usr/local/bin/pinentry-mac' >> ~/.gnupg/gpg-agent.conf

iterm2: brew /Applications/iTerm.app
/Applications/iTerm.app:
	brew install Caskroom/versions/iterm2-nightly

kwmc: brew /usr/local/bin/kwmc
/usr/local/bin/kwmc:
	brew install koekeishiya/kwm/kwm

lftp: brew /usr/local/bin/lftp
/usr/local/bin/lftp:
	brew install homebrew/boneyard/lftp

mas: brew /usr/local/bin/mas/
/usr/local/bin/mas/:
	brew install mas

node: /usr/local/bin/node
/usr/local/bin/node:
	brew install node

prezto: ~/.zprezto ~/.zpreztorc ~/.zlogin ~/.zlogout ~/.zprofile ~/.zshenv ~/.zshrc \
	~/.zprezto/modules/prompt/functions/prompt_seb_mini_setup \
	~/.zprezto/modules/prompt/functions/prompt_seb_setup
~/.zprezto:
	git clone --recursive https://github.com/sorin-ionescu/prezto.git $@
~/.zlogin ~/.zlogout ~/.zprofile ~/.zshenv:
	ln -s ~/.zprezto/runcoms/$(subst .,,$(notdir $@)) $@
~/.zshrc:
	ln -s ~/.dotfiles/zsh/zshrc $@
~/.zpreztorc:
	ln -s ~/.dotfiles/zsh/zpreztorc $@
~/.zprezto/modules/prompt/functions/prompt_seb_mini_setup:
	ln -s ~/.dotfiles/zsh/prompts/prompt_seb_mini_setup $@
~/.zprezto/modules/prompt/functions/prompt_seb_setup:
	ln -s ~/.dotfiles/zsh/prompts/prompt_seb_setup $@

ranger: brew ~/.config/ranger /usr/local/bin/ranger
/usr/local/bin/ranger:
	brew install ranger
~/.config/ranger: ~/.config
	ln -s ~/.dotfiles/ranger $@

slack: mas /Applications/Slack.app
/Applications/Slack.app:
	mas install 803453959

slate: brew ~/Applications/Slate.app ~/.slate.js
~/Applications/Slate.app:
	brew cask install mattr-slate
~/.slate.js:
	ln -s ~/.dotfiles/slate/.slate.js ~/.slate.js

spotify: brew /Applications/Spotify.app
/Applications/Spotify.app:
	brew cask install spotify

tadam: mas /Applications/Tadam.app
/Applications/Tadam.app:
	mas install 531349534

tmux: brew /usr/local/bin/tmux ~/.tmux.conf
/usr/local/bin/tmux:
	brew install reattach-to-user-namespace
	brew install tmux
~/.tmux.conf:
	ln -s ~/.dotfiles/tmux/.tmux.conf ~/.tmux.conf
	
the_silver_searcher: brew /usr/local/bin/ag
/usr/local/bin/ag:
	brew install the_silver_searcher

vagrant: brew virtualbox ansible /usr/local/bin/vagrant
/usr/local/bin/vagrant:
	brew cask install vagrant

vim: brew /usr/local/bin/vim /usr/local/bin/nvim ~/.vimrc
/usr/local/bin/vim:
	brew install vim

/usr/local/bin/nvim:
	brew install neovim/neovim/neovim
~/.config:
	mkdir $@
~/.vimrc:
	ln -s ~/.dotfiles/vim ~/.vim
	ln -s ~/.dotfiles/vim/.vimrc ~/.vimrc
	ln -s ~/.vim ~/.config/nvim
	ln -s ~/.vimrc ~/.config/nvim/init.vim
	cd ~/.vim && make

virtualbox: brew /usr/local/bin/VBoxHeadless
/usr/local/bin/VBoxHeadless:
	brew cask install virtualbox











font-sourcecode: ~/Library/Fonts/Sauce\ Code\ Powerline\ Light.otf
~/Library/Fonts/Sauce\ Code\ Powerline\ Light.otf:
	brew cask install font-source-code-pro-for-powerline

font-anonymous: ~/Library/Fonts/Anonymice\ Powerline.ttf
~/Library/Fonts/Anonymice\ Powerline.ttf:
	brew cask install font-anonymous-pro-for-powerline

soulseek: ~/Applications/SoulseekQt.app
~/Applications/SoulseekQt.app:
	brew cask install soulseek

imagemagick: brew /usr/local/bin/jpegtran
/usr/local/bin/jpegtran:
	brew install imagemagick

ghi: brew /usr/local/bin/ghi
/usr/local/bin/ghi:
	brew install ghi

youtube-dl: brew /usr/local/bin/youtube-dl
/usr/local/bin/youtube-dl:
	brew install youtube-dl

watch: brew /usr/local/bin/watch
/usr/local/bin/watch:
	brew install watch

mtr: brew /usr/local/bin/mtr
/usr/local/bin/mtr:
	brew install mtr

travis:
	gem install travis

siege: brew /usr/local/bin/siege
/usr/local/bin/siege:
	brew install siege


keycastr: brew /opt/homebrew-cask/Caskroom/keycastr/0.0.2-bezel/KeyCastr.app
/opt/homebrew-cask/Caskroom/keycastr/0.0.2-bezel/KeyCastr.app:
	brew cask install keycastr

skype: brew /opt/homebrew-cask/Caskroom/skype/latest/Skype.app
/opt/homebrew-cask/Caskroom/skype/latest/Skype.app:
	brew cask install skype

vlc: brew ~/Applications/VLC.app
~/Applications/VLC.app:
	brew cask install vlc

postman: brew ~/Applications/Postman.app
~/Applications/Postman.app:
	brew install Caskroom/cask/postman

telephone: brew /opt/homebrew-cask/Caskroom/telephone/latest/telephone.app
/opt/homebrew-cask/Caskroom/telephone/latest/telephone.app:
	brew cask install telephone

sequel-pro: brew /opt/homebrew-cask/Caskroom/sequel-pro/latest/sequel-pro.app
/opt/homebrew-cask/Caskroom/sequel-pro/latest/sequel-pro.app:
	brew cask install sequel-pro

keepassx: brew /opt/homebrew-cask/Caskroom/keepassx/latest/keepassx.app
/opt/homebrew-cask/Caskroom/keepassx/latest/keepassx.app:
	brew cask install keepassx

dropbox: brew /opt/homebrew-cask/Caskroom/dropbox/latest/dropbox.app
/opt/homebrew-cask/Caskroom/dropbox/latest/dropbox.app:
	brew cask install dropbox


packer: brew /usr/local/bin/packer
/usr/local/bin/packer:
	brew cask install packer

java: brew
	brew cask install java

cloc: brew
	brew install cloc

jq: brew /usr/local/bin/jq
/usr/local/bin/jq:
	brew install jq

highlight: brew /usr/local/bin/highlight
/usr/local/bin/highlight:
	brew install highlight

ncdu: brew /usr/local/bin/ncdu
/usr/local/bin/ncdu:
	brew install ncdu

tig: brew /usr/local/bin/tig
/usr/local/bin/tig:
	brew install tig

instant-markdown: /usr/local/bin/instant-markdown-d
/usr/local/bin/instant-markdown-d:
	npm -g install instant-markdown-d

cli-tools: how2

how2: node /usr/local/bin/how2
/usr/local/bin/how2:
	@npm -g install how2

devtool: node /usr/local/bin/devtool
/usr/local/bin/devtool:
	@npm -g install devtool

pgcli: brew /usr/local/bin/pgcli
/usr/local/bin/pgcli:
	brew install pgcli

memcached: brew /usr/local/bin/memcached
/usr/local/bin/memcached:
	brew install memcached

redis: brew /usr/local/bin/redis-server
/usr/local/bin/redis-server:
	brew install redis

mongohacker: node /usr/local/lib/node_modules/mongo-hacker
/usr/local/lib/node_modules/mongo-hacker:
	@npm install -g mongo-hacker

mariadb: brew /usr/local/bin/mysql
/usr/local/bin/mysql:
	brew install mariadb

mycli: brew /usr/local/bin/mycli
/usr/local/bin/mycli:
	brew install mycli

mutt: brew /usr/local/bin/mutt
/usr/local/bin/mutt:
	brew install mutt

mutt-config: ~/.muttrc
~/.muttrc:
	ln -s ~/.dotfiles/muttrc ~/.muttrc
	ln -s ~/.dotfiles/mutt ~/.mutt

fetchmail: brew /usr/local/bin/fetchmail
/usr/local/bin/fetchmail:
	brew install fetchmail

offlineimap: brew /usr/local/bin/offlineimap
/usr/local/bin/offlineimap:
	brew install offlineimap

cordova: node /usr/local/bin/cordova
/usr/local/bin/cordova:
	@npm install -g cordova

ionic: node /usr/local/bin/ionic
/usr/local/bin/ionic:
	@npm install -g ionic

clif: /usr/local/bin/clif
/usr/local/bin/clif:
	@npm install -g clif

jsonlint: node /usr/local/bin/jsonlint
/usr/local/bin/jsonlint:
	@npm install -g jsonlint

js-yaml: /usr/local/bin/js-yaml
/usr/local/bin/js-yaml:
	@npm install -g js-yaml

jsinspect: node /usr/local/bin/jsinspect
/usr/local/bin/jsinspect:
	@npm install -g jsinspect

retire: node /usr/local/bin/retire
/usr/local/bin/retire:
	@npm install -g retire

david: node /usr/local/bin/david
/usr/local/bin/david:
	@npm install -g david

nsp: node /usr/local/bin/nsp
/usr/local/bin/nsp:
	@npm install -g nsp

recess:
	@npm install -g recess

sqlint: /usr/local/bin/sqlint
/usr/local/bin/sqlint:
	@gem install sqlint

pm2: node /usr/local/bin/pm2
/usr/local/bin/pm2:
	@npm install -g pm2

cmus: brew /usr/local/bin/cmus
/usr/local/bin/cmus:
	brew install cmus


wallpaper:
	# Set wallpaper
	osascript -e "tell application \"System Events\" to set picture of every \
		desktop to \"/Library/Desktop Pictures/Color Burst 2.jpg\""

xz: brew
/usr/local/bin/xz:
	brew install xz

