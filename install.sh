
if [[ "$(uname -s)" != "Darwin" ]]
then
  echo "This installer runs only on OSX"
  exit 1
fi

if [[ "$(which git2)" == "" ]]
then
  echo "This installer need git"
  echo "Do you want to install xcode ?"
  read -s answer
  if [[ $answer != [yY] ]]
  then
    exit 0
  fi

  echo "Install install xcode"
  xcode-select --install
fi

cd 
git clone https://github.com/Nasga/dotfiles.git .dotfiles
cd .dotfiles
make all
