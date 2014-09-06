usage:
	@echo brew - Install brew and brew-cask
	@echo vagrant - Install vagrant, packer and plugins

brew: /usr/local/bin/brew
/usr/local/bin/brew:
	ruby -e "$(curl -fsSL https://raw.github.com/Homebrew/homebrew/go/install)"
	brew install caskroom/cask/brew-cask

chef: brew /usr/bin/chef
/usr/bin/chef:
	brew cask install chefdk

virtualbox: brew /usr/bin/VBoxHeadless
/usr/bin/VBoxHeadless:
	brew cask install virtualbox

packer: brew /usr/local/bin/packer
/usr/local/bin/packer:
	brew cask install packer

vagrant: brew virtualbox chef packer /usr/bin/vagrant
/usr/bin/vagrant:
	brew cask install vagrant

highlight: brew /usr/local/bin/highlight
/usr/local/bin/highlight:
	brew install highlight

tig: brew /usr/local/bin/tig
/usr/local/bin/tig:
	brew install tig

vim: brew /usr/local/bin/vim
/usr/local/bin/vim:
	brew install vim

tmux: brew /usr/local/bin/tmux
/usr/local/bin/tmux:
	brew install tmux

the_silver_searcher: brew /usr/local/bin/ag
/usr/local/bin/ag:
	brew install the_silver_searcher

memcached: brew /usr/local/bin/memcached
/usr/local/bin/memcached:
	brew install memcached

redis: brew /usr/local/bin/redis-cli
/usr/local/bin/redis-cli:
	brew install redis

mariadb: brew /usr/local/bin/mysql
/usr/local/bin/mysql:
	brew install mariadb

mutt: brew /usr/local/bin/mutt
/usr/local/bin/mutt:
	brew install mutt

fetchmail: brew /usr/local/bin/fetchmail
/usr/local/bin/fetchmail:
	brew install fetchmail

php55:

phpcs: /usr/local/opt/php55/bin/phpcs php55
/usr/local/opt/php55/bin/phpcs:
	@sudo pear install PHP_CodeSniffer

phpmd: /usr/local/opt/php55/bin/phpmd php55
/usr/local/opt/php55/bin/phpmd:
	@sudo pear channel-discover pear.phpmd.org
	@sudo pear channel-discover pear.pdepend.org
	@sudo pear install --alldeps phpmd/PHP_PMD

node: /usr/local/bin/node
/usr/local/bin/node:
	brew install node

jscs: node
	@npm install -g jscs
	ln -s .jscs.json ~/.jscs.json

jshint: node
	@npm install -g jshint
	ln -s .jshintrc ~/.jshintrc

all: brew \
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
	fetchmail \
	phpcs \
	phpmd
