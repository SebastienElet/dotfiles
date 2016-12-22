COMPUTER_NAME := 'MBPR'

apps:
	# Allow apps downloaded from "Anywhere"
	sudo spctl --master-disable
	# Disable the “Are you sure you want to open this application?” dialog
	defaults write com.apple.LaunchServices LSQuarantine -bool false

computer-name:
	# See secrets.blacktree.com
	chsh -s /bin/zsh $(USER)
	sudo scutil --set ComputerName $(COMPUTER_NAME)
	sudo scutil --set HostName $(COMPUTER_NAME)
	sudo scutil --set LocalHostName $(COMPUTER_NAME)
	sudo defaults write \
		/Library/Preferences/SystemConfiguration/com.apple.smb.server \
		NetBIOSName -string $(COMPUTER_NAME)

screenshots:
	# Disabled shadow in screenshots
	@defaults write com.apple.screencapture disable-shadow -bool true

darkmode:
	# Enalble Ctrl+Alt+cmd+t for darkmode
	@sudo defaults write /Library/Preferences/.GlobalPreferences.plist \
		_HIEnableThemeSwitchHotKey -bool true 

network:
	# no .DS_Store on network
	defaults write com.apple.desktopservices DSDontWriteNetworkStores -bool true

finder:
	# Finder
	defaults write com.apple.finder NewWindowTarget -string "PfLo"
	defaults write com.apple.finder NewWindowTargetPath -string "file://$(HOME)/Downloads/"

spaces:
	# Don’t automatically rearrange Spaces based on most recent use
	defaults write com.apple.dock mru-spaces -bool false

trackpad:
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

dock:
	# dock size & autohidden dock
	defaults write com.apple.dock tilesize -int 128
	defaults write com.apple.dock autohide -bool true

ical:
	# 14 days on ical
	defaults write com.apple.iCal n\ days\ of\ week 14

keyboard:
	# Enable full keyboard access for all controls
	# (e.g. enable Tab in modal dialogs)
	defaults write NSGlobalDomain AppleKeyboardUIMode -int 3
	# Set a blazingly fast keyboard repeat rate
	defaults write NSGlobalDomain KeyRepeat -int 0

all: apps \
	keyboard \
	ical \
	dock \
	trackpad \
	spaces \
	finder \
	network \
	darkmode \
	screenshots \
	computer-name
