set project_root (path resolve (path dirname (status filename))/../../../..)
set test_root (mktemp -d)

function fail --argument-names message
    echo "not ok - $message" >&2
    exit 1
end

function cleanup --on-event fish_exit
    command rm -rf -- "$test_root"
end

set -gx HOME "$test_root/home"
mkdir -p "$HOME"
ln -s "$project_root" "$HOME/.dotfiles"
source "$project_root/home/.config/fish/conf.d/git-abbreviations.fish"

set repository "$test_root/repository"
git init --quiet --initial-branch=main "$repository"; or fail "repository initialization failed"
git -C "$repository" config user.email test@example.com
git -C "$repository" config user.name Test
echo base >"$repository/file"
git -C "$repository" add file
git -C "$repository" commit --quiet --message base
git -C "$repository" switch --quiet --create feature
echo feature >"$repository/feature"
git -C "$repository" add feature
git -C "$repository" commit --quiet --message feature
git -C "$repository" switch --quiet main
echo main >"$repository/main"
git -C "$repository" add main
git -C "$repository" commit --quiet --message main
set main_tip (git -C "$repository" rev-parse main)
git -C "$repository" switch --quiet feature

cd "$repository"
grbm; or fail "grbm failed"
test (git merge-base HEAD main) = "$main_tip"; or fail "grbm did not rebase onto main"
echo "ok - grbm resolves the main branch through the shipped entrypoint"
