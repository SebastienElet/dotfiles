function abbreviations
	set -l abbr_file ~/.config/fish/conf.d/git-abbreviations.fish
	if not test -f $abbr_file
		set abbr_file ~/.dotfiles/fish/conf.d/git-abbreviations.fish
	end
	
	if test -f $abbr_file
		command sed -E "s/^abbr -a ([^ ]+) '(.+)'\$/\\1 = \\2/" $abbr_file
	else
		for abbr_name in (abbr -l)
			echo "$abbr_name = (abbreviation)"
		end
	end
end
