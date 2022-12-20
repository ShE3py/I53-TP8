#include "asa.h"

asa * creer_feuilleNb(int val)
{
  asa *p;

  if ((p = malloc(sizeof(asa))) == NULL)
    yyerror("echec allocation mémoire");

  p->type = typeNb;
  p->nb.val = val;
  return p;
}

asa * creer_noeudOp( int ope, asa * p1, asa * p2)
{
  asa * p;

  if ((p = malloc(sizeof(asa))) == NULL)
    yyerror("echec allocation mémoire");

  p->type = typeOp;
  p->op.ope = ope;
  p->op.noeud[0]=p1;
  p->op.noeud[1]=p2;
  p->ninst = p1->ninst+p2->ninst+2;
  
  return p;
}

asa* creer_noeudAffect(const char id[32], asa *expr) {
	asa *p = malloc(sizeof(asa));
	if(!p) {
		yyerror("échec allocation mémoire");
	}
	
	p->type = typeAffect;
	strcpy(&p->affect.id[0], &id[0]);
	p->affect.expr = expr;
	p->ninst = expr->ninst + 1;
	
	return p;
}

asa* creer_noeudBloc(asa *p, asa *q) {
	if(!p) {
		yyerror("creer_bloc(nonnull p) with NULL p");
	}
	
	if(p->type == typeBloc) {
		yyerror("creer_bloc(nonblock p) with p->type == typeBloc");
	}
	
	asa *b = malloc(sizeof(asa));
	if(!b) {
		yyerror("échec allocation mémoire");
	}
	
	asa *svt = NULL;
	if(q != NULL) {
		if(q->type == typeBloc) {
			svt = q;
		}
		else {
			svt = malloc(sizeof(asa));
			if(!svt) {
				yyerror("échec allocation mémoire");
			}
			
			svt->type = typeBloc;
			svt->bloc.p = q;
			svt->bloc.svt = NULL;
		}
	}
	
	b->type = typeBloc;
	b->bloc.p = p;
	b->bloc.svt = svt;
	
	return b;
}

asa* creer_noeudVar(const char id[32]) {
	asa *p = malloc(sizeof(asa));
	if(!p) {
		yyerror("échec allocation mémoire");
	}
	
	p->type = typeVar;
	strcpy(&p->var.id[0], &id[0]);
	
	return p;
}

asa* creer_noeudAfficher(asa *expr) {
	asa *p = malloc(sizeof(asa));
	if(!p) {
		yyerror("échec allocation mémoire");
	}
	
	p->type = typeAfficher;
	p->af.expr = expr;
	
	return p;
}


void free_asa(asa *p)
{
 
  if (!p) return;
  switch (p->type) {
	case typeOp:
		free_asa(p->op.noeud[0]);
		free_asa(p->op.noeud[1]);
		break;
	
	case typeAffect:
		free_asa(p->affect.expr);
		break;
	
	case typeBloc:
		free_asa(p->bloc.p);
		free_asa(p->bloc.svt);
		break;
	
	case typeAfficher:
		free_asa(p->af.expr);
		break;
	
	case typeNb:
	case typeVar:
		break;
  }
  
  free(p);
}

void print_asa(asa *p) {
	if(!p) {
		return;
	}
	
	switch(p->type) {
		case typeNb:
			printf("%i", p->nb.val);
			break;
		
		case typeOp: ;
			asa *lhs = p->op.noeud[0];
			
			if(lhs->type != typeNb && lhs->type != typeVar) {
				printf("(");
				print_asa(lhs);
				printf(")");
			}
			else {
				print_asa(lhs);
			}
			
			printf(" %c ", p->op.ope);
			
			asa *rhs = p->op.noeud[1];
			if(rhs->type != typeNb && rhs->type != typeVar) {
				printf("(");
				print_asa(rhs);
				printf(")");
			}
			else {
				print_asa(rhs);
			}
			
			break;
		
		case typeAffect:
			printf("%s := ", p->affect.id);
			print_asa(p->affect.expr);
			break;
		
		case typeBloc:
			print_asa(p->bloc.p);
			printf("\n");
			print_asa(p->bloc.svt);
			break;
		
		case typeVar:
			printf("%s", p->var.id);
			break;
		
		case typeAfficher:
			printf("AFFICHER ");
			print_asa(p->af.expr);
			break;
	}
}

const char *ope_name(noeudOp n) {
	switch(n.ope) {
		case '+':
			return "ADD";
		
		case '-':
			return "SUB";
		
		case '*':
			return "MUL";
		
		case '/':
			return "DIV";
		
		default:
			fprintf(stderr, "erreur interne: opérateur inconnu: '%1$c' (%1$i)\n", n.ope);
			exit(1);
	}
}

void codegen_nc(asa *p, int *sp) {
	if(!p) {
		return;
	}
	
	switch(p->type) {
		case typeNb:
			printf("LOAD #%i\n", p->nb.val);
			break;
		
		case typeOp:
			codegen_nc(p->op.noeud[1], sp);
			printf("STORE %i\n", ++(*sp));
			
			codegen_nc(p->op.noeud[0], sp);
			printf("%s %i\n", ope_name(p->op), *sp);
			--(*sp);
			break;
		
		case typeAffect: {
			codegen_nc(p->affect.expr, sp);
			
			ts *var = ts_retrouver_id(p->affect.id);
			if(var == NULL) {
				ts_ajouter_id(p->affect.id, 1);
				var = ts_retrouver_id(p->affect.id);
			}
			
			printf("STORE %i\n", var->adr);
			break;
		}
		
		case typeVar: {
			ts *var = ts_retrouver_id(p->var.id);
			if(var == NULL) {
				yyerror("variable non définie");
			}
			
			printf("LOAD %i\n", var->adr);
			break;
		}
		
		case typeBloc:
			printf("NOP ; ");
			print_asa(p->bloc.p);
			printf("\n");
			
			codegen_nc(p->bloc.p, sp);
			codegen_nc(p->bloc.svt, sp);
			break;
		
		case typeAfficher:
			codegen_nc(p->af.expr, sp);
			printf("WRITE\n");
			break;
	}
}

void codegen(asa *p) {
	if(!p) return;
	
	printf("NOP ; DEBUT\n");
	
  	int sp = 1;
	codegen_nc(p, &sp);
	
	printf("NOP ; FIN\n");
}



void yyerror(const char * s)
{
  fprintf(stderr, "%s\n", s);
  exit(0);
}
