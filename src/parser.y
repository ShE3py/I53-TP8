%{
  #include "ts.h"
  #include "asa.h"
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
%token Read Print

// Ponctuation
%token LeftParenthesis RightParenthesis
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

// Opérateurs
%right Assign

%left Or Xor
%left And
%left Not

%left Ge Gt Le Lt Eq Ne

%left Add Sub
%left Mul Div Mod
%left UnaryPlus UnaryMinus

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
| Block Statements               { $$ = make_block_node($1, $2); }
| %empty                         { $$ = NULL; }
;

Statement:
  Expr                                    { $$ = $1; }
| Var Identifier                          { if(ts_retrouver_id($2)) yyerror("variable dupliquée"); ts_ajouter_id($2, 1); $$ = NULL; }
| Read Identifier                         { if(!ts_retrouver_id($2)) ts_ajouter_id($2, 1); $$ = create_read_node($2); }
| Identifier Assign Statement             { $$ = create_assign_node($1, $3); }
| Print Expr                              { $$ = create_print_node($2); }
;

Block:
  If BoolExpr Then Statements ElseOrEndIf { $$ = create_test_node($2, $4, $5); }
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
  LeftParenthesis IntExpr RightParenthesis { $$ = $2; }
| Int                                      { $$ = create_int_leaf($1); }
| Identifier                               { $$ = create_var_leaf($1); }
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
  
	if(argc == 1) {
		fprintf(stderr, "%s <input>\n", argv[0]);
		return 1;
	}
	
	input = argv[1];
	
	yyin = fopen(input, "r");
	yyparse();
	return 0;
}
