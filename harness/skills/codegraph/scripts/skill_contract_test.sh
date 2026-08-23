#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
skill="$here/../SKILL.md"

require() {
  rg -qF -- "$1" "$skill" || {
    printf 'CodeGraph skill contract failed: missing %s\n' "$1" >&2
    exit 1
  }
}

require 'git check-ignore -q --no-index .codegraph/index.db'
require 'Do not infer a cache-write prohibition from'
require 'a request to analyze, review, or audit without editing the repository.'
require 'forbids local cache writes or all filesystem writes.'
require 'Never edit a project `.gitignore` to initialize CodeGraph.'

printf 'CodeGraph skill contract passed (advisory)\n'
