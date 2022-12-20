%{
#include <string.h>
#include "parser.h" 

void yyerror(const char *s);
%}
  
%option nounput
%option noinput

DIGIT [0-9]
  
%%

"DEBUT"    { return Start; }
"FIN"      { return End; }
"AFFICHER" { return Print; }

{DIGIT}                { yylval.n = atoi(yytext); return Number; }
[A-Za-z_][A-Za-z0-9_]* { if(yyleng > 32) yyerror("identifier length exceeded"); strcpy(yylval.s, yytext); return Identifier; }

"+"       { return Add; }
"-"       { return Sub; }
"*"       { return Mul; }
"/"       { return Div; }

"("       { return OpenParenthesis; }
")"       { return CloseParenthesis; }

"->"|":=" { return Assign; }

\n|\r\n|\r|\036|\n\r|\025 { return Newline; }
\t+                       { return Indent; }
;                         { return Semicolon; }
" "                       {}
.                         { fprintf(stderr, "[err lexer] caractere inconnu: '%c'\n", yytext[0]); return 1; }

%%

