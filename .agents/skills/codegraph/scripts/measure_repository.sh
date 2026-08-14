#!/usr/bin/env bash
set -euo pipefail

repository=${1:-.}
tokei_bin=${CODEGRAPH_TOKEI_BIN:-tokei}
jq_bin=${CODEGRAPH_JQ_BIN:-jq}
types='Ark TypeScript,Astro,C,C Header,C#,COBOL,ColdFusion,ColdFusion CFScript,C++,C++ Header,C++ Module,CUDA,Dart,Erlang,Go,HCL,Java,JavaScript,JSX,Kotlin,Liquid,Lua,Metal Shading Language,Nix,Objective-C,Objective-C++,Pascal,PHP,Python,R,Razor,Ruby,Rust,Scala,Solidity,Svelte,Swift,TSX,TypeScript,Visual Basic,Vue'

if [[ "$tokei_bin" == */* ]]; then
  [ -x "$tokei_bin" ] || { echo 'tokei is required' >&2; exit 2; }
else
  command -v "$tokei_bin" >/dev/null 2>&1 || { echo 'tokei is required' >&2; exit 2; }
fi

if [[ "$jq_bin" == */* ]]; then
  [ -x "$jq_bin" ] || { echo 'jq is required' >&2; exit 2; }
else
  command -v "$jq_bin" >/dev/null 2>&1 || { echo 'jq is required' >&2; exit 2; }
fi

repository_path=$(cd "$repository" && pwd -P)
tofu_links=$(mktemp -d)
trap 'rm -rf "$tofu_links"' EXIT
tofu_count=0

link_tofu() {
  file=$1
  case "/$file/" in
    */.git/*|*/.codegraph/*|*/.worktrees/*|*/node_modules/*|*/vendor/*|*/dist/*|*/build/*|*/out/*|*/target/*|*/coverage/*|*/generated/*|*/docs/*|*/fixtures/*) return ;;
  esac
  case "$file" in
    *.[Tt][Oo][Ff][Uu]) ;;
    *) return ;;
  esac
  tofu_count=$((tofu_count + 1))
  ln -s "$repository_path/$file" "$tofu_links/$tofu_count.tf"
}

if git -C "$repository_path" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  while IFS= read -r -d '' file; do
    link_tofu "$file"
  done < <(git -C "$repository_path" ls-files --cached --others --exclude-standard -z)
else
  while IFS= read -r -d '' file; do
    link_tofu "${file#"$repository_path"/}"
  done < <(find "$repository_path" -type f -iname '*.tofu' -print0)
fi

{
  "$tokei_bin" "$repository_path" \
    --hidden \
    --streaming json \
    --types "$types" \
    --exclude .git \
    --exclude .codegraph \
    --exclude .worktrees \
    --exclude node_modules \
    --exclude vendor \
    --exclude dist \
    --exclude build \
    --exclude out \
    --exclude target \
    --exclude coverage \
    --exclude generated \
    --exclude docs \
    --exclude fixtures \
    --exclude '*.lock' \
    --exclude '*.min.js'
  if [ "$tofu_count" -gt 0 ]; then
    find "$tofu_links" -type l -print0 | xargs -0 "$tokei_bin" --streaming json --types HCL
  fi
} | "$jq_bin" -sc '
    [
      .[]
      | select(.stats.name | test("\\.(astro|c|cc|cbl|cob|cobol|cfc|cfm|cfs|cjs|cpp|cpy|cs|cshtml|cts|cu|cuh|cxx|dart|dfm|dpk|dpr|erl|escript|ets|fmx|go|h|hpp|hrl|hxx|inc|install|java|js|jsx|kt|kts|liquid|lpr|lua|luau|m|metal|mjs|mm|module|mts|nix|pas|php|py|pyw|r|rake|razor|rb|rs|sc|scala|sol|svelte|swift|tf|tfvars|theme|ts|tsx|vb|vue|xsjs|xsjslib)$"; "i"))
    ] as $source
    | ([$source[] | .stats.stats.code // 0] | add // 0) as $loc
    | ($source | length) as $files
    | {loc: $loc, files: $files, initialize: ($loc >= 50000 or $files >= 500)}
  '
