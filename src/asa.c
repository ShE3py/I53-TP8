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
		case TagAssign:
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
	}
	
	fprintf(stderr, "entered unreachable code\n");
	exit(1);
}

/**
 * Créer une nouvelle feuille `TagInt` avec la valeur spécifiée.
 */
asa* create_int_leaf(int value) {
	asa *p = malloc(sizeof(asa));
	if(!p) {
		yyerror("échec allocation mémoire");
	}
	
	p->tag = TagInt;
	p->ninst = 1;
	p->tag_int.value = value;
	
	return p;
}

/**
 * Créer une nouvelle feuille `TagVar` avec l'identifiant spécifié.
 */
asa* create_var_leaf(const char id[32]) {
	asa *p = malloc(sizeof(asa));
	if(!p) {
		yyerror("échec allocation mémoire");
	}
	
	p->tag = TagVar;
	p->ninst = 1;
	strcpy(&p->tag_var.identifier[0], &id[0]);
	
	return p;
}

/**
 * Créer un nouveau noeud `TagBinaryOp` avec les valeurs spécifiées.
 */
asa* create_binop_node(BinaryOp binop, asa *lhs, asa *rhs) {
	asa *p = malloc(sizeof(asa));
	if(!p) {
		yyerror("échec allocation mémoire");
	}
	
	p->tag = TagBinaryOp;
	p->ninst = lhs->ninst + rhs->ninst + 2;
	p->tag_binary_op.op = binop;
	p->tag_binary_op.lhs = lhs;
	p->tag_binary_op.rhs = rhs;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagAssign` avec les valeurs spécifiées.
 */
asa* create_assign_node(const char id[32], asa *expr) {
	asa *p = malloc(sizeof(asa));
	if(!p) {
		yyerror("échec allocation mémoire");
	}
	
	p->tag = TagAssign;
	p->ninst = expr->ninst + 1;
	strcpy(&p->tag_assign.identifier[0], &id[0]);
	p->tag_assign.expr = expr;
	
	return p;
}

/**
 * Créer un nouveau noeud `TagPrint` avec l'expression spécifiée.
 */
asa* create_print_node(asa *expr) {
	asa *p = malloc(sizeof(asa));
	if(!p) {
		yyerror("échec allocation mémoire");
	}
	
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
			qBlock = malloc(sizeof(asa));
			if(!qBlock) {
				yyerror("échec d'allocation mémoire");
			}
			
			qBlock->tag = TagBlock;
			qBlock->tag_block.stmt = q;
			qBlock->tag_block.next = NULL;
		}
	}
	
	if(p->tag != TagBlock) {
		asa *r = malloc(sizeof(asa));
		if(!r) {
			yyerror("échec allocation mémoire");
		}
		
		r->tag = TagBlock;
		r->tag_block.stmt = p;
		r->tag_block.next = qBlock;
		
		return r;
	}
	else {
		asa *r = p;
		while(r->tag_block.next) {
			r = r->tag_block.next;
		}
		
		r->tag_block.next = qBlock;
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
		
		case TagAssign:
			printf("%s := ", p->tag_assign.identifier);
			print_asa(p->tag_assign.expr);
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
		
		case TagAssign:
			free_asa(p->tag_assign.expr);
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
			break;
	}
	
	free(p);
}

void yyerror(const char * s)
{
  fprintf(stderr, "%s\n", s);
  exit(0);
}
