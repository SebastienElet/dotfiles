BREW_BIN:=$(shell if [ "$(shell uname -p)" = "arm" ]; then echo "/opt/homebrew/bin"; else echo "/usr/local/bin"; fi)
BREW_GNU_BIN:=$(shell if [ "$(shell uname -p)" = "arm" ]; then echo "/opt/homebrew/opt"; else echo "/usr/local/opt"; fi)
VOLTA_BIN:=$(HOME)/.volta/bin
PNPM_BIN:=$(HOME)/Library/pnpm
LOCAL_BIN:=$(HOME)/.local/bin
BRAIN_PATH?=$(HOME)/Library/Mobile Documents/com~apple~CloudDocs/Brain
APP_BIN:=/Applications
SCRAPLING_IMAGE?=pyd4vinci/scrapling
CLOAKBROWSER_IMAGE?=cloakhq/cloakbrowser:0.5.3
# DOTFILES_PATH should be ~/.dotfiles when installed normally
DOTFILES_PATH:=$(shell pwd)
# SKIP_PAID_APPS: set to 1 to skip paid Mac App Store apps (useful for CI)
SKIP_PAID_APPS?=0
# Avoid Homebrew confirmation prompts during setup.
export HOMEBREW_NO_ASK:=1
# HAS_BREW_TRUST: check if brew trust command is available (Homebrew >= 5.1.15)
HAS_BREW_TRUST:=$(shell brew trust --help >/dev/null 2>&1 && echo yes || echo no)
# Recipe guard: no Docker daemon in CI, and none right after a fresh Orbstack install.
DOCKER_OR_SKIP=docker info >/dev/null 2>&1 || { echo "Docker unavailable, skipping $@"; exit 0; }

.PHONY: usage
usage:
	@echo all - Setup dev env

.PHONY: utils
utils: \
	cleanshot \
	handy \
	rectangle-pro \
	things-3

# Homebrew ignores untrusted taps and warns on every command until they are trusted.
# Runners ship pre-tapped untrusted taps, hence the sweep before a full install.
.PHONY: trust-taps
trust-taps: brew
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew tap | xargs -n1 brew trust --tap 2>&1 | grep -v '^Already trusted' || true; fi

.PHONY: all
all: \
	trust-taps \
	extra \
	terminal \
	work \
	utils \
	personal

.PHONY: extra
extra: \
	daisydisk \
	font-jetbrains-mono

################################################################################
# Terminal section
################################################################################
.PHONY: terminal
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

.PHONY: bat
bat: brew ${BREW_BIN}/bat ~/.config/bat/themes/Catppuccin\ Latte.tmTheme
${BREW_BIN}/bat:
	brew install bat
~/.config/bat/themes:
	mkdir -p $@
~/.config/bat/themes/Catppuccin\ Latte.tmTheme: | ~/.config/bat/themes
	curl -L -o ~/.config/bat/themes/Catppuccin\ Latte.tmTheme https://github.com/catppuccin/bat/raw/main/themes/Catppuccin%20Latte.tmTheme
	curl -L -o ~/.config/bat/themes/Catppuccin\ Mocha.tmTheme https://github.com/catppuccin/bat/raw/main/themes/Catppuccin%20Mocha.tmTheme
	bat cache --build

.PHONY: bottom
bottom: brew ${BREW_BIN}/btm
${BREW_BIN}/btm:
	brew install bottom

.PHONY: broot
broot: brew ${BREW_BIN}/broot
${BREW_BIN}/broot:
	brew install broot

.PHONY: procs
procs: brew ${BREW_BIN}/procs
${BREW_BIN}/procs:
	brew install procs

.PHONY: eza
eza: brew ${BREW_BIN}/eza
${BREW_BIN}/eza:
	brew install eza

.PHONY: fd
fd: brew ${BREW_BIN}/fd
${BREW_BIN}/fd:
	brew install fd

.PHONY: fish
fish: brew starship ~/.config/fish ${BREW_BIN}/fish
${BREW_BIN}/fish:
	brew install fish fisher
	fish -c 'fisher install PatrickF1/fzf.fish'
	@echo 'If you want to switch your shell to fish, please run the following command'
	@echo '$> sudo chpass -s ${BREW_BIN}/fish ${USER}'

~/.config/fish: ${DOTFILES_PATH}/fish | ~/.config
	ln -s ${DOTFILES_PATH}/fish $@

.PHONY: gnu-sed
gnu-sed: brew ${BREW_GNU_BIN}/gnu-sed
${BREW_GNU_BIN}/gnu-sed:
	brew install gnu-sed

.PHONY: htop
htop: brew ${BREW_BIN}/htop
${BREW_BIN}/htop:
	brew install htop

.PHONY: lazygit
lazygit: brew ${BREW_BIN}/lazygit
${BREW_BIN}/lazygit:
	brew install lazygit

.PHONY: tokei
tokei: brew ${BREW_BIN}/tokei
${BREW_BIN}/tokei:
	brew install tokei

.PHONY: wezterm
wezterm: brew font-jetbrains-mono font-iosevka-nerd-font /Applications/WezTerm.app ~/.wezterm.lua
/Applications/WezTerm.app:
	brew tap wez/wezterm
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --tap wez/wezterm; fi
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --cask wez/wezterm/wezterm-nightly; fi
	brew install --cask wez/wezterm/wezterm-nightly
~/.wezterm.lua: ${DOTFILES_PATH}/.wezterm.lua
	ln -s ${DOTFILES_PATH}/.wezterm.lua $@

################################################################################
# End of the terminal section
################################################################################

################################################################################
# Work section
################################################################################
.PHONY: work
work: \
	arc \
	aws \
	ai \
	bkt \
	daily-routine \
	flow \
	language-tool \
	qovery-cli \
	docker \
	doppler \
	frontcli \
	gh \
	google-chrome \
	javascript \
	k9s \
	lazydocker \
	linear-cli \
	mosh \
	pnpm \
	postgresql \
	renovate \
	tableplus \
	terraform \
	uv \
	1password \
	vibe-island

.PHONY: ai
ai: \
	brain \
	chatgpt \
	claude \
	claude-code \
	cloakbrowser \
	codegraph \
	codex \
	codexbar \
	cursor \
	firecrawl \
	googleworkspace-cli \
	llmfit \
	mistral-vibe \
	opencode \
	openspec \
	pi-coding-agent \
	scrapling \
	skills

.PHONY: brain
brain:
	@if [ ! -d "$(BRAIN_PATH)" ]; then \
		exit 0; \
	fi; \
	if [ -L "$(HOME)/Brain" ]; then \
		if [ "$$(readlink "$(HOME)/Brain")" = "$(BRAIN_PATH)" ]; then \
			echo "Brain symlink already configured"; \
		else \
			echo "Error: $(HOME)/Brain is not the expected symbolic link" >&2; \
			exit 1; \
		fi; \
	elif [ -e "$(HOME)/Brain" ]; then \
		echo "Error: $(HOME)/Brain already exists and is not a symbolic link" >&2; \
		exit 1; \
	else \
		ln -s "$(BRAIN_PATH)" "$(HOME)/Brain" && \
		echo "Created $(HOME)/Brain symlink"; \
	fi

.PHONY: arc
arc: brew ${APP_BIN}/Arc.app
${APP_BIN}/Arc.app:
	brew install --cask arc

.PHONY: aws
aws: brew ${BREW_BIN}/aws
${BREW_BIN}/aws:
	brew install awscli

.PHONY: bkt
bkt: brew ${BREW_BIN}/bkt
${BREW_BIN}/bkt:
	brew tap avivsinai/tap
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --tap avivsinai/tap; fi
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --formula avivsinai/tap/bitbucket-cli; fi
	brew install avivsinai/tap/bitbucket-cli

.PHONY: daily-routine
daily-routine: ${LOCAL_BIN}/daily-routine

.PHONY: check-daily-routine-destination
check-daily-routine-destination:
	@if [ -L "${LOCAL_BIN}/daily-routine" ]; then \
		if [ "$$(readlink "${LOCAL_BIN}/daily-routine")" != "${DOTFILES_PATH}/daily-routine/target/release/daily-routine" ]; then \
			echo "Error: ${LOCAL_BIN}/daily-routine is not the expected daily-routine release symlink" >&2; \
			exit 1; \
		fi; \
	elif [ -e "${LOCAL_BIN}/daily-routine" ]; then \
		echo "Error: ${LOCAL_BIN}/daily-routine already exists and is not a symbolic link" >&2; \
		exit 1; \
	fi

${LOCAL_BIN}/daily-routine: \
	${DOTFILES_PATH}/daily-routine/Cargo.toml \
	${DOTFILES_PATH}/daily-routine/Cargo.lock \
	${DOTFILES_PATH}/daily-routine/config.example.toml \
	$(wildcard ${DOTFILES_PATH}/daily-routine/src/*.rs) \
	| ${LOCAL_BIN} rust check-daily-routine-destination
	cd ${DOTFILES_PATH}/daily-routine && ${BREW_BIN}/cargo build --release
	ln -sf ${DOTFILES_PATH}/daily-routine/target/release/daily-routine $@

.PHONY: docker
docker: brew lazydocker /Applications/Orbstack.app
/Applications/Orbstack.app:
	brew install orbstack

.PHONY: doppler
doppler: gnupg ${BREW_BIN}/doppler
${BREW_BIN}/doppler:
	brew tap dopplerhq/cli
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --tap dopplerhq/cli; fi
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --formula dopplerhq/cli/doppler; fi
	brew install dopplerhq/cli/doppler

.PHONY: gnupg
gnupg: brew ${BREW_BIN}/gpg
${BREW_BIN}/gpg:
	brew install gnupg

.PHONY: gh
gh: brew ${BREW_BIN}/gh
${BREW_BIN}/gh:
	brew install gh

.PHONY: google-chrome
google-chrome: brew ${APP_BIN}/Google\ Chrome.app
${APP_BIN}/Google\ Chrome.app:
	brew install --cask google-chrome

.PHONY: k9s
k9s: brew ${BREW_BIN}/k9s
${BREW_BIN}/k9s:
	brew install k9s

.PHONY: lazydocker
lazydocker: brew ${BREW_BIN}/lazydocker
${BREW_BIN}/lazydocker:
	brew tap jesseduffield/lazydocker
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --tap jesseduffield/lazydocker; fi
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --formula jesseduffield/lazydocker/lazydocker; fi
	brew install jesseduffield/lazydocker/lazydocker

.PHONY: meteor
meteor: ~/.meteor/meteor
~/.meteor/meteor:
	curl https://install.meteor.com/ | sh

.PHONY: mongosh
mongosh: brew ${BREW_BIN}/mongosh
${BREW_BIN}/mongosh:
	brew install mongosh

.PHONY: mosh
mosh: brew ${BREW_BIN}/mosh
${BREW_BIN}/mosh:
	brew install mosh

.PHONY: postgresql
postgresql: brew ${BREW_GNU_BIN}/postgresql@16/bin/psql ~/.psqlrc
${BREW_GNU_BIN}/postgresql@16/bin/psql:
	brew install postgresql@16
~/.psqlrc: ${DOTFILES_PATH}/.psqlrc
	ln -s ${DOTFILES_PATH}/.psqlrc $@

.PHONY: renovate
renovate: brew ${VOLTA_BIN}/renovate
${VOLTA_BIN}/renovate: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g renovate

.PHONY: tableplus
tableplus: brew ${APP_BIN}/TablePlus.app
${APP_BIN}/TablePlus.app:
	brew install tableplus

.PHONY: terraform
terraform: brew ${BREW_BIN}/terraform
${BREW_BIN}/terraform:
	brew tap hashicorp/tap
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --tap hashicorp/tap; fi
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --formula hashicorp/tap/terraform; fi
	brew install hashicorp/tap/terraform

.PHONY: uv
uv: brew ${BREW_BIN}/uv
${BREW_BIN}/uv:
	brew install uv

.PHONY: 1password
1password: brew ${APP_BIN}/1Password.app
${APP_BIN}/1Password.app:
	brew install --cask 1password

.PHONY: cursor
cursor: ~/.local/bin/cursor-agent ~/.cursor/skills/enforcement-code ~/.cursor/skills/merge-verdict
~/.local/bin/cursor-agent:
	curl https://cursor.com/install -fsS | bash
# Personal skills go in ~/.cursor/skills; ~/.cursor/skills-cursor holds Cursor's
# own built-ins and is resynchronised from its registry, so a link dropped there
# would be wiped without warning. Inside this repository .cursor/skills already
# points at .agents/skills, so only the global link is needed.
~/.cursor/skills:
	mkdir -p $@
~/.cursor/skills/enforcement-code: ${DOTFILES_PATH}/.agents/skills/enforcement-code | ~/.cursor/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/enforcement-code $@
~/.cursor/skills/merge-verdict: ${DOTFILES_PATH}/.agents/skills/merge-verdict | ~/.cursor/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/merge-verdict $@

.PHONY: claude-code
claude-code: ${LOCAL_BIN}/claude ~/.claude/CLAUDE.md ~/.claude/SOUL.md ~/.claude/USER.md ~/.claude/hooks/agent_handoff ~/.claude/skills/handoff ~/.claude/skills/enforcement-code ~/.claude/skills/merge-verdict claude-handoff-hook
${LOCAL_BIN}/claude:
	curl -fsSL https://claude.ai/install.sh | bash -s latest
~/.claude:
	mkdir -p $@
~/.claude/CLAUDE.md: ${DOTFILES_PATH}/ai/AGENTS.md | ~/.claude
	ln -s ${DOTFILES_PATH}/ai/AGENTS.md $@
# Imported by AGENTS.md; linked as siblings so the @import resolves whether the
# tool follows the symlink or reads it from the destination directory.
~/.claude/SOUL.md: ${DOTFILES_PATH}/ai/SOUL.md | ~/.claude
	ln -s ${DOTFILES_PATH}/ai/SOUL.md $@
~/.claude/USER.md: ${DOTFILES_PATH}/ai/USER.md | ~/.claude
	ln -s ${DOTFILES_PATH}/ai/USER.md $@
~/.claude/hooks ~/.claude/skills: | ~/.claude
	mkdir -p $@
~/.claude/hooks/agent_handoff: ${DOTFILES_PATH}/scripts/agent_handoff | ~/.claude/hooks
	ln -s ${DOTFILES_PATH}/scripts/agent_handoff $@
~/.claude/skills/handoff: ${DOTFILES_PATH}/.agents/skills/handoff | ~/.claude/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/handoff $@
~/.claude/skills/enforcement-code: ${DOTFILES_PATH}/.agents/skills/enforcement-code | ~/.claude/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/enforcement-code $@
# Linked globally because a pull request is reviewed from the repository under
# review, which is never this one.
~/.claude/skills/merge-verdict: ${DOTFILES_PATH}/.agents/skills/merge-verdict | ~/.claude/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/merge-verdict $@

.PHONY: claude-handoff-hook
claude-handoff-hook: ${BREW_BIN}/jq | ~/.claude
	@set -e; \
	umask 077; \
	command='${DOTFILES_PATH}/scripts/agent_handoff'; \
	tmp=$$(mktemp ~/.claude/settings.json.XXXXXX); \
	trap 'rm -f "$$tmp"' EXIT HUP INT TERM; \
	if [ -f ~/.claude/settings.json ]; then input=~/.claude/settings.json; else input=/dev/null; fi; \
	jq -n --arg command "$$command" --slurpfile current "$$input" '($$current[0] // {}) | .hooks.Stop //= [] | if any(.hooks.Stop[]?.hooks[]?; .command == $$command) then . else .hooks.Stop += [{hooks: [{type: "command", command: $$command}]}] end' > "$$tmp"; \
	mv "$$tmp" ~/.claude/settings.json; \
	trap - EXIT HUP INT TERM

.PHONY: codex
codex: ${VOLTA_BIN}/codex ${BREW_BIN}/jq ~/.codex/AGENTS.md ~/.agents/skills/handoff ~/.agents/skills/enforcement-code ~/.agents/skills/merge-verdict codex-handoff-hook
${VOLTA_BIN}/codex: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g @openai/codex
~/.codex:
	mkdir -p $@
# Codex ignores AGENTS.md @import directives, so the three files are assembled
# here instead of symlinked. Written to a temporary path then moved, so an
# existing symlink is replaced rather than written through.
~/.codex/AGENTS.md: ${DOTFILES_PATH}/ai/AGENTS.md ${DOTFILES_PATH}/ai/SOUL.md ${DOTFILES_PATH}/ai/USER.md | ~/.codex
	grep -v '^@' $< | cat - ${DOTFILES_PATH}/ai/SOUL.md ${DOTFILES_PATH}/ai/USER.md > $@.tmp
	mv $@.tmp $@
# Only the global link: inside this repository Codex reads .agents/skills directly.
~/.agents/skills:
	mkdir -p $@
~/.agents/skills/enforcement-code: ${DOTFILES_PATH}/.agents/skills/enforcement-code | ~/.agents/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/enforcement-code $@
~/.agents/skills/handoff: ${DOTFILES_PATH}/.agents/skills/handoff | ~/.agents/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/handoff $@
~/.agents/skills/merge-verdict: ${DOTFILES_PATH}/.agents/skills/merge-verdict | ~/.agents/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/merge-verdict $@

.PHONY: codex-handoff-hook
codex-handoff-hook: | ~/.codex
	@command='${DOTFILES_PATH}/scripts/agent_handoff'; \
	tmp=~/.codex/hooks.json.tmp; \
	if [ -f ~/.codex/hooks.json ]; then input=~/.codex/hooks.json; else input=/dev/null; fi; \
	jq -n --arg command "$$command" --slurpfile current "$$input" '($$current[0] // {}) | .hooks.Stop //= [] | if any(.hooks.Stop[]?.hooks[]?; .command == $$command) then . else .hooks.Stop += [{hooks: [{type: "command", command: $$command}]}] end' > "$$tmp"; \
	mv "$$tmp" ~/.codex/hooks.json

.PHONY: codexbar
codexbar: brew ${APP_BIN}/CodexBar.app
${APP_BIN}/CodexBar.app:
	brew install --cask codexbar

.PHONY: codegraph
codegraph: ${VOLTA_BIN}/codegraph
${VOLTA_BIN}/codegraph: ${VOLTA_BIN}/node
	${BREW_BIN}/volta install @colbymchenry/codegraph

.PHONY: googleworkspace-cli
googleworkspace-cli: ${VOLTA_BIN}/gws
${VOLTA_BIN}/gws: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g @googleworkspace/cli

.PHONY: llmfit
llmfit: brew ${BREW_BIN}/llmfit
${BREW_BIN}/llmfit:
	brew tap AlexsJones/llmfit
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --tap AlexsJones/llmfit; fi
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --formula AlexsJones/llmfit/llmfit; fi
	brew install AlexsJones/llmfit/llmfit

.PHONY: mistral-vibe
mistral-vibe: ${LOCAL_BIN}/vibe
${LOCAL_BIN}/vibe: | uv
	${BREW_BIN}/uv tool install mistral-vibe

.PHONY: opencode
opencode: brew ${BREW_BIN}/opencode
${BREW_BIN}/opencode:
	brew tap anomalyco/tap
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --tap anomalyco/tap; fi
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --formula anomalyco/tap/opencode; fi
	brew install anomalyco/tap/opencode

.PHONY: qovery-cli
qovery-cli: /usr/local/bin/qovery
/usr/local/bin/qovery:
	# Homebrew version is outdated, use the upstream installer
	curl -s https://get.qovery.com | bash

.PHONY: firecrawl
firecrawl: docker
	@$(DOCKER_OR_SKIP); docker compose -f ${DOTFILES_PATH}/ai/firecrawl/compose.yml up -d

.PHONY: scrapling
scrapling: docker ${LOCAL_BIN}/scrapling_mcp
	@$(DOCKER_OR_SKIP); docker image inspect ${SCRAPLING_IMAGE} >/dev/null 2>&1 || docker pull ${SCRAPLING_IMAGE}

# MCP command for agents: starts the shared container on demand instead of one per session.
${LOCAL_BIN}/scrapling_mcp: ${DOTFILES_PATH}/scripts/scrapling_mcp | ${LOCAL_BIN}
	ln -s ${DOTFILES_PATH}/scripts/scrapling_mcp $@
${LOCAL_BIN}:
	mkdir -p $@

.PHONY: cloakbrowser
cloakbrowser: docker
	@$(DOCKER_OR_SKIP); docker image inspect ${CLOAKBROWSER_IMAGE} >/dev/null 2>&1 || docker pull ${CLOAKBROWSER_IMAGE}

.PHONY: skills
skills: ${VOLTA_BIN}/skills
${VOLTA_BIN}/skills: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g skills

.PHONY: chatgpt
chatgpt: brew ${APP_BIN}/ChatGPT.app
${APP_BIN}/ChatGPT.app:
	brew install --cask chatgpt

.PHONY: claude
claude: brew ${APP_BIN}/Claude.app
${APP_BIN}/Claude.app:
	brew install --cask claude

.PHONY: flow
flow: mas /Applications/Flow.app
/Applications/Flow.app:
	@if [ "$(SKIP_PAID_APPS)" = "1" ]; then \
		echo "Skipping Flow installation (SKIP_PAID_APPS=1)"; \
		mkdir -p "$@"; \
	else \
		echo "Installing Flow"; \
		mas install 1423210932 || echo "Warning: Failed to install Flow (may not be purchased on this Apple account)"; \
	fi

.PHONY: frontcli
frontcli: brew ${BREW_BIN}/frontcli
${BREW_BIN}/frontcli:
	brew tap dedene/tap
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --tap dedene/tap; fi
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --formula dedene/tap/frontcli; fi
	brew install dedene/tap/frontcli

.PHONY: language-tool
language-tool: brew ${APP_BIN}/LanguageTool.app
${APP_BIN}/LanguageTool.app:
	brew install --cask languagetool

.PHONY: linear-cli
linear-cli: brew ${BREW_BIN}/linear
${BREW_BIN}/linear:
	brew tap schpet/tap
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --tap schpet/tap; fi
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --formula schpet/tap/linear; fi
	brew install schpet/tap/linear

.PHONY: openspec
openspec: ${VOLTA_BIN}/openspec
${VOLTA_BIN}/openspec: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g @fission-ai/openspec@latest

.PHONY: pi-coding-agent
pi-coding-agent: ${VOLTA_BIN}/pi
${VOLTA_BIN}/pi: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g --ignore-scripts @earendil-works/pi-coding-agent

.PHONY: specsmd
specsmd:
	npx specsmd@latest install

.PHONY: vibe-island
vibe-island: /Applications/Vibe\ Island.app
/Applications/Vibe\ Island.app:
	curl -L https://dl.vibeisland.app/VibeIsland.dmg -o /tmp/VibeIsland.dmg
	hdiutil attach /tmp/VibeIsland.dmg -nobrowse -quiet -mountpoint /tmp/VibeIsland-mount
	cp -R /tmp/VibeIsland-mount/Vibe\ Island.app /Applications/
	hdiutil detach /tmp/VibeIsland-mount -quiet
	rm -f /tmp/VibeIsland.dmg

################################################################################
# End of work section
################################################################################

################################################################################
# Personal section
################################################################################

.PHONY: personal
personal: \
	apple-notes-exporter \
	calibre \
	obsidian \
	perplexity \
	whatsapp

# No Homebrew cask available; the release ships the notes-export-mcp binary
# inside the app bundle, used by the .mcp.json server entry.
.PHONY: apple-notes-exporter
apple-notes-exporter: ${APP_BIN}/Apple\ Notes\ Exporter.app
${APP_BIN}/Apple\ Notes\ Exporter.app:
	curl -L https://github.com/kzaremski/apple-notes-exporter/releases/download/v2.0-2/AppleNotesExporter_v2.0-2.zip -o /tmp/AppleNotesExporter.zip
	unzip -q -o /tmp/AppleNotesExporter.zip -d ${APP_BIN}
	rm -f /tmp/AppleNotesExporter.zip

# Local vault on 1password does not work with 1password
# app from the app store. We need to manually download
# 1password from the website
# 1password: /Applications/1password\ 7.app
# /Applications/1password\ 7.app:
#	brew install 1password

.PHONY: calibre
calibre: brew ${APP_BIN}/Calibre.app
${APP_BIN}/Calibre.app:
	brew install calibre

.PHONY: obsidian
obsidian: brew ${APP_BIN}/Obsidian.app
${APP_BIN}/Obsidian.app:
	brew install --cask obsidian

.PHONY: perplexity
perplexity: mas ${APP_BIN}/Perplexity.app
${APP_BIN}/Perplexity.app:
	mas install 6714467650 || echo "Warning: Failed to install Perplexity (may not be available in this App Store region)"

.PHONY: whatsapp
whatsapp: brew ${APP_BIN}/WhatsApp.app
${APP_BIN}/WhatsApp.app:
	brew install --cask whatsapp

################################################################################
# End of personal section
################################################################################

################################################################################
# Utils section
################################################################################

.PHONY: cleanshot
cleanshot: brew ${APP_BIN}/CleanShot\ X.app
${APP_BIN}/CleanShot\ X.app:
	brew install cleanshot

.PHONY: rectangle-pro
rectangle-pro: brew /Applications/Rectangle\ Pro.app
/Applications/Rectangle\ Pro.app:
	brew install --cask rectangle-pro

.PHONY: handy
handy: brew ${APP_BIN}/Handy.app
${APP_BIN}/Handy.app:
	brew install --cask handy

.PHONY: things-3
things-3: mas /Applications/Things3.app things3-cli-wrapper
/Applications/Things3.app:
	@if [ "$(SKIP_PAID_APPS)" = "1" ]; then \
		echo "Skipping Things 3 installation (SKIP_PAID_APPS=1)"; \
		mkdir -p "$@"; \
	else \
		echo "Installing Things 3"; \
		mas install 904280696 || echo "Warning: Failed to install Things 3 (may not be purchased on this Apple account)"; \
	fi

.PHONY: things3-cli-wrapper
things3-cli-wrapper: ${VOLTA_BIN}/thangs
${VOLTA_BIN}/thangs: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g @dougskinner/thangs

################################################################################
# End of utils section
################################################################################

.PHONY: javascript
javascript: prettier cspell
.PHONY: prettier
prettier: ${VOLTA_BIN}/prettier
${VOLTA_BIN}/prettier: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g prettier @fsouza/prettierd
.PHONY: cspell
cspell: ${VOLTA_BIN}/cspell
${VOLTA_BIN}/cspell: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g cspell

.PHONY: rust
rust: ${BREW_BIN}/cargo
${BREW_BIN}/cargo: | brew
	brew install rust

.PHONY: nvim
nvim: ripgrep brew ${BREW_BIN}/nvim ~/.config/nvim ~/cspell.json
${BREW_BIN}/nvim: | ${VOLTA_BIN}/node
	brew install neovim
	${VOLTA_BIN}/npm install -g neovim
~/.config/nvim: ${DOTFILES_PATH}/nvim | ~/.config
	ln -s ${DOTFILES_PATH}/nvim ~/.config/nvim
~/cspell.json: ${DOTFILES_PATH}/cspell.json
	ln -s ${DOTFILES_PATH}/cspell.json $@

.PHONY: font-jetbrains-mono
font-jetbrains-mono: ~/Library/Fonts/JetBrainsMonoNLNerdFont-Regular.ttf
~/Library/Fonts/JetBrainsMonoNLNerdFont-Regular.ttf:
	brew install font-jetbrains-mono-nerd-font

.PHONY: font-iosevka-nerd-font
font-iosevka-nerd-font: ~/Library/Fonts/IosevkaNerdFont-Regular.ttf
~/Library/Fonts/IosevkaNerdFont-Regular.ttf:
	brew install font-iosevka-nerd-font

.PHONY: fzf
fzf: ~/.fzf
~/.fzf:
	git clone https://github.com/junegunn/fzf.git ~/.fzf
	~/.fzf/install --no-update-rc --no-fish

.PHONY: ripgrep
ripgrep: brew ${BREW_BIN}/rg
${BREW_BIN}/rg:
	brew install ripgrep

.PHONY: zoxide
zoxide: brew ${BREW_BIN}/zoxide
${BREW_BIN}/zoxide:
	brew install zoxide

.PHONY: zsh
zsh: ~/.zshrc
~/.zshrc: ${DOTFILES_PATH}/.zshrc
	ln -s ${DOTFILES_PATH}/.zshrc $@

.PHONY: git-delta
git-delta: brew ${BREW_BIN}/delta ~/.gitconfig.delta
${BREW_BIN}/delta:
	brew install git-delta
~/.gitconfig.delta: ${DOTFILES_PATH}/.gitconfig.delta
	ln -s ${DOTFILES_PATH}/.gitconfig.delta ~/.gitconfig.delta
	@if ! git config --global --get include.path | grep -q "\.gitconfig\.delta"; then \
		git config --global include.path "~/.gitconfig.delta"; \
		echo "Added include.path to ~/.gitconfig"; \
	fi

.PHONY: starship
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

.PHONY: tmux
tmux: brew ${BREW_BIN}/tmux ~/.tmux.conf
${BREW_BIN}/tmux:
	brew install tmux
~/.tmux.conf: ${DOTFILES_PATH}/tmux/.tmux.conf
	ln -s ${DOTFILES_PATH}/tmux/.tmux.conf ~/.tmux.conf

.PHONY: brew
brew: ${BREW_BIN}/brew
${BREW_BIN}/brew:
	curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh > /tmp/brew-installer.sh
	chmod +x /tmp/brew-installer.sh
	/tmp/brew-installer.sh
	brew tap gapple/services
	@if [ "$(HAS_BREW_TRUST)" = "yes" ]; then brew trust --tap gapple/services; fi

.PHONY: daisydisk
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

.PHONY: jscpd
jscpd: ${VOLTA_BIN}/jscpd
${VOLTA_BIN}/jscpd: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g jscpd

.PHONY: mas
mas: brew ${BREW_BIN}/mas
${BREW_BIN}/mas:
	brew install mas

.PHONY: node
node: ${VOLTA_BIN}/node
${VOLTA_BIN}/node: ${BREW_BIN}/volta
	${BREW_BIN}/volta install node@lts
	touch $@

.PHONY: pnpm
pnpm: ${VOLTA_BIN}/pnpm
${VOLTA_BIN}/pnpm: ${VOLTA_BIN}/node
	${BREW_BIN}/volta install pnpm
	touch $@

.PHONY: volta
volta: brew ${BREW_BIN}/volta
${BREW_BIN}/volta:
	brew install volta

.PHONY: mtr
mtr: brew ${BREW_BIN}/../sbin/mtr
${BREW_BIN}/../sbin/mtr:
	brew install mtr

.PHONY: jq
jq: brew ${BREW_BIN}/jq
${BREW_BIN}/jq:
	brew install jq

.PHONY: clean
clean:
	rm -rf ~/.config/nvim
	rm -rf ~/.local/share/nvim
	rm -rf ~/.cache/nvim
