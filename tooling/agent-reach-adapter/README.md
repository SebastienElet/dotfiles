# @sebastienelet/agent-reach-adapter

Local release candidate for 1.0.0, not published or qualified from npm yet.
Complete integration targets macOS ARM64 only. Bun 1.4.3 runs the TypeScript
entry point directly; no build, Moon, Volta or dotfiles checkout is required.

After publication, acquire the exact version in a durable, dedicated directory:

```sh
bun install --exact @sebastienelet/agent-reach-adapter@1.0.0
bun run agent-reach-adapter plan
bun run agent-reach-adapter prepare "$HOME"
bun run agent-reach-adapter doctor
```

Acquisition runs no lifecycle installer. `plan` reports the pinned upstream
revision and Python/Node prerequisites. `prepare HOME` requires Git and fetches
that exact revision, verifies its files, then assembles the marketplace under
`HOME/.local/share/agent-reach/adapter/1.0.0/marketplace`. It preserves an
identical replay and refuses divergent files, directories or symlinks. It does
not replace the historical marketplace or activate a host. Partial acquisition
can leave a verified upstream cache, but not a partial marketplace.

Python 3.13, uv, the Python tools reported by `plan`, ffmpeg, Node, Docker,
GitHub CLI, and the selected agent are external prerequisites. Install Python
and its tools separately with native tooling, explicitly requested by the user;
Bun never installs them. Preserve the historical isolated interpreter at
`~/.local/share/agent-reach/python-tools/agent-reach/bin/python`, which the
packaged RSS and podcast commands use. The upstream runtime must use the full
Git revision in `resources/source.json`. Install the exact Node tools reported
by `plan` with Bun in a durable tool directory and expose their binaries on PATH.
Claude and Codex stay on their latest releases, without a pin.

`doctor` runs version probes with a ten second timeout per command and checks
that declared platform executables and the historical isolated interpreter exist.
Presence does not establish their versions, imports or platform behavior.
It reports the first missing, failed or empty probe with a nonzero exit. It
does not inspect browser sessions, invoke upstream `agent-reach doctor`, test
credentials, contact backends, or establish plugin activation.

Personal mcporter configuration, credentials, cookies, browser sessions and
Docker configuration are supplied by the workstation owner. The existing
LinkedIn backend uses
`stickerdaniel/linkedin-mcp-server:4.26.2@sha256:c7aa139cfaaecc8ae9f5e810d6517e5e36b5a1a52cadcb4eef6f0832810b4da6`.
Keep the image digest and existing profile volume when configuring Docker.
Pulling, starting and logging in to that service are separate explicit native
Docker operations; preparation never performs them.

Inspect each host's existing marketplace/plugin registrations before activation.
Refuse an unknown homonym, preserve unrelated entries, and do not register both
copies of `agent-reach`. In a clean host configuration, the native commands are:

```sh
claude plugin marketplace add "$HOME/.local/share/agent-reach/adapter/1.0.0/marketplace"
claude plugin install agent-reach@agent-reach-adapter --scope user
codex plugin marketplace add "$HOME/.local/share/agent-reach/adapter/1.0.0/marketplace"
codex plugin add agent-reach@agent-reach-adapter --json
```

For the historical `agent-reach@dotfiles-agent-reach`, record the previous path,
version and registrations first. Disable/remove only the known old registration
using that host's native CLI before activating the new one. For rollback,
remove the new registration and add/install the recorded previous marketplace
and plugin using native commands. Preserve the old marketplace until discovery,
activation and rollback have been observed. Never purge profiles or overwrite
shared configuration to complete a switch; there is no transaction across hosts.

Generic web reading remains anonymous Jina Reader access for public,
non-sensitive URLs only. Private/authenticated reading and browser fallback
are unavailable. The bundled skill records the budgets and content checks;
HTTP success alone does not establish retrieval.

The adapter retains the dotfiles BSD-2-Clause license. The exact upstream
Panniantong/Agent-Reach MIT license is copied into the prepared plugin alongside
its adapted references and scripts; `ADAPTER-LICENSE` retains the adapter notice.
`resources/source.json` records provenance.

Development uses `bun install --frozen-lockfile`, `bun test`, `bun run typecheck`,
`bun run lint` and `bun run format:check`. Tests and resources ship with the
package. A local tarball smoke is preparation evidence only: registry
installation, provenance, real Claude/Codex discovery, activation and rollback
remain release acceptance work. Linux and other architectures are not qualified.

The prepared manual `publish.yml` workflow belongs at the future repository
root. It uses Bun for dependencies/checks and npm only for CI publication with
OIDC and provenance. Before running it, create the public repository and configure
the npm trusted publisher for that exact repository and `publish.yml`, allowing
`npm publish`. Those external operations require the maintainer's approval.
See [npm trusted publishing](https://docs.npmjs.com/trusted-publishers/).
