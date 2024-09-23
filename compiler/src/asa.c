#include "asa.h"

/**
 * Renvoie `1` si l'étiquette spécifiée est une feuille, sinon
 * renvoie `0`.
 */
int is_leaf(NodeTag tag) {
	switch(tag) {
		case TagInt:
		case TagVar:
			return 1;
		
		case TagIndex:
		case TagBinaryOp:
		case TagUnaryOp:
		case TagAssignScalar:
		case TagAssignIndexed:
		case TagAssignIntList:
		case TagAssignArray:
		case TagTest:
		case TagWhile:
		case TagRead:
		case TagReadIndexed:
		case TagReadArray:
		case TagPrint:
		case TagPrintArray:
		case TagBlock:
		case TagFn:
		case TagFnCall:
		case TagReturn:
			return 0;
	}
	
	fprintf(stderr, "entered unreachable code\n");
	exit(1);
}

/**
 * Renvoie le symbole associé à un opérateur binaire.
 */
const char* binop_symbol(BinaryOp binop) {
	switch(binop) {
		case OpAdd:
			return "+";
		
		case OpSub:
			return "-";
		
		case OpMul:
			return "*";
		
		case OpDiv:
			return "/";
		
		case OpMod:
			return "%";
		
		case OpGe:
			return ">=";
		
		case OpGt:
			return ">";
		
		case OpLe:
			return "<=";
		
		case OpLt:
			return "<";
		
		case OpEq:
			return "==";
		
		case OpNe:
			return "!=";
		
		case OpAnd:
			return "ET";
		
		case OpOr:
			return "OU";
		
		case OpXor:
			return "OU EXCLUSIF";
	}
	
	fprintf(stderr, "entered unreachable code\n");
	exit(1);
}

/**
 * Renvoie le type d'un opérateur binaire.
 */
OpKind binop_kind(BinaryOp binop) {
	switch(binop) {
		case OpAdd:
		case OpSub:
		case OpMul:
		case OpDiv:
		case OpMod:
			return Arithmetic;
		
		case OpGe:
		case OpGt:
		case OpLe:
		case OpLt:
		case OpEq:
		case OpNe:
			return Comparative;
		
		case OpAnd:
		case OpOr:
		case OpXor:
			return Logic;
	}
	
	fprintf(stderr, "entered unreachable code\n");
	exit(1);
}

/**
 * Renvoie le symbole associé à un opérateur unaire.
 */
const char* unop_symbol(UnaryOp unop) {
	switch(unop) {
		case OpNeg:
			return "-";
		
		case OpNot:
			return "NON";
	}
	
	fprintf(stderr, "entered unreachable code\n");
	exit(1);
}

/**
 * Créer une nouvelle liste à partir de son premier élément et des éléments suivants.
 */
asa_list asa_list_append(asa *head, asa_list next) {
	if(next.is_nop) {
		free_asa(head);
		
		return next;
	}
	else if(head == NOP) {
		asa_list_destroy(next);
		
		// `asa_list_destroy()` met `is_nop` sur `1`,
		// et tous les autres champs sur `0` après désallocation
		return next;
	}
	
	if(!head) {
		fprintf(stderr, "called `asa_list_append()` with null `head`\n");
		exit(1);
	}
	
	asa_list l;
	l.len = 1 + next.len;
	l.ninst = head->ninst + next.ninst;
	l.is_nop = 0;
	
	asa_list_node *n = malloc(sizeof(asa_list_node));
	n->value = head;
	n->next = next.head;
	
	l.head = n;
	return l;
}

/**
 * Créer une nouvelle liste vide.
 */
asa_list asa_list_empty() {
	asa_list l;
	l.len = 0;
	l.ninst = 0;
	l.head = NULL;
	l.is_nop = 0;
	
	return l;
}

/**
 * Affiche une liste dans un fichier.
 */
void asa_list_fprint(FILE *stream, asa_list l) {
	if(l.is_nop) {
		fprintf(stream, "NoOp");
	}
	else if(l.len == 0) {
		fprintf(stream, "{}");
	}
	else {
		asa_list_node *n = l.head;
		fprintf(stream, "{ ");
		fprint_asa(stream, n->value);
		
		while(n->next) {
			n = n->next;
			
			fprintf(stream, ", ");
			fprint_asa(stream, n->value);
		}
		
		fprintf(stream, " }");
	}
}

/**
 * Libère les ressources allouées à une liste.
 */
void asa_list_destroy(asa_list l) {
	asa_list_node *n = l.head;
	while(n) {
		asa_list_node *m = n;
		n = n->next;
		
		free_asa(m->value);
		free(m);
	}
	
	l.len = 0;
	l.ninst = 0;
	l.head = NULL;
	l.is_nop = 1;
}

/**
 * Créer une nouvelle liste à partir de son premier élément et de ses éléments suivants.
 */
id_list id_list_append(const char id[32], id_list next) {
	id_list l;
	l.len = 1 + next.len;
	
	id_list_node *n = malloc(sizeof(id_list_node));
	strcpy(&n->value[0], &id[0]);
	n->next = next.head;
	
	l.head = n;
	return l;
}

/**
 * Créer une nouvelle liste vide.
 */
id_list id_list_empty() {
	id_list l;
	l.len = 0;
	l.head = NULL;
	
	return l;
}

/**
 * Affiche une liste dans la sortie standard.
 */
void id_list_fprint(FILE *stream, id_list l) {
	if(l.len == 0) {
		fprintf(stream, "()");
	}
	else {
		id_list_node *n = l.head;
		fprintf(stream, "(%s", n->value);
		
		while(n->next) {
			n = n->next;
			
			fprintf(stream, ", %s", n->value);
		}
		
		fprintf(stream, ")");
	}
}

/**
 * Libère les ressources allouées à une liste.
 */
void id_list_destroy(id_list l) {
	id_list_node *n = l.head;
	while(n) {
		id_list_node *m = n;
		n = n->next;
		
		free(m);
	}
	
	l.len = 0;
	l.head = NULL;
}

asa* checked_malloc() {
	asa *p = malloc(sizeof(asa));
	if(!p) {
		fprintf(stderr, "échec d'allocation mémoire\n");
		exit(1);
	}
	
	if(p == NOP) {
		fprintf(stderr, "`malloc()` returned `NOP`\n");
		exit(1);
	}
	
	return p;
}

/**
 * Créer une nouvelle feuille `TagInt` avec la valeur spécifiée.
 */
asa* create_int_leaf(int value) {
	asa *p = checked_malloc();
	
	p->tag = TagInt;
	p->ninst = 1;
	p->tag_int.value = value;
	
	return p;
}

/**
 * Créer une nouvelle feuille `TagVar` avec l'identifiant spécifié.
 */
asa* create_var_leaf(const char id[32]) {
	symbol var = st_find_or_yyerror(id);
	if(var.size != SCALAR_SIZE) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation requise: '%s' est un tableau, un scalaire était attendu\n", infile, yylineno, id);
		exit(1);
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagVar;
	p->ninst = 3;
	strcpy(&p->tag_var.identifier[0], &id[0]);
	
	return p;
}

/**
 * Créer un nouveau noeud `TagIndex` avec les valeurs spécifiées.
 */
asa* create_index_node(const char id[32], asa *index) {
	symbol var = st_find_or_yyerror(id);
	if(var.size == SCALAR_SIZE) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' est un scalaire\n", infile, yylineno, id);
		exit(1);
	}
	else if(var.size == 0) {
		free_asa(index);
		
		return NOP;
	}
	else if(index == NOP) {
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagIndex;
	p->ninst = 3 + (index->tag != TagInt ? index->ninst : 0);
	strcpy(&p->tag_index.identifier[0], &id[0]);
	p->tag_index.index = index;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagBinaryOp` avec les valeurs spécifiées.
 */
asa* create_binop_node(BinaryOp binop, asa *lhs, asa *rhs) {
	if(lhs == NOP || rhs == NOP) {
		free_asa(lhs);
		free_asa(rhs);
		
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagBinaryOp;
	
	switch(binop) {
		case OpAdd:
		case OpSub:
		case OpMul:
		case OpDiv:
		case OpMod:
			p->ninst = lhs->ninst + rhs->ninst + 4;
			break;
		
		case OpGe:
		case OpGt:
		case OpLe:
		case OpLt:
		case OpEq:
		case OpNe:
			p->ninst = lhs->ninst + rhs->ninst + 8;
			break;
		
		case OpAnd:
			p->ninst = lhs->ninst + rhs->ninst + 3;
			break;
		
		case OpOr:
			p->ninst = lhs->ninst + rhs->ninst + 4;
			break;
		
		case OpXor:
			p->ninst = lhs->ninst + rhs->ninst + 10;
			break;
	}
	
	p->tag_binary_op.op = binop;
	p->tag_binary_op.lhs = lhs;
	p->tag_binary_op.rhs = rhs;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagUnaryOp` avec les valeurs spécifiées.
 */
asa* create_unop_node(UnaryOp unop, asa *expr) {
	if(expr == NOP) {
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagUnaryOp;
	p->ninst = expr->ninst + (unop == OpNeg ? 3 : 4);
	p->tag_unary_op.op = unop;
	p->tag_unary_op.expr = expr;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagAssignScalar` avec les valeurs spécifiées.
 */
asa* create_assign_scalar_node(const char id[32], asa *expr) {
	symbol var = st_find_or_yyerror(id);
	if(var.size != SCALAR_SIZE) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: impossible d'affecter un scalaire à un tableau\n", infile, yylineno);
		exit(1);
	}
	else if(expr == NOP) {
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagAssignScalar;
	p->ninst = expr->ninst + 6;
	strcpy(&p->tag_assign_scalar.identifier[0], &id[0]);
	p->tag_assign_scalar.expr = expr;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagAssignIndexed` avec les valeurs spécifiées.
 */
asa* create_assign_indexed_node(const char id[32], asa *index, asa *expr) {
	symbol var = st_find_or_yyerror(id);
	if(var.size == SCALAR_SIZE) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' est un scalaire\n", infile, yylineno, id);
		exit(1);
	}
	else if(var.size == 0) {
		return NOP;
	}
	else if(index == NOP || expr == NOP) {
		free_asa(index);
		free_asa(expr);
		
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagAssignIndexed;
	p->ninst = index->ninst + expr->ninst + 8;
	strcpy(&p->tag_assign_indexed.identifier[0], &id[0]);
	p->tag_assign_indexed.index = index;
	p->tag_assign_indexed.expr = expr;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagAssignIntList` avec les valeurs spécifiées.
 */
asa* create_assign_int_list_node(const char id[32], asa_list values) {
	symbol var = st_find_or_yyerror(id);
	if(var.size == SCALAR_SIZE) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: impossible d'affecter un tableau au scalaire '%s'\n", infile, yylineno, id);
		exit(1);
	}
	else if((size_t) var.size != values.len) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: affectation impossible: le tableau n'a pas la taille adéquate\n", infile, yylineno);
		exit(1);
	}
	
	if(values.len == 0 || values.is_nop) {
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagAssignIntList;
	p->ninst = 3 + values.ninst + values.len * 2;
	strcpy(&p->tag_assign_int_list.identifier[0], &id[0]);
	p->tag_assign_int_list.values = values;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagAssignArray` avec les valeurs spécifiées.
 */
asa* create_assign_array_node(const char dst[32], const char src[32]) {
	symbol dst_var = st_find_or_yyerror(dst);
	symbol src_var = st_find_or_yyerror(src);
	
	if(src_var.size == SCALAR_SIZE) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: '%s' doit être un tableau\n", infile, yylineno, src);
		exit(1);
	}
	else if(dst_var.size == SCALAR_SIZE) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: impossible d'affecter un tableau à un scalaire\n", infile, yylineno);
		exit(1);
	}
	else if(src_var.size != dst_var.size) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: affectation impossible: les deux tableaux doivent avoir la même taille\n", infile, yylineno);
		exit(1);
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagAssignArray;
	p->ninst = 3 + dst_var.size * 5;
	strcpy(&p->tag_assign_array.dst[0], &dst[0]);
	strcpy(&p->tag_assign_array.src[0], &src[0]);
	
	return p;
}

/**
 * Créer un nouveau noeud `TagTest` avec les valeurs spécifiées.
 */
asa* create_test_node(asa *expr, asa *therefore, asa *alternative) {
	if(!therefore && !alternative) {
		free_asa(expr);
		
		return NOP;
	}
	else if(expr == NOP) {
		free_asa(therefore);
		free_asa(alternative);
		
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagTest;
	p->ninst = expr->ninst + 1 + (therefore ? therefore->ninst : 0) + (alternative ? (4 + alternative->ninst) : 2);
	p->tag_test.expr = expr;
	p->tag_test.therefore = therefore;
	p->tag_test.alternative = alternative;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagWhile` avec les valeurs spécifiées.
 */
asa* create_while_node(asa *expr, asa *body) {
	if(!body) {
		free_asa(expr);
		
		return NOP;
	}
	else if(expr == NOP) {
		free_asa(body);
		
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagWhile;
	p->ninst = expr->ninst + body->ninst + 2;
	p->tag_while.expr = expr;
	p->tag_while.body = body;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagRead` avec l'identifiant spécifié.
 */
asa* create_read_node(const char id[32]) {	
	st_find_or_create_scalar(id);
	
	asa *p = checked_malloc();
	
	p->tag = TagRead;
	p->ninst = 5;
	strcpy(&p->tag_read.identifier[0], &id[0]);
	
	return p;
}

/**
 * Créer un nouveau noeud `TagReadIndexed` avec les valeurs spécifiées.
 */
asa* create_read_indexed_node(const char id[32], asa *index) {
	symbol var = st_find_or_yyerror(id);
	if(var.size == SCALAR_SIZE) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' est un scalaire\n", infile, yylineno, id);
		exit(1);
	}
	else if(var.size == 0) {
		return NOP;
	}
	else if(index == NOP) {
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagReadIndexed;
	p->ninst = index->ninst + 7;
	strcpy(&p->tag_read_indexed.identifier[0], &id[0]);
	p->tag_read_indexed.index = index;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagReadArray` avec l'identifiant spécifié.
 */
asa* create_read_array_node(const char id[32]) {
	symbol var = st_find_or_yyerror(id);
	if(var.size == -1) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' est un scalaire\n", infile, yylineno, id);
		exit(1);
	}
	else if(var.size == 0) {
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagReadArray;
	p->ninst = 3 + 3 * var.size;
	strcpy(&p->tag_read_array.identifier[0], &id[0]);
	
	return p;
}

/**
 * Créer un nouveau noeud `TagPrint` avec l'expression spécifiée.
 */
asa* create_print_node(asa *expr) {
	if(expr == NOP) {
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagPrint;
	p->ninst = expr->ninst + 1;
	p->tag_print.expr = expr;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagPrintArray` avec l'identifiant spécifié.
 */
asa* create_print_array_node(const char id[32]) {
	symbol var = st_find_or_internal_error(id);
	if(var.size == -1) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' est un scalaire\n", infile, yylineno, id);
		exit(1);
	}
	else if(var.size == 0) {
		return NOP;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagPrintArray;
	p->ninst = 3 + 3 * var.size;
	strcpy(&p->tag_print_array.identifier[0], &id[0]);
	
	return p;
}

/**
 * Transforme deux noeuds en un noeud `TagBlock` équivalent.
 */
asa* make_block_node(asa *p, asa *q) {
	if(p == NOP) {
		p = NULL;
	}
	
	if(q == NOP) {
		q = NULL;
	}
	
	if(!p && !q) {
		return NULL;
	}
	
	if(!p && q) {
		// swap(p, q)
		p = q;
		q = NULL;
	}
	
	// p is now nonnull
	
	if(p->tag == TagBinaryOp && p->tag_binary_op.op == OpEq) {
	    extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: erreur: test d'égalité inutilisé\n", infile, yylineno);
		exit(1);
	}
	
	asa *qBlock = NULL;
	if(q) {
		if(q->tag == TagBlock) {
			qBlock = q;
		}
		else {
			qBlock = checked_malloc();
			
			qBlock->tag = TagBlock;
			qBlock->ninst = q->ninst + 1;
			qBlock->tag_block.stmt = q;
			qBlock->tag_block.next = NULL;
		}
	}
	
	if(p->tag != TagBlock) {
		asa *r = checked_malloc();
		
		r->tag = TagBlock;
		r->ninst = p->ninst + (qBlock ? qBlock->ninst : 0) + 1;
		r->tag_block.stmt = p;
		r->tag_block.next = qBlock;
		
		return r;
	}
	else {
		if(qBlock) {
			p->ninst += qBlock->ninst;
			
			asa *r = p;
			while(r->tag_block.next) {
				r = r->tag_block.next;
			}
			
			r->tag_block.next = qBlock;
		}
		
		return p;
	}
}

/**
 * Créer un nouveau noeud correspondant à la méthode spécifiée.
 */
asa* create_methodcall_node(const char varname[32], const char methodname[32]) {
	symbol var = st_find_or_yyerror(varname);
	
	if(strcmp(methodname, "len") != 0) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: seule la méthode intrinsèque 'len()' est actuellement acceptée\n", infile, yylineno);
		exit(1);
	}
	
	if(var.size == SCALAR_SIZE) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: 'len()' n'est pas disponible sur les scalaires\n", infile, yylineno);
		exit(1);
	}
	
	return create_int_leaf(var.size);
}

/**
 * Créer un nouveau noeud `TagFn` avec les valeurs spécifiées.
 */
asa* create_fn_node(const char id[32], id_list params, asa *body, symbol_table *st) {
	asa *p = checked_malloc();
	
	p->tag = TagFn;
	p->ninst = 7 + ((body && body != NOP) ? body->ninst : 0);
	strcpy(&p->tag_fn.identifier[0], &id[0]);
	p->tag_fn.params = params;
	p->tag_fn.body = body;
	p->tag_fn.st = st;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagFnCall` avec les paramètres spécifiés.
 */
asa* create_fncall_node(const char id[32], asa_list args) {
	// l'existence de la fonction sera vérifiée pendant
	// la génération de code
	
	asa *p = checked_malloc();
	
	p->tag = TagFnCall;
	p->ninst = 17 + args.ninst + args.len * 6;
	strcpy(&p->tag_fn_call.identifier[0], &id[0]);
	p->tag_fn_call.args = args;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagReturn` avec l'expression spécifiée.
 */
asa* create_return_node(asa *expr) {
	asa *p = checked_malloc();
	
	p->tag = TagReturn;
	p->ninst = 4 + ((expr && expr != NOP) ? expr->ninst : 1);
	p->tag_return.expr = expr;
	
	return p;
}

/**
 * Affiche le noeud dans un fichier.
 */
void fprint_asa(FILE *stream, asa *p) {
	if(!p) {
		return;
	}
	
	if(p == NOP) {
		fprintf(stream, "NoOp");
		return;
	}
	
	switch(p->tag) {
		case TagInt:
			fprintf(stream, "%i", p->tag_int.value);
			break;
		
		case TagVar:
			fprintf(stream, "%s", p->tag_var.identifier);
			break;
		
		case TagIndex:
			fprintf(stream, "%s[", p->tag_index.identifier);
			fprint_asa(stream, p->tag_index.index);
			fprintf(stream, "]");
			break;
		
		case TagBinaryOp:
			if(is_leaf(p->tag_binary_op.lhs->tag)) {
				fprint_asa(stream, p->tag_binary_op.lhs);
			}
			else {
				fprintf(stream, "(");
				fprint_asa(stream, p->tag_binary_op.lhs);
				fprintf(stream, ")");
			}
			
			fprintf(stream, " %s ", binop_symbol(p->tag_binary_op.op));
			
			if(is_leaf(p->tag_binary_op.rhs->tag)) {
				fprint_asa(stream, p->tag_binary_op.rhs);
			}
			else {
				fprintf(stream, "(");
				fprint_asa(stream, p->tag_binary_op.rhs);
				fprintf(stream, ")");
			}
			
			break;
		
		case TagUnaryOp:
			fprintf(stream, "%s", unop_symbol(p->tag_unary_op.op));
			
			if(is_leaf(p->tag_unary_op.expr->tag)) {
				fprint_asa(stream, p->tag_unary_op.expr);
			}
			else {
				fprintf(stream, "(");
				fprint_asa(stream, p->tag_unary_op.expr);
				fprintf(stream, ")");
			}
			
			break;
		
		case TagAssignScalar:
			fprintf(stream, "%s := ", p->tag_assign_scalar.identifier);
			fprint_asa(stream, p->tag_assign_scalar.expr);
			break;
		
		case TagAssignIndexed:
			fprintf(stream, "%s[", p->tag_assign_indexed.identifier);
			fprint_asa(stream, p->tag_assign_indexed.index);
			fprintf(stream, "] := ");
			fprint_asa(stream, p->tag_assign_indexed.expr);
			break;
		
		case TagAssignIntList:
			fprintf(stream, "%s := ", p->tag_assign_int_list.identifier);
			asa_list_fprint(stream, p->tag_assign_int_list.values);
			break;
		
		case TagAssignArray:
			fprintf(stream, "%s := [%s]", p->tag_assign_array.dst, p->tag_assign_array.src);
			break;
		
		case TagTest:
			fprintf(stream, "SI ");
			fprint_asa(stream, p->tag_test.expr);
			break;
		
		case TagWhile:
			fprintf(stream, "TQ ");
			fprint_asa(stream, p->tag_test.expr);
			break;
		
		case TagRead:
			fprintf(stream, "LIRE %s", p->tag_read.identifier);
			break;
		
		case TagReadIndexed:
			fprintf(stream, "LIRE %s[", p->tag_read_indexed.identifier);
			fprint_asa(stream, p->tag_read_indexed.index);
			fprintf(stream, "]");
			break;
		
		case TagReadArray: {
			symbol var = st_find_or_internal_error(p->tag_read_array.identifier);
			
			fprintf(stream, "LIRE[%i] %s", var.size, var.identifier);
			break;
		}
		
		case TagPrint:
			fprintf(stream, "AFFICHER ");
			fprint_asa(stream, p->tag_print.expr);
			break;
		
		case TagPrintArray:
			fprintf(stream, "AFFICHER [%s]", p->tag_print_array.identifier);
			break;
		
		case TagBlock:
		    fprint_asa(stream, p->tag_block.stmt);
			fprintf(stream, "\n");
		    fprint_asa(stream, p->tag_block.next);
			break;
		
		case TagFn:
			fprintf(stream, "FONCTION %s", p->tag_fn.identifier);
			id_list_fprint(stream, p->tag_fn.params);
			break;
		
		case TagFnCall:
			fprintf(stream, "%s", p->tag_fn_call.identifier);
			asa_list_fprint(stream, p->tag_fn_call.args);
			break;
		
		case TagReturn:
			fprintf(stream, "RENVOYER ");
			fprint_asa(stream, p->tag_return.expr);
			break;
	}
}

/**
 * Libère les ressources allouées à un noeud.
 */
void free_asa(asa *p) {
	if(!p) {
		return;
	}
	
	if(p == NOP) {
		return;
	}
	
	switch(p->tag) {
		case TagIndex:
			free_asa(p->tag_index.index);
			break;
		
		case TagBinaryOp:
			free_asa(p->tag_binary_op.lhs);
			free_asa(p->tag_binary_op.rhs);
			break;
		
		case TagUnaryOp:
			free_asa(p->tag_unary_op.expr);
			break;
		
		case TagAssignScalar:
			free_asa(p->tag_assign_scalar.expr);
			break;
		
		case TagAssignIndexed:
			free_asa(p->tag_assign_indexed.index);
			free_asa(p->tag_assign_indexed.expr);
			break;
		
		case TagAssignIntList:
			asa_list_destroy(p->tag_assign_int_list.values);
			break;
		
		case TagTest:
			free_asa(p->tag_test.expr);
			free_asa(p->tag_test.therefore);
			free_asa(p->tag_test.alternative);
			break;
		
		case TagWhile:
			free_asa(p->tag_while.expr);
			free_asa(p->tag_while.body);
			break;
		
		case TagReadIndexed:
			free_asa(p->tag_read_indexed.index);
			break;
		
		case TagPrint:
			free_asa(p->tag_print.expr);
			break;
		
		case TagBlock:
			free_asa(p->tag_block.stmt);
			free_asa(p->tag_block.next);
			break;
		
		case TagFn:
			id_list_destroy(p->tag_fn.params);
			free_asa(p->tag_fn.body);
			st_destroy(p->tag_fn.st);
			break;
		
		case TagFnCall:
			asa_list_destroy(p->tag_fn_call.args);
			break;
		
		case TagReturn:
			free_asa(p->tag_return.expr);
			break;
		
		case TagInt:
		case TagVar:
		case TagAssignArray:
		case TagRead:
		case TagReadArray:
		case TagPrintArray:
			break;
	}
	
	free(p);
}
