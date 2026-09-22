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

The current contract uses schema version 5 and predicate type
`https://proof-integrity.local/receipt/v5`. Earlier receipts and epochs remain historical and must be
regenerated after a new independent assessment; renaming their version or verdict is not a migration.
Refer to the typed records in `scripts/src/` for the exact wire
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
- the auditor's session identity, host, freshness, independent first pass, authorship and sandbox;
- capability fields `persistent_memory` and `write_tools_enabled`: `true`, `false`, or `null`
  (unknown), never a fabricated boolean. Essential independence declarations remain required;
- nullable `isolation`: no explicit obligation is `null`; otherwise an object with `requirement`,
  `enforced` (true/false/null) and `evidence`. Only true with nonempty obligation and evidence passes;
- claims with `impact` (high/low), `kind` (modified-oracle/critical-guarantee/other), `rationale`,
  sources, enforcement points, oracles and covered paths;
- each claim's `positive_evidence`: `origin` (reviewer/ci/author/retained), `artifact`, `input_basis`
  and `environment`, all nonempty. References preserve the original run and input comparison;
- each modified oracle or critical/high-impact claim's mutant, target contents and digests, red/green execution evidence,
  exit codes, and artifact digest;
- the auditor's `PROOF_ADEQUATE` verdict and `limitations` array for nonessential limits.

Every changed path needs a semantic assessment, possibly grouped with related paths. Low-impact
editorial changes use kind `other` and a rationale, with relevant inspection as positive evidence;
they need no mutation. Categories impose neither impact nor generic claim families. Unmatched paths
are assessed too; the gate never returns an automatic exemption from classifier output. A purely
editorial change can return `NOT_APPLICABLE` after semantic inspection without requesting a gate.
Unknown fields, invalid types, unsupported versions, essential missing witnesses and stale records
are rejected. Explicit unknown capabilities are valid values, not malformed records.

The gate compares repository and policy state with the pre-audit epoch even when write tools are
available. This detects persistent changes, not transient writes later restored. The auditor keeps
the candidate untouched and uses disposable copies for experiments. A security contract requiring
prevention rather than stability checks needs technical isolation, with its actual evidence.

The gate validates reference structure, not the remote CI service or the semantic completeness of
the input comparison, risk classification or isolation obligation. The independent auditor checks
these, including whether a claimed low impact hides a critical guarantee or modified oracle. Missing
essential evidence blocks; bounded limits remain in `limitations`. Reuse prior raw execution only
after comparing relevant source, oracle, configuration, dependency, environment and integration
inputs. Generate current bindings without rewriting the original logs or claiming a new execution.

Mutation targets distinguish `repository`, `policy`, and `fixture` scopes. Exact UTF-8 content and
canonical relative paths bind their before/after states; fixture-only evidence cannot establish a
material repository or policy claim. At most 64 targets and 8 MiB of carried content are allowed
per witness. Repository/policy overlays and fixture input replacement are distinct injection modes.

Each witness's `evidence.provenance` uses the same observation fields as `positive_evidence`.
Keep red and green output streams verbatim, including when reusing historical logs. The current
receipt binds them through `artifact_digest`, target digests and an explicit input comparison;
no added execution markers are required. Old markers may remain in unchanged historical outputs.

The red output must contain `expected_failure` as an exact complete line. That diagnostic
must contain at least eight characters, no surrounding whitespace or control characters, and no
`PROOF_` marker. Preserve the actual failing diagnostic; a marker alone is not an observed failure.

Hashes, provenance references, command strings and auditor booleans remain declarations. A passing receipt
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
