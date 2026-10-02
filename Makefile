BREW_BIN:=$(shell if [ "$(shell uname -p)" = "arm" ]; then echo "/opt/homebrew/bin"; else echo "/usr/local/bin"; fi)
VOLTA_BIN:=$(HOME)/.volta/bin
PNPM_BIN:=$(HOME)/Library/pnpm
LOCAL_BIN:=$(HOME)/.local/bin
APP_BIN:=/Applications
SCRAPLING_IMAGE?=pyd4vinci/scrapling
CLOAKBROWSER_IMAGE?=cloakhq/cloakbrowser:0.5.3
DOCKER_UNAVAILABLE_POLICY?=require-docker
DOTFILES_PATH:=$(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST)))))
DEPLOY_LINK=bun --config=/dev/null --no-env-file "${DOTFILES_PATH}/tooling/deploy-link.ts"
SKIP_PAID_APPS?=0
MOON_EXEC?=moon exec --quiet
export HOMEBREW_NO_ASK:=1
export PATH:=$(PATH):$(HOME)/.moon/bin:$(BREW_BIN)

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
minimal: bootstrap
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:install </dev/null

.PHONY: optional
optional: minimal
	@$(MAKE) --no-print-directory bundle-optional </dev/null
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:anarlog
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:minutes </dev/null
	@$(MAKE) --no-print-directory optional-artifacts </dev/null

.PHONY: smoke-minimal
smoke-minimal:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) tooling:smoke-minimal

.PHONY: bundle-minimal
bundle-minimal:
	@$(MOON_EXEC) applications-install

.PHONY: bundle-optional
bundle-optional:
	@skip_mas=; if [ "$(SKIP_PAID_APPS)" = "1" ]; then skip_mas="411643860 904280696"; fi; HOMEBREW_BUNDLE_MAS_SKIP="$$skip_mas" brew bundle check --quiet --no-upgrade --file "${DOTFILES_PATH}/Brewfile.optional" || { echo "brew bundle --no-upgrade --file ${DOTFILES_PATH}/Brewfile.optional"; HOMEBREW_BUNDLE_MAS_SKIP="$$skip_mas" brew bundle --no-upgrade --file "${DOTFILES_PATH}/Brewfile.optional" </dev/null; }

.PHONY: optional-artifacts
optional-artifacts: cursor cloakbrowser scrapling postgresql daisydisk things-3 lumen

.PHONY: lumen
lumen:
	@bun "${DOTFILES_PATH}/tooling/install-lumen.ts" "${APP_BIN}"

.PHONY: docker
docker:
	@command -v docker >/dev/null || { echo "Error: Docker CLI unavailable" >&2; exit 1; }

.PHONY: postgresql
postgresql: ~/.psqlrc
~/.psqlrc: ${DOTFILES_PATH}/home/.psqlrc FORCE
	@${DEPLOY_LINK} "$<" "$@"

.PHONY: cursor
cursor:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) harness:cursor

.PHONY: scrapling
scrapling: docker ${LOCAL_BIN}/scrapling_mcp
	@"${DOTFILES_PATH}/tooling/install-docker-artifact" install scrapling "${DOCKER_UNAVAILABLE_POLICY}" "${SCRAPLING_IMAGE}"

.PHONY: verify-scrapling-docker
verify-scrapling-docker:
	@"${DOTFILES_PATH}/tooling/install-docker-artifact" verify scrapling "${DOCKER_UNAVAILABLE_POLICY}" "${SCRAPLING_IMAGE}"

# MCP command for agents: starts the shared container on demand instead of one per session.
${LOCAL_BIN}/scrapling_mcp: ${DOTFILES_PATH}/tooling/scrapling-mcp FORCE
	@${DEPLOY_LINK} "$<" "$@"

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

.PHONY: daisydisk
daisydisk:
	@if [ "$(SKIP_PAID_APPS)" != "1" ] && [ ! -d "${APP_BIN}/DaisyDisk.app" ]; then echo "Error: Homebrew Bundle did not install ${APP_BIN}/DaisyDisk.app" >&2; exit 1; fi

.PHONY: moon
moon:
	@set -e; \
	moon_installer=$$(curl -fsSL --connect-timeout 10 --max-time 60 https://moonrepo.dev/install/moon.sh); \
	/bin/bash -c "$$moon_installer"

.PHONY: clean
clean:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:clean

.PHONY: bat
bat:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:bat

.PHONY: fish
fish:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:fish

.PHONY: nvim
nvim:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:nvim

.PHONY: wezterm
wezterm:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:wezterm

.PHONY: git-delta
git-delta:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:git-delta

.PHONY: starship
starship:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:starship

.PHONY: tmux
tmux:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:tmux

.PHONY: arnes
arnes:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) arnes:install

.PHONY: agent-memory
agent-memory:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) agent-memory:install

.PHONY: agent-handoff
agent-handoff:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) agent-handoff:install

.PHONY: claude-code
claude-code:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) harness:claude

.PHONY: codex
codex:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) harness:codex

.PHONY: hunspell
hunspell:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:hunspell-dictionaries

.PHONY: hunspell-dictionaries
hunspell-dictionaries:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:hunspell-dictionaries

.PHONY: brew
brew:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:homebrew

.PHONY: node
node:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:node

.PHONY: volta
volta:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:volta

.PHONY: pnpm
pnpm:
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:pnpm

~/.arnes.yaml: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:arnes-config

~/.claude/CLAUDE.md ~/.claude/SOUL.md ~/.claude/USER.md: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) harness:claude-instructions

~/.claude/rules/agent-instructions.md: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) harness:claude-rules

$(HOME)/.claude/skills/%: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) harness:claude-skills

~/.codex/AGENTS.md: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) harness:codex-instructions

~/.codex/agents/design-claim-auditor.toml: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) harness:codex-agents

$(HOME)/.agents/skills/%: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) harness:codex-skills

$(HOME)/.cursor/skills/%: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) harness:cursor-skills

${LOCAL_BIN}/arnes: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) arnes:binary

${LOCAL_BIN}/agent-memory: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) agent-memory:binary

${LOCAL_BIN}/agent-handoff: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) agent-handoff:binary

${VOLTA_BIN}/node: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:node

${VOLTA_BIN}/pnpm: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:pnpm

${BREW_BIN}/volta: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:volta

${HOME}/.cargo/bin/cargo: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) repository:rust

~/cspell.json ~/.config/cspell/user.txt: FORCE
	@cd "${DOTFILES_PATH}" && $(MOON_EXEC) home:cspell-config
