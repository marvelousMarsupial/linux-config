; extends
; Only parsed operators: strings, comments and plain text stay literal.
("=>" @operator.ligature (#set! conceal "⇒"))
("===" @operator.ligature (#set! conceal "≡"))
("!==" @operator.ligature (#set! conceal "≢"))
("!=" @operator.ligature (#set! conceal "≠"))
("<=" @operator.ligature (#set! conceal "≤"))
(">=" @operator.ligature (#set! conceal "≥"))
