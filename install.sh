
if [[ "$(uname -s)" != "Darwin" ]]
then
  echo "This installer runs only on OSX"
  exit 1
fi

if [[ "$(which git)" == "" ]]
then
  echo "This installer need git"
  echo "Do you want to install xcode ?"
  read answer
  if [[ $answer != [yY] ]]
  then
    exit 0
  fi

  echo "Install install xcode"
  xcode-select --install
  exit 0
fi

cd 
git clone --depth 1 https://github.com/SebastienElet/dotfiles.git .dotfiles
cd .dotfiles
make all
