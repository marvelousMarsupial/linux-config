" Python has no installed Tree-sitter parser in the base setup. Vim syntax
" regions keep these matches out of strings and comments.
syntax match pythonOperatorLigatureArrow /->/ conceal cchar=→
syntax match pythonOperatorLigatureNotEqual /!=/ conceal cchar=≠
syntax match pythonOperatorLigatureLessEqual /<=/ conceal cchar=≤
syntax match pythonOperatorLigatureGreaterEqual />=/ conceal cchar=≥
highlight default link pythonOperatorLigatureArrow OperatorLigature
highlight default link pythonOperatorLigatureNotEqual OperatorLigature
highlight default link pythonOperatorLigatureLessEqual OperatorLigature
highlight default link pythonOperatorLigatureGreaterEqual OperatorLigature
