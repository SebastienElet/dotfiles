#!/usr/bin/env bash
set -euo pipefail

repository=${1:-.}
tokei_bin=${CODEGRAPH_TOKEI_BIN:-tokei}
jq_bin=${CODEGRAPH_JQ_BIN:-jq}
types='Ark TypeScript,Astro,C,C Header,C#,COBOL,ColdFusion,ColdFusion CFScript,C++,C++ Header,C++ Module,CUDA,Dart,Erlang,Go,HCL,Java,JavaScript,JSX,Kotlin,Liquid,Lua,Metal Shading Language,Nix,Objective-C,Objective-C++,Pascal,PHP,Python,R,Ruby,Rust,Scala,Solidity,Svelte,Swift,TSX,TypeScript,Visual Basic,Vue'

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

"$tokei_bin" "$repository" \
  --hidden \
  --output json \
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
  --exclude '*.min.js' |
  "$jq_bin" -c '
    ([to_entries[] | select(.key != "Total") | .value.code // 0] | add // 0) as $loc
    | ([to_entries[] | select(.key != "Total") | .value.reports[]?] | length) as $files
    | {loc: $loc, files: $files, initialize: ($loc >= 50000 or $files >= 500)}
  '
