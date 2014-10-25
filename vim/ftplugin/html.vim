if &filetype !~ 'html'
  finish
endif

"see http://google-styleguide.googlecode.com/svn/trunk/htmlcssguide.xml#Indentation
setlocal tabstop=2
setlocal shiftwidth=2
setlocal softtabstop=2
