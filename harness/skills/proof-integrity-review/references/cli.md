# Standalone CLI

The skill contains its Rust package in `scripts/`. Git and a Rust toolchain supporting the
package's edition are required. No Python, Arnes, Moon, or dotfiles checkout is required by the
CLI itself. Moon is only this repository's build and CI integration.

## Build and locate

Resolve the directory containing this skill's `SKILL.md`, including through an installation
symlink, then set `PROOF_SKILL_ROOT` to that absolute path. Set `PROOF_REPOSITORY` to the repository
under review. Build for the current host:

```sh
cargo build --release --locked --manifest-path "$PROOF_SKILL_ROOT/scripts/Cargo.toml" --target-dir "$PROOF_SKILL_ROOT/scripts/target"
PROOF_BIN="$PROOF_SKILL_ROOT/scripts/target/release/proof-integrity"
```

Sources and lockfile are versioned; generated `target/` output is ignored. Copy the complete skill
and build again on another host. A macOS binary is not a Linux binary; a binary for one CPU is not
implicitly compatible with another. The repository CI targets macOS and Linux; do not infer
Windows support from Rust alone. Use the trusted build when auditing a candidate policy change.

## Commands

Replace the sample `main` and `HEAD` with the actual review base and candidate. Keep output files
outside both repository and skill roots so writing evidence cannot change the snapshot it records.

```sh
"$PROOF_BIN" classify --repository "$PROOF_REPOSITORY" --base main --head HEAD
"$PROOF_BIN" classify --repository "$PROOF_REPOSITORY" --base main --worktree
"$PROOF_BIN" classify --paths .moon/tasks/checks.yml harness/skills/example/SKILL.md

"$PROOF_BIN" epoch --repository "$PROOF_REPOSITORY" --base main --head HEAD --policy-root "$PROOF_SKILL_ROOT" --output /tmp/proof-epoch.json
"$PROOF_BIN" epoch --repository "$PROOF_REPOSITORY" --base main --include-worktree --policy-root "$PROOF_SKILL_ROOT" --output /tmp/proof-worktree-epoch.json

"$PROOF_BIN" gate --epoch /tmp/proof-epoch.json --receipt /tmp/proof-receipt.json --policy-root "$PROOF_SKILL_ROOT"
```

Use unique artifact paths for concurrent reviews. `epoch` creates the machine-readable record;
never hand-author one. `gate` checks the supplied record against current repository and policy
inputs. A rejected gate returns a nonzero exit; retain its JSON diagnostic, not just its status.
An `ALLOW` result validates the receipt's declared evidence against the implemented policy; it is
not a statement that the product is correct or that a merge is authorized.

## Receipt contract

The Rust port uses schema version 4 and predicate type
`https://proof-integrity.local/receipt/v4`. Python version-3 receipts must be regenerated; renaming
their version is not a migration. Refer to the typed records in `scripts/src/` for the exact wire
fields and to the integration fixtures in `scripts/tests/` for schema examples. Test fixture
receipts are synthetic inputs to the validator and must never be presented as product evidence.

Use the binary's digest command when assembling real evidence:

```sh
"$PROOF_BIN" digest --file /tmp/mutation-before.txt
"$PROOF_BIN" digest --file /tmp/mutant.json --json
```

The output's `digest` is SHA-256 in lowercase hexadecimal with the `sha256:` prefix. Raw content
uses exact file bytes. Structured content uses compact JSON with object keys sorted recursively,
array order preserved, UTF-8 strings, and no trailing newline. Duplicate keys and floating-point
numbers are rejected. Hash the exact target array for `targets_digest`, the mutant object for `mutant_digest`,
and the evidence object for `artifact_digest`; do not hash the pretty-printed receipt file instead.

The receipt records:

- the epoch's subject and policy digests;
- the auditor's session identity, host, freshness, memory, authorship, and sandbox declarations;
- material claims, their sources, enforcement points, oracles, and covered paths;
- each high-impact claim's mutant, target contents and digests, red/green execution evidence,
  exit codes, and artifact digest;
- the auditor's `PROOF_ADEQUATE` verdict.

Every triggered path needs a high-impact concrete claim of its own, in addition to required
generic claims. Category claims cover every path in that category. Unknown fields, invalid types,
unsupported versions, missing witnesses, and stale records are rejected rather than repaired.

Path classification is deliberately conservative. In a mixed change, an editorial-only matched
path can therefore prevent an adequate receipt; record that false positive and return `PROOF_WEAK`
rather than inventing a high-impact claim or a mutation. A purely editorial change can return
`NOT_APPLICABLE` after semantic inspection without requesting a gate verdict. Conversely, a
verification change missed by the classifier remains a review finding; a classifier omission is
not permission to omit its material claims from the audit.

Mutation targets distinguish `repository`, `policy`, and `fixture` scopes. Exact UTF-8 content and
canonical relative paths bind their before/after states; fixture-only evidence cannot establish a
material repository or policy claim. At most 64 targets and 8 MiB of carried content are allowed
per witness. Repository/policy overlays and fixture input replacement are distinct injection modes.

The evidence must contain the required marker as a complete line within a single output stream:

```text
PROOF_RED:<claim-id>:<mutant-digest>
PROOF_GREEN:<claim-id>:<subject-digest>
```

The red output must also contain `expected_failure` as an exact complete line. That diagnostic
must contain at least eight characters, no surrounding whitespace or control characters, and no
`PROOF_` marker. Preserve the actual failing diagnostic; a marker alone is not an observed failure.

Markers, hashes, command strings, and auditor booleans remain declarations. A passing receipt
cannot establish that commands ran, a fault was relevant, the auditor was independent, or a
sandbox was enforced. The reviewer must assess that evidence separately before returning the
skill's adequate verdict. This CLI is not installed as a forge-side merge restriction.

## Tests and migration boundaries

Run the package's real CLI integration tests from any copy of the skill:

```sh
cargo test --locked --manifest-path "$PROOF_SKILL_ROOT/scripts/Cargo.toml"
cargo clippy --all-targets --all-features --locked --manifest-path "$PROOF_SKILL_ROOT/scripts/Cargo.toml" -- -D warnings
```

The Python classifier and epoch/gate policy are replaced by this binary. The Python negative
witness runner becomes Rust tests of refusal and restored acceptance. The author's Windows paths,
installed Codex/Claude policy manifests, and external Bitbucket publishing scripts are not copied:
the explicit skill root owns policy inputs, and publication stays with the authorized forge
workflow. Python bytecode and historical PR receipts are not distribution artifacts.
