%{
#include <string.h>

#include "parser.h"

extern const char *infile;

void yyerror(const char *s) {
	fprintf(stderr, "%s:%i: %s\n", infile, yylineno, s);
	exit(1);
}
%}

%option nounput
%option noinput
%option yylineno
  
%%

 /* Mots-clefs */
"FONCTION"      { return Fn; }
"RENVOYER"      { return Return; }

"DEBUT"|"DÉBUT" { return Start; }
"FIN"           { return End; }

"VAR"           { return Var; }

"SI"            { return If; }
"ALORS"         { return Then; }
"SINON"         { return Else; }
"FSI"           { return EndIf; }

"TQ"            { return While; }
"FAIRE"         { return Do; }
"FTQ"           { return EndWhile; }

"LIRE"          { return Read; }
"AFFICHER"      { return Print; }

 /* Opérateurs logiques */
"ET"          { return And; }
"OU"          { return Or; }
"OU EXCLUSIF" { return Xor; }
"NON"         { return Not; }

 /* Littéraux et identifiants */
[-+]?[0-9]+            { yylval.ival = atoi(yytext); return Int; }
[A-Za-z_][A-Za-z0-9_]* { if(yyleng > 32) yyerror("identifier length exceeded"); strcpy(yylval.sval, yytext); return Identifier; }

 /* Opérateurs d'affectation */
"<-"|":=" { return Assign; }

 /* Opérateurs arithmétiques */
"+"       { return Add; }
"-"       { return Sub; }
"*"       { return Mul; }
"/"       { return Div; }
"%"       { return Mod; }

 /* Opérateurs de comparaison */
">="      { return Ge; }
">"       { return Gt; }
"<="      { return Le; }
"<"       { return Lt; }
"="|"=="  { return Eq; }
"!="      { return Ne; }

 /* Ponctuation */
"("       { return LeftParenthesis; }
")"       { return RightParenthesis; }
"["       { return LeftSquareBracket; }
"]"       { return RightSquareBracket; }
"{"       { return LeftBracket; }
"}"       { return RightBracket; }
"."       { return Dot; }
","       { return Comma; }
";"       { return Semicolon; }

 /* Espaces et sauts de ligne */
[\n\r \t]+ {}

 /* Commentaires */
#.+ {}

 /* Tout le reste */
. { fprintf(stderr, "%1$s:%2$i: caractère inconnu: '%3$c' (%3$u)\n", infile, yylineno, (unsigned char) yytext[0]); exit(1); }

%%

