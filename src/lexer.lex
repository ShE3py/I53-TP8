%{
#include <string.h>

#include "parser.h" 

void yyerror(const char *s);
%}
  
%option nounput
%option noinput
  
%%

 /* Mots-clefs */
"DEBUT"|"DÉBUT" { return Start; }
"FIN"           { return End; }

"AFFICHER"      { return Print; }

 /* Littéraux et identifiants */
[0-9]+                 { yylval.ival = atoi(yytext); return Int; }
[A-Za-z_][A-Za-z0-9_]* { if(yyleng > 32) yyerror("identifier length exceeded"); strcpy(yylval.sval, yytext); return Identifier; }

 /* Opérateurs d'affectation */
"->"|":=" { return Assign; }

 /* Opérateurs arithmétiques */
"+"       { return Add; }
"-"       { return Sub; }
"*"       { return Mul; }
"/"       { return Div; }
"%"       { return Mod; }

 /* Ponctuation */
"("       { return LeftParenthesis; }
")"       { return RightParenthesis; }
";"       { return Semicolon; }

 /* Espaces et sauts de ligne */
[\n\r \t]+ {}

 /* Tout le reste */
. { fprintf(stderr, "[err lexer] caractere inconnu: '%c'\n", yytext[0]); return 1; }

%%
