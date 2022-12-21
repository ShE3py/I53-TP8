%{
  #include "ts.h"
  #include "codegen.h"

  extern int yylex();
  extern void yyerror(const char *s);
  
  const char *input;
%}

%define parse.error verbose

// Mots-clefs
%token Start End
%token Var
%token If Then Else EndIf
%token While Do EndWhile
%token Read Print

// Ponctuation
%token LeftParenthesis RightParenthesis
%token LeftSquareBracket RightSquareBracket
%token LeftBracket RightBracket
%token Dot
%token Comma
%token Semicolon

// Littéraux et identifiants
%token <ival> Int
%token <sval> Identifier

// Non-terminaux
%start Program
%type <nval> Statements Statement Block
%type <nval> ElseOrEndIf
%type <nval> Expr
%type <nval> IntExpr IntValue
%type <nval> CmpExpr
%type <nval> BoolExpr

%type <lval> IntArray IntList

// Opérateurs
%right Assign

%left Or Xor
%left And
%left Not

%left Ge Gt Le Lt Eq Ne

%left Add Sub
%left Mul Div Mod
%left UnaryPlus UnaryMinus

%code requires {
	#include "asa.h"
}

// yylval
%union {
	int ival;
	char sval[32];
	struct asa *nval;
	asa_list lval;
}

%%

Program: Start Statements End { codegen($2); free_asa($2); ts_free_table(); return 0; };

Statements:
  Statement Semicolon Statements { $$ = make_block_node($1, $3); }
| Block Statements               { $$ = make_block_node($1, $2); }
| %empty                         { $$ = NULL; }
;

Statement:
  Expr                                                                     { $$ = $1; }
| Var Identifier                                                           { if(ts_retrouver_id($2)) yyerror("variable dupliquée"); ts_ajouter_scalaire($2); $$ = NULL; }
| Var Identifier Assign Statement                                          { if(ts_retrouver_id($2)) yyerror("variable dupliquée"); ts_ajouter_scalaire($2); $$ = create_assign_node($2, $4); }
| Var Identifier LeftSquareBracket Int RightSquareBracket                  { if(ts_retrouver_id($2)) yyerror("variable dupliquée"); ts_ajouter_tableau($2, $4); $$ = NULL; }
| Var Identifier Assign IntArray                                           { if(ts_retrouver_id($2)) yyerror("variable dupliquée"); ts_ajouter_tableau($2, $4.len); $$ = create_assign_int_list_node($2, $4); }

| Read Identifier                                                          { if(!ts_retrouver_id($2)) ts_ajouter_scalaire($2); $$ = create_read_node($2); }
| Read Identifier LeftSquareBracket IntExpr RightSquareBracket             { if(!ts_retrouver_id($2)) yyerror("variable inconnue"); $$ = create_read_indexed_node($2, $4); }
| Read LeftSquareBracket Int RightSquareBracket Identifier                 { if( ts_retrouver_id($5)) yyerror("variable dupliquée"); ts_ajouter_tableau($5, $3); $$ = create_read_array_node($5); }
| Read LeftSquareBracket Identifier RightSquareBracket                     { if(!ts_retrouver_id($3)) yyerror("variable inconnue"); $$ = create_read_array_node($3); }

| Identifier Assign Statement                                              { $$ = create_assign_node($1, $3); }
| Identifier Assign IntArray                                               { $$ = create_assign_int_list_node($1, $3); }
| Identifier LeftSquareBracket IntExpr RightSquareBracket Assign Statement { $$ = create_assign_indexed_node($1, $3, $6); }

| Print Expr                                                               { $$ = create_print_node($2); }
| Print LeftSquareBracket Identifier RightSquareBracket                    { $$ = create_print_array_node($3); }
;

IntArray:
  LeftBracket RightBracket                 { $$ = asa_list_empty(); }
| LeftBracket IntExpr IntList RightBracket { $$ = asa_list_append($2, $3); }
;

IntList:
  Comma IntExpr IntList { $$ = asa_list_append($2, $3); }
| %empty                { $$ = asa_list_empty(); }
;

Block:
  If BoolExpr Then Statements ElseOrEndIf { $$ = create_test_node($2, $4, $5); }
| While BoolExpr Do Statements EndWhile   { $$ = create_while_node($2, $4); }
;

ElseOrEndIf:
  Else Statements EndIf { $$ = $2; }
| EndIf                 { $$ = NULL; }
;

Expr:
  IntExpr  { $$ = $1; }
//CmpExpr  { $$ = $1; }  accessible with BoolExpr
| BoolExpr { $$ = $1; }
;

IntExpr:
  IntExpr Add IntExpr           { $$ = create_binop_node(OpAdd, $1, $3); }
| IntExpr Sub IntExpr           { $$ = create_binop_node(OpSub, $1, $3); }
| IntExpr Mul IntExpr           { $$ = create_binop_node(OpMul, $1, $3); }
| IntExpr Div IntExpr           { $$ = create_binop_node(OpDiv, $1, $3); }
| IntExpr Mod IntExpr           { $$ = create_binop_node(OpMod, $1, $3); }
| Sub IntValue %prec UnaryMinus { $$ = create_unop_node(OpNeg, $2); }
| Add IntValue %prec UnaryPlus  { $$ = $2; }
| IntValue                      { $$ = $1; }
;

IntValue:
  LeftParenthesis IntExpr RightParenthesis                   { $$ = $2; }
| Int                                                        { $$ = create_int_leaf($1); }
| Identifier                                                 { $$ = create_var_leaf($1); }
| Identifier LeftSquareBracket IntExpr RightSquareBracket    { $$ = create_index_node($1, $3); }
| Identifier Dot Identifier LeftParenthesis RightParenthesis { $$ = create_fncall_node($1, $3); }
;

CmpExpr:
  IntExpr Ge IntExpr { $$ = create_binop_node(OpGe, $1, $3); }
| IntExpr Gt IntExpr { $$ = create_binop_node(OpGt, $1, $3); }
| IntExpr Le IntExpr { $$ = create_binop_node(OpLe, $1, $3); }
| IntExpr Lt IntExpr { $$ = create_binop_node(OpLt, $1, $3); }
| IntExpr Eq IntExpr { $$ = create_binop_node(OpEq, $1, $3); }
| IntExpr Ne IntExpr { $$ = create_binop_node(OpNe, $1, $3); }
;

BoolExpr:
  BoolExpr And BoolExpr                     { $$ = create_binop_node(OpAnd, $1, $3); }
| BoolExpr Or BoolExpr                      { $$ = create_binop_node(OpOr, $1, $3); }
| BoolExpr Xor BoolExpr                     { $$ = create_binop_node(OpXor, $1, $3); }
| Not BoolExpr                              { $$ = create_unop_node(OpNot, $2); }
| CmpExpr                                   { $$ = $1; }
| LeftParenthesis BoolExpr RightParenthesis { $$ = $2; }
;

%%

int main(int argc, char *argv[]) {
	extern FILE *yyin;
	extern int yylex_destroy(void);
	
	if(argc == 1) {
		fprintf(stderr, "%s <input>\n", argv[0]);
		return 1;
	}
	
	input = argv[1];
	
	FILE *f = fopen(input, "r");
	
	yyin = f;
	yyparse();
	yylex_destroy();
	fclose(f);
	return 0;
}
