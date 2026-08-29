BREW_BIN:=$(shell if [ "$(shell uname -p)" = "arm" ]; then echo "/opt/homebrew/bin"; else echo "/usr/local/bin"; fi)
VOLTA_BIN:=$(HOME)/.volta/bin
PNPM_BIN:=$(HOME)/Library/pnpm
LOCAL_BIN:=$(HOME)/.local/bin
APP_BIN:=/Applications
MINIMAL_SNAPSHOT_PATHS:=.agents/skills .arnes.yaml .claude .claude.json .codex/AGENTS.md .codex/agents .codex/config.toml .config/bat .config/cspell .config/fish .config/git/config.delta .config/git/ignore .config/nvim .config/starship.toml .config/tmux .config/wezterm .gitconfig .local/bin/agent-handoff .local/bin/arnes .local/bin/claude .local/bin/colgrep-search .tmux/plugins/tpm .volta/bin/codex .volta/bin/node .volta/bin/pnpm Library/Spelling cspell.json
SCRAPLING_IMAGE?=pyd4vinci/scrapling
CLOAKBROWSER_IMAGE?=cloakhq/cloakbrowser:0.5.3
DOCKER_UNAVAILABLE_POLICY?=require-docker
DOTFILES_PATH:=$(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST)))))
CREATE_SYMLINK=if [ -L "$@" ] && [ "$$(readlink "$@")" = "$<" ]; then exit 0; fi; if [ -e "$@" ] || [ -L "$@" ]; then echo "Error: $@ exists and is not the expected symbolic link" >&2; exit 1; fi; echo "ln -s $< $@"; ln -s "$<" "$@"
SKIP_PAID_APPS?=0
export HOMEBREW_NO_ASK:=1

.PHONY: usage
usage:
	@echo minimal - Install the development baseline
	@echo optional - Install the optional profile

.PHONY: FORCE
FORCE:

.PHONY: bootstrap
bootstrap:
	@command -v curl >/dev/null || { echo "Error: curl is required" >&2; exit 1; }
	@command -v git >/dev/null || { echo "Error: Git is required" >&2; exit 1; }
	@xcode-select --print-path >/dev/null || { echo "Error: Apple Command Line Tools are required" >&2; exit 1; }

.PHONY: minimal
minimal: bootstrap brew
	@$(MAKE) --no-print-directory bundle-minimal </dev/null
	@$(MAKE) --no-print-directory minimal-artifacts </dev/null

.PHONY: optional
optional: bootstrap
	@$(MAKE) --no-print-directory brew
	@$(MAKE) --no-print-directory minimal </dev/null
	@$(MAKE) --no-print-directory bundle-optional </dev/null
	@$(MAKE) --no-print-directory optional-artifacts </dev/null

.PHONY: smoke-minimal
smoke-minimal:
	@set -eu; \
	$(MAKE) --no-print-directory minimal </dev/null; \
	brew bundle check --quiet --no-upgrade --file "${DOTFILES_PATH}/Brewfile"; \
	for executable in "${BREW_BIN}/colgrep" "${LOCAL_BIN}/agent-handoff" "${LOCAL_BIN}/arnes" "${LOCAL_BIN}/claude" "${LOCAL_BIN}/colgrep-search" "${VOLTA_BIN}/codex" "${VOLTA_BIN}/node" "${VOLTA_BIN}/pnpm"; do test -x "$$executable"; done; \
	stdout=$$(mktemp); stderr=$$(mktemp); trap 'rm -f "$$stdout" "$$stderr"' EXIT; \
	before=$$(cd "$(HOME)" && tar -cf - ${MINIMAL_SNAPSHOT_PATHS} | shasum -a 256); \
	if ! $(MAKE) --no-print-directory minimal </dev/null >"$$stdout" 2>"$$stderr"; then cat "$$stdout"; cat "$$stderr" >&2; exit 1; fi; \
	test ! -s "$$stdout" || { cat "$$stdout"; exit 1; }; \
	test ! -s "$$stderr" || { cat "$$stderr" >&2; exit 1; }; \
	after=$$(cd "$(HOME)" && tar -cf - ${MINIMAL_SNAPSHOT_PATHS} | shasum -a 256); \
	test "$$before" = "$$after"; \
	brew bundle check --quiet --no-upgrade --file "${DOTFILES_PATH}/Brewfile"

.PHONY: bundle-minimal
bundle-minimal:
	@brew bundle check --quiet --no-upgrade --file "${DOTFILES_PATH}/Brewfile" || { echo "brew bundle --no-upgrade --file ${DOTFILES_PATH}/Brewfile"; brew bundle --no-upgrade --file "${DOTFILES_PATH}/Brewfile" </dev/null; }

.PHONY: bundle-optional
bundle-optional:
	@skip_mas=; if [ "$(SKIP_PAID_APPS)" = "1" ]; then skip_mas="411643860 904280696"; fi; HOMEBREW_BUNDLE_MAS_SKIP="$$skip_mas" brew bundle check --quiet --no-upgrade --file "${DOTFILES_PATH}/Brewfile.optional" || { echo "brew bundle --no-upgrade --file ${DOTFILES_PATH}/Brewfile.optional"; HOMEBREW_BUNDLE_MAS_SKIP="$$skip_mas" brew bundle --no-upgrade --file "${DOTFILES_PATH}/Brewfile.optional" </dev/null; }

.PHONY: minimal-artifacts
minimal-artifacts: bat fish nvim wezterm git-delta starship tmux node pnpm arnes claude-code codex hunspell ${LOCAL_BIN}/colgrep-search

.PHONY: optional-artifacts
optional-artifacts: cspell cursor cloakbrowser scrapling postgresql daisydisk things-3

~/.config:
	mkdir -p $@

.PHONY: bat
bat: ~/.config/bat/themes/Catppuccin\ Latte.tmTheme
~/.config/bat/themes:
	mkdir -p $@
~/.config/bat/themes/Catppuccin\ Latte.tmTheme: | ~/.config/bat/themes
	curl -L -o ~/.config/bat/themes/Catppuccin\ Latte.tmTheme https://github.com/catppuccin/bat/raw/main/themes/Catppuccin%20Latte.tmTheme
	curl -L -o ~/.config/bat/themes/Catppuccin\ Mocha.tmTheme https://github.com/catppuccin/bat/raw/main/themes/Catppuccin%20Mocha.tmTheme
	bat cache --build

.PHONY: fish
fish: starship ~/.config/fish ~/.config/fish/functions/fzf_configure_bindings.fish

~/.config/fish: ${DOTFILES_PATH}/home/.config/fish FORCE | ~/.config
	@${CREATE_SYMLINK}
~/.config/fish/functions/fzf_configure_bindings.fish: FORCE ${BREW_BIN}/fish | ~/.config/fish
	@if [ ! -e "$@" ]; then \
		if ! ${BREW_BIN}/fish -c 'fisher install PatrickF1/fzf.fish' || [ ! -e "$@" ]; then \
			echo "Error: Fisher did not install $@" >&2; \
			exit 1; \
		fi; \
	fi

.PHONY: wezterm
wezterm: ~/.config/wezterm/wezterm.lua
~/.config/wezterm:
	mkdir -p $@
~/.config/wezterm/wezterm.lua: ${DOTFILES_PATH}/home/.config/wezterm/wezterm.lua FORCE | ~/.config/wezterm
	@${CREATE_SYMLINK}

~/.arnes.yaml: ${DOTFILES_PATH}/home/.arnes.yaml FORCE
	@${CREATE_SYMLINK}

.PHONY: arnes
arnes: ~/.arnes.yaml ${LOCAL_BIN}/arnes

${LOCAL_BIN}/arnes: ${DOTFILES_PATH}/tooling/arnes/Cargo.toml FORCE | ${LOCAL_BIN}
	@cd ${DOTFILES_PATH}/tooling/arnes && ${BREW_BIN}/cargo build --quiet --release
	@source_path="${DOTFILES_PATH}/tooling/arnes/target/release/arnes"; if [ -L "$@" ] && [ "$$(readlink "$@")" = "$$source_path" ]; then exit 0; fi; if [ -e "$@" ] || [ -L "$@" ]; then echo "Error: $@ exists and is not the expected symbolic link" >&2; exit 1; fi; echo "ln -s $$source_path $@"; ln -s "$$source_path" "$@"

${LOCAL_BIN}/agent-handoff: ${DOTFILES_PATH}/tooling/agent-handoff FORCE | ${LOCAL_BIN}
	@${CREATE_SYMLINK}

.PHONY: docker
docker:
	@command -v docker >/dev/null || { echo "Error: Docker CLI unavailable" >&2; exit 1; }

.PHONY: postgresql
postgresql: ~/.psqlrc
~/.psqlrc: ${DOTFILES_PATH}/home/.psqlrc FORCE
	@${CREATE_SYMLINK}

.PHONY: cursor
cursor: ~/.cursor/skills/claude-developer ~/.cursor/skills/code-search ~/.cursor/skills/enforcement-code ~/.cursor/skills/harness-reflection ~/.cursor/skills/issue-creation ~/.cursor/skills/linear-issue-spec ~/.cursor/skills/linear-start ~/.cursor/skills/linear-sync ~/.cursor/skills/linear-workflow ~/.cursor/skills/obsidian-retrieval ~/.cursor/skills/pr-fix ~/.cursor/skills/pr-feedback ~/.cursor/skills/pr-verdict ~/.cursor/skills/requirements-clarification ~/.cursor/skills/skill-manager ~/.cursor/skills/workflow-automation cursor-hooks
~/.cursor/skills:
	mkdir -p $@
~/.cursor/skills/claude-developer: ${DOTFILES_PATH}/harness/skills/claude-developer FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/code-search: ${DOTFILES_PATH}/harness/skills/code-search FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/enforcement-code: ${DOTFILES_PATH}/harness/skills/enforcement-code FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/issue-creation: ${DOTFILES_PATH}/harness/skills/issue-creation FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/harness-reflection: ${DOTFILES_PATH}/harness/skills/harness-reflection FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/linear-issue-spec: ${DOTFILES_PATH}/harness/skills/linear-issue-spec FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/linear-start: ${DOTFILES_PATH}/harness/skills/linear-start FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/linear-sync: ${DOTFILES_PATH}/harness/skills/linear-sync FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/linear-workflow: ${DOTFILES_PATH}/harness/skills/linear-workflow FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/obsidian-retrieval: ${DOTFILES_PATH}/harness/skills/obsidian-retrieval FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/pr-fix: ${DOTFILES_PATH}/harness/skills/pr-fix FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/pr-feedback: ${DOTFILES_PATH}/harness/skills/pr-feedback FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/pr-verdict: ${DOTFILES_PATH}/harness/skills/pr-verdict FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/requirements-clarification: ${DOTFILES_PATH}/harness/skills/requirements-clarification FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/skill-manager: ${DOTFILES_PATH}/harness/skills/skill-manager FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}
~/.cursor/skills/workflow-automation: ${DOTFILES_PATH}/harness/skills/workflow-automation FORCE | ~/.cursor/skills
	@${CREATE_SYMLINK}

.PHONY: cursor-hooks
cursor-hooks: arnes
	@"${LOCAL_BIN}/arnes" doctor hooks --agent cursor --color never >/dev/null 2>&1 || "${LOCAL_BIN}/arnes" setup hooks --agent cursor

.PHONY: claude-code
claude-code: hunspell ${LOCAL_BIN}/claude ~/.claude/CLAUDE.md ~/.claude/SOUL.md ~/.claude/USER.md ~/.claude/rules/agent-instructions.md ~/.claude/skills/code-search ~/.claude/skills/handoff ~/.claude/skills/enforcement-code ~/.claude/skills/harness-reflection ~/.claude/skills/issue-creation ~/.claude/skills/linear-issue-spec ~/.claude/skills/linear-start ~/.claude/skills/linear-sync ~/.claude/skills/linear-workflow ~/.claude/skills/obsidian-retrieval ~/.claude/skills/pr-fix ~/.claude/skills/pr-feedback ~/.claude/skills/pr-verdict ~/.claude/skills/requirements-clarification ~/.claude/skills/skill-manager ~/.claude/skills/workflow-automation claude-code-hooks
${LOCAL_BIN}/claude:
	curl -fsSL https://claude.ai/install.sh | bash -s latest
~/.claude:
	mkdir -p $@
~/.claude/CLAUDE.md: ${DOTFILES_PATH}/harness/AGENTS.md FORCE | ~/.claude
	@${CREATE_SYMLINK}
# Imported by AGENTS.md; linked as siblings so the @import resolves whether the
# tool follows the symlink or reads it from the destination directory.
~/.claude/SOUL.md: ${DOTFILES_PATH}/harness/SOUL.md FORCE | ~/.claude
	@${CREATE_SYMLINK}
~/.claude/USER.md: ${DOTFILES_PATH}/harness/USER.md FORCE | ~/.claude
	@${CREATE_SYMLINK}
~/.claude/rules ~/.claude/skills: | ~/.claude
	mkdir -p $@
~/.claude/rules/agent-instructions.md: ${DOTFILES_PATH}/harness/rules/agent-instructions.md FORCE | ~/.claude/rules
	@${CREATE_SYMLINK}
~/.claude/skills/code-search: ${DOTFILES_PATH}/harness/skills/code-search FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/handoff: ${DOTFILES_PATH}/harness/skills/handoff FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/enforcement-code: ${DOTFILES_PATH}/harness/skills/enforcement-code FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/issue-creation: ${DOTFILES_PATH}/harness/skills/issue-creation FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/harness-reflection: ${DOTFILES_PATH}/harness/skills/harness-reflection FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/linear-issue-spec: ${DOTFILES_PATH}/harness/skills/linear-issue-spec FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/linear-start: ${DOTFILES_PATH}/harness/skills/linear-start FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/linear-sync: ${DOTFILES_PATH}/harness/skills/linear-sync FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/linear-workflow: ${DOTFILES_PATH}/harness/skills/linear-workflow FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/obsidian-retrieval: ${DOTFILES_PATH}/harness/skills/obsidian-retrieval FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
# Linked globally because a pull request is reviewed from the repository under
# review, which is never this one.
~/.claude/skills/pr-fix: ${DOTFILES_PATH}/harness/skills/pr-fix FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/pr-feedback: ${DOTFILES_PATH}/harness/skills/pr-feedback FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/pr-verdict: ${DOTFILES_PATH}/harness/skills/pr-verdict FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/requirements-clarification: ${DOTFILES_PATH}/harness/skills/requirements-clarification FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/skill-manager: ${DOTFILES_PATH}/harness/skills/skill-manager FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}
~/.claude/skills/workflow-automation: ${DOTFILES_PATH}/harness/skills/workflow-automation FORCE | ~/.claude/skills
	@${CREATE_SYMLINK}

.PHONY: claude-code-hooks
claude-code-hooks: arnes ${LOCAL_BIN}/agent-handoff
	@"${LOCAL_BIN}/arnes" doctor hooks --agent claude --color never >/dev/null 2>&1 || "${LOCAL_BIN}/arnes" setup hooks --agent claude

.PHONY: hunspell
hunspell: hunspell-dictionaries

.PHONY: hunspell-dictionaries
hunspell-dictionaries:
	@"${DOTFILES_PATH}/tooling/install-hunspell-dictionary" "https://raw.githubusercontent.com/LibreOffice/dictionaries/f2ff99058268502bdcf4cad25c1ca2935ad8aa7d/fr_FR/dictionaries/fr.aff" "c176610cd5dc4846806a65ddd029f422d87978bf58f224aa44222662a16a2de5" "$(HOME)/Library/Spelling/fr.aff"
	@"${DOTFILES_PATH}/tooling/install-hunspell-dictionary" "https://raw.githubusercontent.com/LibreOffice/dictionaries/f2ff99058268502bdcf4cad25c1ca2935ad8aa7d/fr_FR/dictionaries/fr.dic" "b78a868e31dd6e373b6c3217969afb898a9acde828a5e7ef97308da42218c88c" "$(HOME)/Library/Spelling/fr.dic"
	@"${DOTFILES_PATH}/tooling/install-hunspell-dictionary" "https://raw.githubusercontent.com/LibreOffice/dictionaries/f2ff99058268502bdcf4cad25c1ca2935ad8aa7d/en/en_US.aff" "e746c882dd6f303c2c46e7452804b9201115a6942cfeb15f18f8edf774d2e24e" "$(HOME)/Library/Spelling/en_US.aff"
	@"${DOTFILES_PATH}/tooling/install-hunspell-dictionary" "https://raw.githubusercontent.com/LibreOffice/dictionaries/f2ff99058268502bdcf4cad25c1ca2935ad8aa7d/en/en_US.dic" "f0b1a234bd178bdd01875b2a392a9647f888b8fe879f79c52aae62c2759b3647" "$(HOME)/Library/Spelling/en_US.dic"

.PHONY: codex
codex: ${VOLTA_BIN}/codex ~/.codex/AGENTS.md ~/.codex/agents/design-claim-auditor.toml ~/.agents/skills/agent-instructions ~/.agents/skills/claude-developer ~/.agents/skills/code-search ~/.agents/skills/design-claim-audit ~/.agents/skills/handoff ~/.agents/skills/enforcement-code ~/.agents/skills/harness-reflection ~/.agents/skills/issue-creation ~/.agents/skills/linear-issue-spec ~/.agents/skills/linear-start ~/.agents/skills/linear-sync ~/.agents/skills/linear-workflow ~/.agents/skills/obsidian-retrieval ~/.agents/skills/pr-fix ~/.agents/skills/pr-feedback ~/.agents/skills/pr-verdict ~/.agents/skills/requirements-clarification ~/.agents/skills/skill-manager ~/.agents/skills/workflow-automation codex-hooks
${VOLTA_BIN}/codex: ${VOLTA_BIN}/node
	${BREW_BIN}/volta install @openai/codex
~/.codex:
	mkdir -p $@
# Codex ignores AGENTS.md @import directives, so the sources are assembled
# here instead of symlinked. Written to a temporary path then moved, so an
# existing symlink is replaced rather than written through.
~/.codex/AGENTS.md: ${DOTFILES_PATH}/harness/AGENTS.md ${DOTFILES_PATH}/harness/SOUL.md ${DOTFILES_PATH}/harness/USER.md FORCE | ~/.codex
	@expected="$@.expected.$$$$"; trap 'rm -f "$$expected"' EXIT; grep -v '^@' "$<" | cat - "${DOTFILES_PATH}/harness/SOUL.md" "${DOTFILES_PATH}/harness/USER.md" > "$$expected"; if [ -f "$@" ] && [ ! -L "$@" ] && cmp -s "$$expected" "$@"; then exit 0; fi; if [ -e "$@" ] || [ -L "$@" ]; then echo "Error: $@ exists and does not contain the expected instructions" >&2; exit 1; fi; echo "mv $$expected $@"; mv "$$expected" "$@"; trap - EXIT
~/.codex/agents:
	mkdir -p $@
~/.codex/agents/design-claim-auditor.toml: ${DOTFILES_PATH}/home/.codex/agents/design-claim-auditor.toml FORCE | ~/.codex/agents
	@${CREATE_SYMLINK}
~/.agents/skills:
	mkdir -p $@
~/.agents/skills/agent-instructions: ${DOTFILES_PATH}/harness/skills/agent-instructions FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/claude-developer: ${DOTFILES_PATH}/harness/skills/claude-developer FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/code-search: ${DOTFILES_PATH}/harness/skills/code-search FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/design-claim-audit: ${DOTFILES_PATH}/harness/skills/design-claim-audit FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/enforcement-code: ${DOTFILES_PATH}/harness/skills/enforcement-code FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/handoff: ${DOTFILES_PATH}/harness/skills/handoff FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/issue-creation: ${DOTFILES_PATH}/harness/skills/issue-creation FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/harness-reflection: ${DOTFILES_PATH}/harness/skills/harness-reflection FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/linear-issue-spec: ${DOTFILES_PATH}/harness/skills/linear-issue-spec FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/linear-start: ${DOTFILES_PATH}/harness/skills/linear-start FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/linear-sync: ${DOTFILES_PATH}/harness/skills/linear-sync FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/linear-workflow: ${DOTFILES_PATH}/harness/skills/linear-workflow FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/obsidian-retrieval: ${DOTFILES_PATH}/harness/skills/obsidian-retrieval FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/pr-fix: ${DOTFILES_PATH}/harness/skills/pr-fix FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/pr-feedback: ${DOTFILES_PATH}/harness/skills/pr-feedback FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/pr-verdict: ${DOTFILES_PATH}/harness/skills/pr-verdict FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/requirements-clarification: ${DOTFILES_PATH}/harness/skills/requirements-clarification FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/skill-manager: ${DOTFILES_PATH}/harness/skills/skill-manager FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}
~/.agents/skills/workflow-automation: ${DOTFILES_PATH}/harness/skills/workflow-automation FORCE | ~/.agents/skills
	@${CREATE_SYMLINK}

.PHONY: codex-hooks
codex-hooks: arnes ${LOCAL_BIN}/agent-handoff
	@"${LOCAL_BIN}/arnes" doctor hooks --agent codex --color never >/dev/null 2>&1 || "${LOCAL_BIN}/arnes" setup hooks --agent codex

.PHONY: obsidian-retrieval-test
obsidian-retrieval-test: ${BREW_BIN}/bun
	cd "${DOTFILES_PATH}" && "${BREW_BIN}/bun" ci
	cd "${DOTFILES_PATH}" && "${BREW_BIN}/bun" run typecheck
	cd "${DOTFILES_PATH}" && "${BREW_BIN}/bun" test tooling/obsidian-retrieval/contract.test.ts

${LOCAL_BIN}/colgrep-search: ${DOTFILES_PATH}/tooling/colgrep-search-cli.ts FORCE | ${LOCAL_BIN}
	@${CREATE_SYMLINK}

.PHONY: scrapling
scrapling: docker ${LOCAL_BIN}/scrapling_mcp
	@"${DOTFILES_PATH}/tooling/install-docker-artifact" install scrapling "${DOCKER_UNAVAILABLE_POLICY}" "${SCRAPLING_IMAGE}"

.PHONY: verify-scrapling-docker
verify-scrapling-docker:
	@"${DOTFILES_PATH}/tooling/install-docker-artifact" verify scrapling "${DOCKER_UNAVAILABLE_POLICY}" "${SCRAPLING_IMAGE}"

# MCP command for agents: starts the shared container on demand instead of one per session.
${LOCAL_BIN}/scrapling_mcp: ${DOTFILES_PATH}/tooling/scrapling-mcp FORCE | ${LOCAL_BIN}
	@${CREATE_SYMLINK}
${LOCAL_BIN}:
	mkdir -p $@

.PHONY: cloakbrowser
cloakbrowser: docker
	@"${DOTFILES_PATH}/tooling/install-docker-artifact" install cloakbrowser "${DOCKER_UNAVAILABLE_POLICY}" "${CLOAKBROWSER_IMAGE}"

.PHONY: verify-cloakbrowser-docker
verify-cloakbrowser-docker:
	@"${DOTFILES_PATH}/tooling/install-docker-artifact" verify cloakbrowser "${DOCKER_UNAVAILABLE_POLICY}" "${CLOAKBROWSER_IMAGE}"

# No Homebrew cask available; the release ships the notes-export-mcp binary
# inside the app bundle, used by the .mcp.json server entry.
.PHONY: apple-notes-exporter
apple-notes-exporter: ${APP_BIN}/Apple\ Notes\ Exporter.app
${APP_BIN}/Apple\ Notes\ Exporter.app:
	curl -L https://github.com/kzaremski/apple-notes-exporter/releases/download/v2.0-2/AppleNotesExporter_v2.0-2.zip -o /tmp/AppleNotesExporter.zip
	unzip -q -o /tmp/AppleNotesExporter.zip -d ${APP_BIN}
	rm -f /tmp/AppleNotesExporter.zip

.PHONY: things-3
things-3:
	@if [ "$(SKIP_PAID_APPS)" = "1" ]; then exit 0; fi; $(MAKE) --silent things3-cli-wrapper; if [ ! -d "${APP_BIN}/Things3.app" ]; then echo "Error: Homebrew Bundle did not install ${APP_BIN}/Things3.app" >&2; exit 1; fi

.PHONY: things3-cli-wrapper
things3-cli-wrapper: ${VOLTA_BIN}/thangs
${VOLTA_BIN}/thangs: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g @dougskinner/thangs

.PHONY: cspell
cspell: ${VOLTA_BIN}/cspell
${VOLTA_BIN}/cspell: ${VOLTA_BIN}/node
	${VOLTA_BIN}/npm install -g cspell

.PHONY: nvim
nvim: ~/.config/nvim ~/cspell.json ~/.config/cspell/user.txt
~/.config/nvim: ${DOTFILES_PATH}/home/.config/nvim FORCE | ~/.config
	@${CREATE_SYMLINK}
~/cspell.json: ${DOTFILES_PATH}/home/cspell.json FORCE
	@${CREATE_SYMLINK}
~/.config/cspell:
	mkdir -p $@
~/.config/cspell/user.txt: ${DOTFILES_PATH}/home/.config/cspell/user.txt FORCE | ~/.config/cspell
	@${CREATE_SYMLINK}

.PHONY: git-delta
git-delta: ~/.config/git/config.delta
	@includes=$$(git config --global --get-all include.path || test $$? -eq 1) || exit; \
	if ! printf '%s\n' "$$includes" | grep -Fxq '~/.config/git/config.delta'; then \
		git config --global --add include.path '~/.config/git/config.delta' || exit; \
		echo "Added include.path to Git's global configuration"; \
	fi; \
	if printf '%s\n' "$$includes" | grep -Fxq '~/.gitconfig.delta'; then \
		git config --global --unset-all include.path '^~/[.]gitconfig[.]delta$$' || exit; \
	fi
~/.config/git:
	mkdir -p $@
~/.config/git/config.delta: ${DOTFILES_PATH}/home/.config/git/config.delta FORCE | ~/.config/git
	@${CREATE_SYMLINK}

.PHONY: starship
starship: ~/.config/starship.toml
~/.config/starship.toml: ${DOTFILES_PATH}/home/.config/starship.toml FORCE | ~/.config
	@${CREATE_SYMLINK}

.PHONY: tmux
tmux: ~/.config/tmux/tmux.conf ~/.tmux/plugins/tpm/tpm
~/.config/tmux:
	mkdir -p $@
~/.config/tmux/tmux.conf: ${DOTFILES_PATH}/home/.config/tmux/tmux.conf FORCE | ~/.config/tmux
	@${CREATE_SYMLINK}
~/.tmux/plugins:
	mkdir -p $@
~/.tmux/plugins/tpm/tpm: | ~/.tmux/plugins
	git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm

.PHONY: brew
brew: ${BREW_BIN}/brew
${BREW_BIN}/brew:
	curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh > /tmp/brew-installer.sh
	chmod +x /tmp/brew-installer.sh
	/tmp/brew-installer.sh

.PHONY: daisydisk
daisydisk:
	@if [ "$(SKIP_PAID_APPS)" != "1" ] && [ ! -d "${APP_BIN}/DaisyDisk.app" ]; then echo "Error: Homebrew Bundle did not install ${APP_BIN}/DaisyDisk.app" >&2; exit 1; fi

.PHONY: node
node: ${VOLTA_BIN}/node
${VOLTA_BIN}/node: ${BREW_BIN}/volta ${DOTFILES_PATH}/package.json ${DOTFILES_PATH}/tooling/node-version-contract.ts | ${BREW_BIN}/bun
	node_install_spec=$$(${BREW_BIN}/bun --no-install ${DOTFILES_PATH}/tooling/node-version-contract.ts install-spec) && \
		${BREW_BIN}/volta install "$$node_install_spec"
	touch $@

.PHONY: pnpm
pnpm: ${VOLTA_BIN}/pnpm
${VOLTA_BIN}/pnpm: ${VOLTA_BIN}/node
	${BREW_BIN}/volta install pnpm
	touch $@

.PHONY: clean
clean:
	rm -rf ~/.config/nvim
	rm -rf ~/.local/share/nvim
	rm -rf ~/.cache/nvim
