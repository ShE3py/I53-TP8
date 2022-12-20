%{
  #include <stdio.h>
  #include <ctype.h>
  #include <unistd.h>
  
  #include "asa.h"
  #include "ts.h"

  extern int yylex();
%}

%define parse.error verbose

%token Start End Newline Indent Semicolon
%token Print

%token <n> Number
%token <s> Identifier
%type <node> StatementList Statement Expr Value
%token OpenParenthesis CloseParenthesis

%left Add Sub
%left Mul Div

%right Assign

%start Program

%union {
	int n;
	char s[32];
	struct asa *node;
}

%%

Program: Start Newline StatementList End { codegen($3); free_asa($3); return 0; };

StatementList:
  Indent Statement Semicolon Newline StatementList { $$ = creer_noeudBloc($2, $5); }
| Indent Newline StatementList                     { $$ = $3; }
| %empty                                           { $$ = NULL; }
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
  OpenParenthesis Expr CloseParenthesis { $$ = $2; }
| Number                                { $$ = creer_feuilleNb($1); }
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

