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
