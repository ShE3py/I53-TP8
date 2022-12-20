%{
#include <string.h>

#include "parser.h"

extern const char *input;

void yyerror(const char *s) {
	fprintf(stderr, "%s:%i: %s\n", input, yylineno, s);
	exit(1);
}
%}
  
%option nounput
%option noinput
%option yylineno
  
%%

 /* Mots-clefs */
"DEBUT"|"DÉBUT" { return Start; }
"FIN"           { return End; }

"VAR"           { return Var; }
"AFFICHER"      { return Print; }

 /* Littéraux et identifiants */
[0-9]+                 { yylval.ival = atoi(yytext); return Int; }
[A-Za-z_][A-Za-z0-9_]* { if(yyleng > 32) yyerror("identifier length exceeded"); strcpy(yylval.sval, yytext); return Identifier; }

 /* Opérateurs d'affectation */
"<-"|":=" { return Assign; }

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
. { fprintf(stderr, "%1$s:%2$i: caractère inconnu: '%3$c' (%3$u)\n", input, yylineno, (unsigned char) yytext[0]); exit(1); }

%%
