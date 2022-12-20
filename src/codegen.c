#include "codegen.h"

/**
 * Renvoie l'instruction de la machine RAM associée à un opérateur binaire.
 */
const char* binop_name(BinaryOp binop) {
	switch(binop) {
		case OpAdd:
			return "ADD";
		
		case OpSub:
			return "SUB";
		
		case OpMul:
			return "MUL";
		
		case OpDiv:
			return "DIV";
		
		case OpMod:
			return "MOD";
	}
	
	fprintf(stderr, "entered unreachable code\n");
	exit(1);
}

/**
 *
 */
void codegen_nc(asa *p, int *sp) {
	if(!p) {
		return;
	}
	
	switch(p->tag) {
		case TagInt: {
			printf("LOAD #%i\n", p->tag_int.value);
			break;
		}
		
		case TagVar: {
			ts *var = ts_retrouver_id(p->tag_var.identifier);
			if(var == NULL) {
				fprintf(stderr, "illegal state: '%s' should exists at this stage but it does not\n", p->tag_var.identifier);
				exit(1);
			}
			
			printf("LOAD %i\n", var->adr);
			break;
		}
		
		case TagBinaryOp: {
			codegen_nc(p->tag_binary_op.rhs, sp);
			printf("STORE %i\n", ++(*sp));
			
			codegen_nc(p->tag_binary_op.lhs, sp);
			printf("%s %i\n", binop_name(p->tag_binary_op.op), *sp);
			--(*sp);
			break;
		}
		
		case TagAssign: {
			codegen_nc(p->tag_assign.expr, sp);
			
			ts *var = ts_retrouver_id(p->tag_assign.identifier);
			if(var == NULL) {
				ts_ajouter_id(p->tag_assign.identifier, 1);
				var = ts_retrouver_id(p->tag_assign.identifier);
			}
			
			printf("STORE %i\n", var->adr);
			break;
		}
		
		case TagPrint: {
			codegen_nc(p->tag_print.expr, sp);
			printf("WRITE\n");
			break;
		}
		
		case TagBlock: {
			printf("NOP ; ");
			print_asa(p->tag_block.stmt);
			printf("\n");
			
			codegen_nc(p->tag_block.stmt, sp);
			codegen_nc(p->tag_block.next, sp);
			break;
		}
	}
}

/**
 * Génère le code pour la machine RAM correspondant à l'arbre syntaxique abstrait spécifié.
 */
void codegen(asa *p) {
	if(!p) return;
	
	printf("NOP ; DEBUT\n");
	
  	int sp = 1;
	codegen_nc(p, &sp);
	
	printf("STOP ; FIN\n");
}
