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

asa* checked_malloc() {
	asa *p = malloc(sizeof(asa));
	if(!p) {
		fprintf(stderr, "échec d'allocation mémoire");
		exit(1);
	}
	
	return p;
}

/**
 * Créer une nouvelle liste à partir de son premier élément et des éléments suivants.
 */
asa_list asa_list_append(asa *head, asa_list next) {
	if(!head) {
		fprintf(stderr, "called asa_list_append() with null `head`\n");
		exit(1);
	}
	
	asa_list l;
	l.len = 1 + next.len;
	l.ninst = head->ninst + next.ninst;
	
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
	
	return l;
}

/**
 * Affiche une liste dans la sortie standard.
 */
void asa_list_print(asa_list l) {
	if(l.len == 0) {
		printf("{}");
	}
	else {
		asa_list_node *n = l.head;
		printf("{ ");
		print_asa(n->value);
		
		while(n->next) {
			n = n->next;
			
			printf(", ");
			print_asa(n->value);
		}
		
		printf(" }");
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
		
		free(m);
	}
	
	l.len = 0;
	l.head = NULL;
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
	ts *var = ts_retrouver_id(id);
	if(var == NULL) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: variable inconnue: '%s'\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size != -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation requise: '%s' est un tableau, un scalaire était attendu\n", input, yylineno, id);
		exit(1);
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagVar;
	p->ninst = 1;
	strcpy(&p->tag_var.identifier[0], &id[0]);
	
	return p;
}

/**
 * Créer un nouveau noeud `TagIndex` avec les valeurs spécifiées.
 */
asa* create_index_node(const char id[32], asa *index) {
	ts *var = ts_retrouver_id(id);
	if(var == NULL) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: variable inconnue: '%s'\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size == -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' est un scalaire\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size == 0) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' n'occupe pas d'espace\n", input, yylineno, id);
		exit(1);
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagIndex;
	p->ninst = index->tag == TagInt ? 1 : index->ninst + 2;
	strcpy(&p->tag_index.identifier[0], &id[0]);
	p->tag_index.index = index;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagBinaryOp` avec les valeurs spécifiées.
 */
asa* create_binop_node(BinaryOp binop, asa *lhs, asa *rhs) {
	asa *p = checked_malloc();
	
	p->tag = TagBinaryOp;
	
	switch(binop) {
		case OpAdd:
		case OpSub:
		case OpMul:
		case OpDiv:
		case OpMod:
			p->ninst = lhs->ninst + rhs->ninst + 2;
			break;
		
		case OpGe:
		case OpGt:
		case OpLe:
		case OpLt:
		case OpEq:
		case OpNe:
			p->ninst = lhs->ninst + rhs->ninst + 6;
			break;
		
		case OpAnd:
			p->ninst = lhs->ninst + rhs->ninst + 3;
			break;
		
		case OpOr:
			p->ninst = lhs->ninst + rhs->ninst + 4;
			break;
		
		case OpXor:
			p->ninst = lhs->ninst + rhs->ninst + 7;
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
	asa *p = checked_malloc();
	
	p->tag = TagUnaryOp;
	p->ninst = expr->ninst + 3;
	p->tag_unary_op.op = unop;
	p->tag_unary_op.expr = expr;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagAssignScalar` avec les valeurs spécifiées.
 */
asa* create_assign_scalar_node(const char id[32], asa *expr) {
	ts *var = ts_retrouver_id(id);
	if(var == NULL) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: variable inconnue: '%s'\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size != -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: impossible d'affecter un scalaire à un tableau\n", input, yylineno);
		exit(1);
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagAssignScalar;
	p->ninst = expr->ninst + 1;
	strcpy(&p->tag_assign_scalar.identifier[0], &id[0]);
	p->tag_assign_scalar.expr = expr;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagAssignIndexed` avec les valeurs spécifiées.
 */
asa* create_assign_indexed_node(const char id[32], asa *index, asa *expr) {
	ts *var = ts_retrouver_id(id);
	if(var == NULL) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: variable inconnue: '%s'\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size == -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' est un scalaire\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size == 0) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' n'occupe pas d'espace\n", input, yylineno, id);
		exit(1);
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagAssignIndexed;
	p->ninst = index->ninst + expr->ninst + 3;
	strcpy(&p->tag_assign_indexed.identifier[0], &id[0]);
	p->tag_assign_indexed.index = index;
	p->tag_assign_indexed.expr = expr;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagAssignIntList` avec les valeurs spécifiées.
 */
asa* create_assign_int_list_node(const char id[32], asa_list values) {
	ts *var = ts_retrouver_id(id);
	if(var == NULL) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: variable inconnue: '%s'\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size == -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: impossible d'affecter un tableau au scalaire '%s'\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size != values.len) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: affectation impossible: le tableau n'a pas la taille adéquate\n", input, yylineno);
		exit(1);
	}
	
	if(values.len == 0) {
		return NULL;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagAssignIntList;
	p->ninst = values.ninst + values.len;
	strcpy(&p->tag_assign_int_list.identifier[0], &id[0]);
	p->tag_assign_int_list.values = values;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagAssignArray` avec les valeurs spécifiées.
 */
asa* create_assign_array_node(const char dst[32], const char src[32]) {
	ts *dst_var = ts_retrouver_id(dst);
	if(dst_var == NULL) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: variable inconnue: '%s'\n", input, yylineno, dst);
		exit(1);
	}
	
	ts *src_var = ts_retrouver_id(src);
	if(src_var == NULL) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: variable inconnue: '%s'\n", input, yylineno, src);
		exit(1);
	}
	else if(src_var->size == -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: '%s' doit être un tableau\n", input, yylineno, src);
		exit(1);
	}
	else if(dst_var->size == -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: impossible d'affecter un tableau à un scalaire\n", input, yylineno);
		exit(1);
	}
	else if(src_var->size != dst_var->size) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: affectation impossible: les deux tableaux doivent avoir la même taille\n", input, yylineno);
		exit(1);
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagAssignArray;
	p->ninst = dst_var->size * 2;
	strcpy(&p->tag_assign_array.dst[0], &dst[0]);
	strcpy(&p->tag_assign_array.src[0], &src[0]);
	
	return p;
}

/**
 * Créer un nouveau noeud `TagTest` avec les valeurs spécifiées.
 */
asa* create_test_node(asa *expr, asa *therefore, asa *alternative) {
	if(!therefore && !alternative) {
		return NULL;
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
		return NULL;
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
	ts *var = ts_retrouver_id(id);
	if(var == NULL) {
		fprintf(stderr, "illegal state: '%s' should exists at this stage but it does not\n", id);
		exit(1);
	}
	else if(var->size != -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation requise: '%s' est un tableau\n", input, yylineno, id);
		exit(1);
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagRead;
	p->ninst = 2;
	strcpy(&p->tag_read.identifier[0], &id[0]);
	
	return p;
}

/**
 * Créer un nouveau noeud `TagReadIndexed` avec les valeurs spécifiées.
 */
asa* create_read_indexed_node(const char id[32], asa *index) {
	ts *var = ts_retrouver_id(id);
	if(var == NULL) {
		fprintf(stderr, "illegal state: '%s' should exists at this stage but it does not\n", id);
		exit(1);
	}
	else if(var->size == -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' est un scalaire\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size == 0) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' n'occupe pas d'espace\n", input, yylineno, id);
		exit(1);
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagReadIndexed;
	p->ninst = index->ninst + 4;
	strcpy(&p->tag_read_indexed.identifier[0], &id[0]);
	p->tag_read_indexed.index = index;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagReadArray` avec l'identifiant spécifié.
 */
asa* create_read_array_node(const char id[32]) {
	ts *var = ts_retrouver_id(id);
	if(var == NULL) {
		fprintf(stderr, "illegal state: '%s' should exists at this stage but it does not\n", id);
		exit(1);
	}
	else if(var->size == -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' est un scalaire\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size == 0) {
		return NULL;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagReadArray;
	p->ninst = 2 * var->size;
	strcpy(&p->tag_read_array.identifier[0], &id[0]);
	
	return p;
}

/**
 * Créer un nouveau noeud `TagPrint` avec l'expression spécifiée.
 */
asa* create_print_node(asa *expr) {
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
	ts *var = ts_retrouver_id(id);
	if(var == NULL) {
		fprintf(stderr, "illegal state: '%s' should exists at this stage but it does not\n", id);
		exit(1);
	}
	else if(var->size == -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: indexation impossible: '%s' est un scalaire\n", input, yylineno, id);
		exit(1);
	}
	else if(var->size == 0) {
		return NULL;
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagPrintArray;
	p->ninst = 2 * var->size;
	strcpy(&p->tag_print_array.identifier[0], &id[0]);
	
	return p;
}

/**
 * Transforme deux noeuds en un noeud `TagBlock` équivalent.
 */
asa* make_block_node(asa *p, asa *q) {
	if(!p && !q) {
		return NULL;
	}
	
	if(!p && q) {
		// swap(p, q)
		p = q;
		q = NULL;
	}
	
	// p is now nonnull
	
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
asa* create_fncall_node(const char varname[32], const char methodname[32]) {
	ts *var = ts_retrouver_id(varname);
	if(var == NULL) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: variable inconnue: '%s'\n", input, yylineno, varname);
		exit(1);
	}
	
	if(strcmp(methodname, "len") != 0) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: seule la méthode intrinsèque 'len()' est actuellement acceptée\n", input, yylineno);
		exit(1);
	}
	
	if(var->size == -1) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: 'len()' n'est pas disponible sur les scalaires\n", input, yylineno);
		exit(1);
	}
	
	return create_int_leaf(var->size);
}

/**
 * Affiche le noeud dans la sortie standard.
 */
void print_asa(asa *p) {
	if(!p) {
		return;
	}
	
	switch(p->tag) {
		case TagInt:
			printf("%i", p->tag_int.value);
			break;
		
		case TagVar:
			printf("%s", p->tag_var.identifier);
			break;
		
		case TagIndex:
			printf("%s[", p->tag_index.identifier);
			print_asa(p->tag_index.index);
			printf("]");
			break;
		
		case TagBinaryOp:
			if(is_leaf(p->tag_binary_op.lhs->tag)) {
				print_asa(p->tag_binary_op.lhs);
			}
			else {
				printf("(");
				print_asa(p->tag_binary_op.lhs);
				printf(")");
			}
			
			printf(" %s ", binop_symbol(p->tag_binary_op.op));
			
			if(is_leaf(p->tag_binary_op.rhs->tag)) {
				print_asa(p->tag_binary_op.rhs);
			}
			else {
				printf("(");
				print_asa(p->tag_binary_op.rhs);
				printf(")");
			}
			
			break;
		
		case TagUnaryOp:
			printf("%s", unop_symbol(p->tag_unary_op.op));
			
			if(is_leaf(p->tag_unary_op.expr->tag)) {
				print_asa(p->tag_unary_op.expr);
			}
			else {
				printf("(");
				print_asa(p->tag_unary_op.expr);
				printf(")");
			}
			
			break;
		
		case TagAssignScalar:
			printf("%s := ", p->tag_assign_scalar.identifier);
			print_asa(p->tag_assign_scalar.expr);
			break;
		
		case TagAssignIndexed:
			printf("%s[", p->tag_assign_indexed.identifier);
			print_asa(p->tag_assign_indexed.index);
			printf("] := ");
			print_asa(p->tag_assign_indexed.expr);
			break;
		
		case TagAssignIntList:
			printf("%s := ", p->tag_assign_int_list.identifier);
			asa_list_print(p->tag_assign_int_list.values);
			break;
		
		case TagAssignArray:
			printf("%s := [%s]", p->tag_assign_array.dst, p->tag_assign_array.src);
			break;
		
		case TagTest:
			printf("SI ");
			print_asa(p->tag_test.expr);
			break;
		
		case TagWhile:
			printf("TQ ");
			print_asa(p->tag_test.expr);
			break;
		
		case TagRead:
			printf("LIRE %s", p->tag_read.identifier);
			break;
		
		case TagReadIndexed:
			printf("LIRE %s[", p->tag_read_indexed.identifier);
			print_asa(p->tag_read_indexed.index);
			printf("]");
			break;
		
		case TagReadArray: {
			ts *var = ts_retrouver_id(p->tag_read_array.identifier);
			if(var == NULL) {
				fprintf(stderr, "illegal state: '%s' should exists at this stage but it does not\n", p->tag_read_array.identifier);
				exit(1);
			}
			
			printf("LIRE[%i] %s", var->size, var->id);
			break;
		}
		
		case TagPrint:
			printf("AFFICHER ");
			print_asa(p->tag_print.expr);
			break;
		
		case TagPrintArray:
			printf("AFFICHER [%s]", p->tag_print_array.identifier);
			break;
		
		case TagBlock:
			print_asa(p->tag_block.stmt);
			printf("\n");
			print_asa(p->tag_block.next);
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
