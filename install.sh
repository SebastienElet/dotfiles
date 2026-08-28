#!/usr/bin/env bash

if [[ "$(uname -s)" != "Darwin" ]]
then
  echo "This installer supports macOS only; Linux and GitHub Codespaces are unsupported."
  exit 1
fi

if ! xcode-select --print-path >/dev/null
then
  echo "Apple Command Line Tools are required." >&2
  echo "Install them and complete the system dialog:" >&2
  echo "xcode-select --install" >&2
  echo "After installation finishes, rerun:" >&2
  echo "curl -fsSL https://raw.githubusercontent.com/SebastienElet/dotfiles/main/install.sh | bash" >&2
  exit 1
fi

git_path="$(command -v git)"
if [[ "$git_path" == "" ]] || ! git --version >/dev/null
then
  echo "Git is required but unavailable." >&2
  echo "Install Apple's Command Line Tools and complete the system dialog:" >&2
  echo "xcode-select --install" >&2
  echo "After installation finishes, rerun:" >&2
  echo "curl -fsSL https://raw.githubusercontent.com/SebastienElet/dotfiles/main/install.sh | bash" >&2
  exit 1
fi

cd 
git clone --depth 1 https://github.com/SebastienElet/dotfiles.git .dotfiles
cd .dotfiles
make brew
make minimal </dev/null
