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
		
		case TagBinaryOp:
		case TagUnaryOp:
		case TagAssign:
		case TagTest:
		case TagRead:
		case TagPrint:
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
	if(ts_retrouver_id(id) == NULL) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: variable inconnue: '%s'\n", input, yylineno, id);
		exit(1);
	}
	
	asa *p = checked_malloc();
	
	p->tag = TagVar;
	p->ninst = 1;
	strcpy(&p->tag_var.identifier[0], &id[0]);
	
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
 * Créer un nouveau noeud `TagAssign` avec les valeurs spécifiées.
 */
asa* create_assign_node(const char id[32], asa *expr) {
	asa *p = checked_malloc();
	
	p->tag = TagAssign;
	p->ninst = expr->ninst + 1;
	strcpy(&p->tag_assign.identifier[0], &id[0]);
	p->tag_assign.expr = expr;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagTest` avec les valeurs spécifiées.
 */
asa* create_test_node(asa *expr, asa *therefore, asa *alternative) {
	asa *p = checked_malloc();
	
	p->tag = TagTest;
	p->ninst = expr->ninst + 1 + therefore->ninst + (alternative ? (4 + alternative->ninst) : 2);
	p->tag_test.expr = expr;
	p->tag_test.therefore = therefore;
	p->tag_test.alternative = alternative;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagRead` avec l'identifiant spécifié.
 */
asa* create_read_node(const char id[32]) {
	asa *p = checked_malloc();
	
	p->tag = TagRead;
	p->ninst = 2;
	strcpy(&p->tag_read.identifier[0], &id[0]);
	
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
		
		case TagAssign:
			printf("%s := ", p->tag_assign.identifier);
			print_asa(p->tag_assign.expr);
			break;
		
		case TagTest:
			printf("SI ");
			print_asa(p->tag_test.expr);
			break;
		
		case TagRead:
			printf("LIRE %s", p->tag_read.identifier);
			break;
		
		case TagPrint:
			printf("AFFICHER ");
			print_asa(p->tag_print.expr);
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
		case TagBinaryOp:
			free_asa(p->tag_binary_op.lhs);
			free_asa(p->tag_binary_op.rhs);
			break;
		
		case TagUnaryOp:
			free_asa(p->tag_unary_op.expr);
			break;
		
		case TagAssign:
			free_asa(p->tag_assign.expr);
			break;
		
		case TagTest:
			free_asa(p->tag_test.expr);
			free_asa(p->tag_test.therefore);
			free_asa(p->tag_test.alternative);
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
		case TagRead:
			break;
	}
	
	free(p);
}
