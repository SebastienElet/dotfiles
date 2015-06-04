COMPUTER_NAME := 'MBA'

usage:
	@echo brew - Install brew and brew-cask
	@echo vagrant - Install vagrant, packer and plugins

brew: /usr/local/bin/brew
/usr/local/bin/brew:
	curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install > /tmp/brew-installer.sh
	chmod +x /tmp/brew-installer.sh
	/tmp/brew-installer.sh
	brew install caskroom/cask/brew-cask
	brew tap gapple/services

iterm2: /Applications/iTerm.app
/Applications/iTerm.app:
	git clone https://github.com/Nasga/iterm2-borderless.git /tmp/iterm2
	mv /tmp/iterm2/iTerm.app /Applications/
	rm -rf /tmp/iterm2

zsh: ~/.zshrc
~/.zshrc:
	ln -s ~/.dotfiles/zsh/.zshrc ~/.zshrc

fzf: ~/.fzf
~/.fzf:
	git clone https://github.com/junegunn/fzf.git ~/.fzf
	~/.fzf/install

slate: brew /opt/homebrew-cask/Caskroom/slate/latest/Slate.app
/opt/homebrew-cask/Caskroom/slate/latest/Slate.app:
	brew cask install mattr-slate

keycastr: brew /opt/homebrew-cask/Caskroom/keycastr/0.0.2-bezel/KeyCastr.app
/opt/homebrew-cask/Caskroom/keycastr/0.0.2-bezel/KeyCastr.app:
	brew cask install keycastr

slate-config: slate ~/.slate.js
~/.slate.js:
	ln -s ~/.dotfiles/slate/.slate.js ~/.slate.js

skype: brew /opt/homebrew-cask/Caskroom/skype/latest/Skype.app
/opt/homebrew-cask/Caskroom/skype/latest/Skype.app:
	brew cask install skype

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

chef: brew /usr/bin/chef
/usr/bin/chef:
	brew cask install chefdk

virtualbox: brew /usr/bin/VBoxHeadless
/usr/bin/VBoxHeadless:
	brew cask install virtualbox

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

vagrant: brew virtualbox chef packer /usr/bin/vagrant
/usr/bin/vagrant:
	brew cask install vagrant

docker: brew virtualbox
	brew install docker
	brew install boot2docker
	/usr/local/bin/boot2docker init

highlight: brew /usr/local/bin/highlight
/usr/local/bin/highlight:
	brew install highlight

tig: brew /usr/local/bin/tig
/usr/local/bin/tig:
	brew install tig

vim: brew /usr/local/bin/vim
/usr/local/bin/vim:
	brew install vim

vim-config: ~/.vimrc
~/.vimrc:
	ln -s ~/.dotfiles/vim ~/.vim
	ln -s ~/.dotfiles/vim/.vimrc ~/.vimrc

tmux: brew /usr/local/bin/tmux
/usr/local/bin/tmux:
	brew install reattach-to-user-namespace
	brew install tmux

tmux-config: tmux ~/.tmux.conf
~/.tmux.conf:
	ln -s ~/.dotfiles/tmux/.tmux.conf ~/.tmux.conf

the_silver_searcher: brew /usr/local/bin/ag
/usr/local/bin/ag:
	brew install the_silver_searcher

memcached: brew /usr/local/bin/memcached
/usr/local/bin/memcached:
	brew install memcached

redis: brew /usr/local/bin/redis-server
/usr/local/bin/redis-server:
	brew install redis

mongodb: brew
	brew install mongodb

mariadb: brew /usr/local/bin/mysql
/usr/local/bin/mysql:
	brew install mariadb

mutt: brew /usr/local/bin/mutt
/usr/local/bin/mutt:
	brew install mutt

fetchmail: brew /usr/local/bin/fetchmail
/usr/local/bin/fetchmail:
	brew install fetchmail

php56:
	brew install homebrew/php/php56
	brew install homebrew/php/php56-memcache

phpcs: ~/.composer/vendor/bin/phpcs
~/.composer/vendor/bin/phpcs:
	@composer global require "squizlabs/php_codesniffer=*"

phpcs-rules: phpcs
	ln -s $(shell pwd)/.phpcs.xml ~/.phpcs.xml
	phpcs --config-set default_standard ~/.phpcs.xml

php-cs-fixer: brew
	brew install php-cs-fixer

phpmd: ~/.composer/vendor/bin/phpmd
~/.composer/vendor/bin/phpmd:
	@composer global require "phpmd/phpmd=*"

node: /usr/local/bin/node
/usr/local/bin/node:
	brew install node

jscs: node ~/.jscs.json
~/.jscs.json:
	@npm install -g jscs
	ln -s ~/.dotfiles/.jscs.json ~/.jscs.json

pomo:
	@npm install -g pomo

clif: /usr/local/bin/clif
/usr/local/bin/clif:
	@npm install -g clif

jshint: node ~/.jshintrc
~/.jshintrc:
	@npm install -g jshint
	ln -s $(shell pwd)/.jshintrc ~/.jshintrc

eslint: node ~/.eslintrc
~/.eslintrc:
	@npm install -g eslint
	ln -s $(shell pwd)/.eslintrc ~/.eslintrc

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

pm2: node /usr/local/bin/pm2
/usr/local/bin/pm2:
	@npm install -g pm2

cmus: brew /usr/local/bin/cmus
/usr/local/bin/cmus:
	brew install cmus

osx:
	# See secrets.blacktree.com
	chsh -s /bin/zsh $(USER)
	sudo scutil --set ComputerName $(COMPUTER_NAME)
	sudo scutil --set HostName $(COMPUTER_NAME)
	sudo scutil --set LocalHostName $(COMPUTER_NAME)
	sudo defaults write \
		/Library/Preferences/SystemConfiguration/com.apple.smb.server \
		NetBIOSName -string $(COMPUTER_NAME)
	# Allow apps downloaded from "Anywhere"
	sudo spctl --master-disable
	# Disabled shadow in screenshots
	@defaults write com.apple.screencapture disable-shadow -bool true
	# Enalble Ctrl+Alt+cmd+t for darkmode
	@sudo defaults write /Library/Preferences/.GlobalPreferences.plist \
		_HIEnableThemeSwitchHotKey -bool true 
	# no .DS_Store on network
	defaults write com.apple.desktopservices DSDontWriteNetworkStores -bool true
	# Disable the “Are you sure you want to open this application?” dialog
	defaults write com.apple.LaunchServices LSQuarantine -bool false
	# Don’t automatically rearrange Spaces based on most recent use
	defaults write com.apple.dock mru-spaces -bool false
	# Trackpad: enable tap to click for this user and for the login screen
	defaults write com.apple.driver.AppleBluetoothMultitouch.trackpad \
		Clicking -bool true
	defaults -currentHost write NSGlobalDomain com.apple.mouse.tapBehavior -int 1
	defaults write NSGlobalDomain com.apple.mouse.tapBehavior -int 1
	# Disable "Natural" scroll
	defaults write NSGlobalDomain com.apple.swipescrolldirection -bool false
	# Enable move with 3 fingers
	defaults write com.apple.driver.AppleBluetoothMultitouch.trackpad \
		TrackpadThreeFingerDrag -bool true
	# dock size & autohidden dock
	defaults write com.apple.dock tilesize -int 128
	defaults write com.apple.dock autohide -bool true
	# Enable full keyboard access for all controls
	# (e.g. enable Tab in modal dialogs)
	defaults write NSGlobalDomain AppleKeyboardUIMode -int 3
	# Set a blazingly fast keyboard repeat rate
	defaults write NSGlobalDomain KeyRepeat -int 0
	# 14 days on ical
	defaults write com.apple.iCal n\ days\ of\ week 14
	# Finder
	defaults write com.apple.finder NewWindowTarget -string "PfLo"
	defaults write com.apple.finder NewWindowTargetPath -string "file://$(HOME)/Downloads/"

wallpaper:
	# Set wallpaper
	osascript -e "tell application \"System Events\" to set picture of every \
		desktop to \"~/.dotfiles/wallpapers/2.png\""



all: brew \
	zsh-config \
	vim-config \
	slate-config \
	node \
	vagrant \
	highlight \
	tig \
	vim \
	tmux \
	the_silver_searcher \
	memcached \
	redis \
	mutt \
	mariadb \
	mongodb \
	fetchmail \
	phpcs \
	phpmd \
	jsinspect \
	david \
	nsp \
	retire
