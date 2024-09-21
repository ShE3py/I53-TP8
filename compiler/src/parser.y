%{
  #include <errno.h>
  #include "ts.h"
  #include "codegen.h"

  extern int yylex();
  extern void yyerror(const char *s);
  
  const char *infile;
  FILE *outfile;
%}

%define parse.error verbose

// Mots-clefs
%token Fn Return
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
%type <nlval> Fns
%type <nval> FnScope
%type <idlval> Params CommaParams
%type <sval> Param
%type <nval> Statements Statement Block
%type <nval> ElseOrEndIf
%type <nval> Expr
%type <nval> IntExpr IntValue
%type <nval> CmpExpr
%type <nval> BoolExpr

%type <nlval> IntArray IntList
%type <nlval> Args CommaArgs

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
	asa_list nlval;
	id_list idlval;
}

%%

Program: Fns { codegen($1); asa_list_destroy($1); };

Fns:
  FnScope Fns { $$ = asa_list_append($1, $2); }
| %empty      { $$ = asa_list_empty(); st_destroy_current(); }
;

FnScope: Fn Identifier LeftParenthesis Params RightParenthesis Start Statements End { $$ = create_fn_node($2, $4, $7, st_pop_push_empty()); }

Params:
  Param CommaParams { $$ = id_list_append($1, $2); }
| %empty            { $$ = id_list_empty(); }
;

CommaParams:
  Comma Param CommaParams { $$ = id_list_append($2, $3); }
| %empty                  { $$ = id_list_empty(); }
;

// force la création des paramètres dès le passage de l'identifiant,
// et non après que tous les paramètres à droite aient étés trouvés
Param: Identifier { strcpy($$, $1); st_create_scalar($1); };

Statements:
  Statement Semicolon Statements { $$ = make_block_node($1, $3); }
| Block Statements               { $$ = make_block_node($1, $2); }
| %empty                         { $$ = NULL; }
;

Statement:
  Expr                                                                  { $$ = $1; }
| Var Identifier                                                        { st_create_scalar($2); $$ = NOP; }
| Var Identifier Assign Expr                                            { st_create_scalar($2); $$ = create_assign_scalar_node($2, $4); }
| Var Identifier LeftSquareBracket Int RightSquareBracket               { st_create_array($2, $4); $$ = NOP; }
| Var Identifier Assign IntArray                                        { st_create_array($2, $4.len); $$ = create_assign_int_list_node($2, $4); }
| Var Identifier Assign LeftSquareBracket Identifier RightSquareBracket { symbol dst = st_find_or_yyerror($5); st_create_array($2, dst.size); $$ = create_assign_array_node($2, $5); }

| Read Identifier                                                       { $$ = create_read_node($2); }
| Read Identifier LeftSquareBracket IntExpr RightSquareBracket          { $$ = create_read_indexed_node($2, $4); }
| Read LeftSquareBracket Int RightSquareBracket Identifier              { st_create_array($5, $3); $$ = create_read_array_node($5); }
| Read LeftSquareBracket Identifier RightSquareBracket                  { $$ = create_read_array_node($3); }

| Identifier Assign Expr                                                { $$ = create_assign_scalar_node($1, $3); }
| Identifier Assign IntArray                                            { $$ = create_assign_int_list_node($1, $3); }
| Identifier Assign LeftSquareBracket Identifier RightSquareBracket     { $$ = create_assign_array_node($1, $4); }
| Identifier LeftSquareBracket IntExpr RightSquareBracket Assign Expr   { $$ = create_assign_indexed_node($1, $3, $6); }

| Print Expr                                                            { $$ = create_print_node($2); }
| Print LeftSquareBracket Identifier RightSquareBracket                 { $$ = create_print_array_node($3); }

| Return Expr                                                           { $$ = create_return_node($2); }
| Return                                                                { $$ = create_return_node(NULL); }
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
| Identifier Dot Identifier LeftParenthesis RightParenthesis { $$ = create_methodcall_node($1, $3); }
| Identifier LeftParenthesis Args RightParenthesis           { $$ = create_fncall_node($1, $3); }
;

Args:
  IntExpr CommaArgs { $$ = asa_list_append($1, $2); }
| %empty            { $$ = asa_list_empty(); }
;

CommaArgs:
  Comma IntExpr CommaArgs { $$ = asa_list_append($2, $3); }
| %empty                  { $$ = asa_list_empty(); }
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

void arc_compile_file(const char *_infile, const char *_outfile) {
    extern FILE *yyin;
    extern int yylex_destroy(void);

    infile = _infile;
    outfile = fopen(_outfile, "w");
    if(!outfile) {
        fprintf(stderr, "%s: %s\n", _outfile, strerror(errno));
        exit(1);
    }
    
    FILE *f = fopen(_infile, "r");
    if(!f) {
        fprintf(stderr, "%s: %s\n", _infile, strerror(errno));
        exit(1);
    }
    
    st_pop_push_empty();

    yyin = f;
    yyparse();
    yylex_destroy();
    fclose(f);
    fclose(outfile);
}

