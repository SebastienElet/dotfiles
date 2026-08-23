#!/usr/bin/env bash
set -euo pipefail

test_root=$(mktemp -d "${TMPDIR:-/tmp}/obsidian-retrieval.XXXXXX")
trap 'rm -rf "$test_root"' EXIT

fail() {
  printf 'Obsidian retrieval smoke failed: %s\n' "$1" >&2
  exit 1
}

lexical_candidates() {
  local root=$1
  local query=$2

  if command -v rg >/dev/null 2>&1; then
    rg --files --glob '*.md' --glob '!.obsidian/**' "$root" | grep -F -- "$query" || true
    rg -n -F --glob '*.md' --glob '!.obsidian/**' -- "$query" "$root" || true
    return
  fi
  find "$root" -type f -name '*.md' ! -path '*/.obsidian/*' -print | grep -F -- "$query" || true
  find "$root" -type f -name '*.md' ! -path '*/.obsidian/*' -exec grep -nH -F -- "$query" {} + || true
}

read_candidate() {
  local candidate=$1
  local path=${candidate%%:*}

  printf 'source=%s\n' "$path"
  sed -n '1,120p' "$path"
}

conceptual_retrieval() {
  local root=$1
  local query=$2

  if command -v qmd >/dev/null 2>&1 && qmd status >/dev/null 2>&1; then
    qmd search "$query"
    return
  fi
  printf 'degraded=semantic-unavailable; fallback=lexical\n'
  lexical_candidates "$root" "$query" | head -n 1
}

obsidian_retrieval() {
  local root=$1
  local target=$2

  if command -v obsidian >/dev/null 2>&1 && obsidian version >/dev/null 2>&1; then
    (
      cd "$root"
      obsidian backlinks path="$target" format=tsv
      obsidian read path="$target"
    )
    return
  fi
  printf 'degraded=obsidian-unavailable; fallback=filesystem\n'
  lexical_candidates "$root" "$target"
}

vault=$test_root/vault
mkdir -p "$vault/.obsidian" "$vault/decisions" "$test_root/bin"
cat > "$vault/decisions/portable-retrieval.md" <<'EOF'
# Portable retrieval
Anchor: ADR-183
Distributed decisions need explicit source reading.
The source-only detail proves that the note was read.
EOF
cat > "$vault/index.md" <<'EOF'
# Index
[[decisions/portable-retrieval]]
EOF

exact=$(lexical_candidates "$vault" 'ADR-183' | head -n 1)
[[ $exact == *'portable-retrieval.md:2:'* ]] || fail 'exact anchor did not return a path and line'
source=$(read_candidate "$exact")
[[ $source == *'source-only detail'* ]] || fail 'source note was not read after lexical search'

cat > "$test_root/bin/obsidian" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$OBSIDIAN_SMOKE_LOG"
if [[ ${OBSIDIAN_SMOKE_DISABLED:-} == 1 && $1 == version ]]; then
  exit 69
fi
case "$1" in
  version) printf '1.12.7\n' ;;
  backlinks) printf 'index.md\t1\n' ;;
  read) sed -n '1,120p' "$OBSIDIAN_SMOKE_ROOT/${2#path=}" ;;
  *) exit 64 ;;
esac
EOF
chmod +x "$test_root/bin/obsidian"

export OBSIDIAN_SMOKE_LOG=$test_root/obsidian.log
export OBSIDIAN_SMOKE_ROOT=$vault
relation=$(PATH="$test_root/bin:/usr/bin:/bin" obsidian_retrieval "$vault" 'decisions/portable-retrieval.md')
[[ $relation == *$'index.md\t1'* && $relation == *'source-only detail'* ]] || fail 'Obsidian relation retrieval did not read the source'
[[ $(sed -n '2p' "$OBSIDIAN_SMOKE_LOG") == 'backlinks path=decisions/portable-retrieval.md format=tsv' ]] || fail 'read-only backlinks command was not used'
[[ $(sed -n '3p' "$OBSIDIAN_SMOKE_LOG") == 'read path=decisions/portable-retrieval.md' ]] || fail 'Obsidian source was not read'

unavailable=$(PATH="/usr/bin:/bin" obsidian_retrieval "$vault" 'portable-retrieval')
[[ $unavailable == *'degraded=obsidian-unavailable; fallback=filesystem'* ]] || fail 'missing Obsidian CLI was not reported'
[[ $unavailable == *'portable-retrieval.md'* ]] || fail 'missing Obsidian CLI did not fall back to filesystem retrieval'

disabled=$(OBSIDIAN_SMOKE_DISABLED=1 PATH="$test_root/bin:/usr/bin:/bin" obsidian_retrieval "$vault" 'portable-retrieval')
[[ $disabled == *'degraded=obsidian-unavailable; fallback=filesystem'* ]] || fail 'disabled Obsidian CLI was not reported'
[[ $disabled == *'portable-retrieval.md'* ]] || fail 'disabled Obsidian CLI did not fall back to filesystem retrieval'

concept=$(PATH="/usr/bin:/bin" conceptual_retrieval "$vault" 'Distributed decisions')
[[ $concept == *'degraded=semantic-unavailable; fallback=lexical'* ]] || fail 'missing QMD was not reported'
[[ $concept == *'portable-retrieval.md:3:'* ]] || fail 'missing QMD did not fall back to lexical retrieval'

printf 'Obsidian retrieval smoke passed on %s\n' "$(uname -s)"
