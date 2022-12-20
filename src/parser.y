%{
  #include "ts.h"
  #include "asa.h"
  #include "codegen.h"

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
  Statement Semicolon Statements { $$ = make_block_node($1, $3); }
| %empty                         { $$ = NULL; }
;

Statement:
  Expr                        { $$ = $1; }
| Identifier Assign Statement { $$ = create_assign_node($1, $3); }
| Print Expr                  { $$ = create_print_node($2); }
;

Expr:
  Expr Add Expr { $$ = create_binop_node(OpAdd, $1, $3); }
| Expr Sub Expr { $$ = create_binop_node(OpSub, $1, $3); }
| Expr Mul Expr { $$ = create_binop_node(OpMul, $1, $3); }
| Expr Div Expr { $$ = create_binop_node(OpDiv, $1, $3); }
| Expr Mod Expr { $$ = create_binop_node(OpMod, $1, $3); }
| Value         { $$ = $1; }
;

Value:
  LeftParenthesis Expr RightParenthesis { $$ = $2; }
| Int                                   { $$ = create_int_leaf($1); }
| Identifier                            { $$ = create_var_leaf($1); }
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

