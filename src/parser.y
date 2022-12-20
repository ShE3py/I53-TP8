%{
  #include <stdio.h>
  #include <ctype.h>
  #include <unistd.h>
  
  #include "asa.h"
  #include "ts.h"

  extern int yylex();
%}

%define parse.error verbose

// Mots-clefs
%token Start End
%token Print

// Ponctuation
%token LeftParenthesis RightParenthesis
%token Semicolon

// Littéraux et identifiants
%token <ival> Int
%token <sval> Identifier

// Non-terminaux
%start Program
%type <nval> Statements Statement Expr Value

// Opérateurs
%right Assign

%left Add Sub
%left Mul Div Mod

// yylval
%union {
	int ival;
	char sval[32];
	struct asa *nval;
}

%%

Program: Start Statements End { codegen($2); free_asa($2); return 0; };

Statements:
  Statement Semicolon Statements { $$ = creer_noeudBloc($1, $3); }
| %empty                         { $$ = NULL; }
;

Statement:
  Expr                        { $$ = $1; }
| Identifier Assign Statement { $$ = creer_noeudAffect($1, $3); }
| Print Expr                  { $$ = creer_noeudAfficher($2); }
;

Expr:
  Expr Add Expr { $$ = creer_noeudOp('+', $1, $3); }
| Expr Sub Expr { $$ = creer_noeudOp('-', $1, $3); }
| Expr Mul Expr { $$ = creer_noeudOp('*', $1, $3); }
| Expr Div Expr { $$ = creer_noeudOp('/', $1, $3); }
| Value         { $$ = $1; }
;

Value:
  LeftParenthesis Expr RightParenthesis { $$ = $2; }
| Int                                   { $$ = creer_feuilleNb($1); }
| Identifier                            { $$ = creer_noeudVar($1); }
;

%%

int main( int argc, char * argv[] ) {
  extern FILE *yyin;
  
  if(argc == 1) {
    fprintf(stderr, "aucun fichier fournie\n");
    return 1;
  }
  
  yyin = fopen(argv[1], "r");
  yyparse();
  return 0;
}

