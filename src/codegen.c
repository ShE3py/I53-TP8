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
		
		case OpGe:
		case OpGt:
		case OpLe:
		case OpLt:
		case OpEq:
		case OpNe:
			// x ♥ y <=> x - y ♥ 0
			// codegen fera `♥ 0`
			return "SUB";
		
		case OpAnd:
		case OpOr:
		case OpXor:
			fprintf(stderr, "binop_name(...) is not defined for logic binops\n");
			exit(1);
	}
	
	fprintf(stderr, "entered unreachable code\n");
	exit(1);
}

/**
 *
 */
void codegen_nc(asa *p, int *sp, int *ip) {
	if(!p) {
		return;
	}
	
	switch(p->tag) {
		case TagInt: {
			printf("LOAD #%i\n", p->tag_int.value);
			++(*ip);
			
			break;
		}
		
		case TagVar: {
			ts *var = ts_retrouver_id(p->tag_var.identifier);
			if(var == NULL) {
				fprintf(stderr, "illegal state: '%s' should exists at this stage but it does not\n", p->tag_var.identifier);
				exit(1);
			}
			
			printf("LOAD %i\n", var->adr);
			++(*ip);
			
			break;
		}
		
		case TagBinaryOp: {
			switch(binop_kind(p->tag_binary_op.op)) {
				case Arithmetic:
				case Comparative:
					codegen_nc(p->tag_binary_op.rhs, sp, ip);
					printf("STORE %i\n", ++(*sp));
					++(*ip);
					
					codegen_nc(p->tag_binary_op.lhs, sp, ip);
					printf("%s %i\n", binop_name(p->tag_binary_op.op), *sp);
					++(*ip);
					--(*sp);
					
					switch(p->tag_binary_op.op) {
						// remarque: il y a toujours une instruction après ce noeud,
						// a minima un `STOP`, donc `*ip + 4` existe bien toujours.
						
						case OpGe:
							// x - y >= 0 <=> !((x - y) < 0)
							
							printf("JUML %i\n", *ip + 3);
							printf("LOAD #1\n");
							printf("JUMP %i\n", *ip + 4);
							printf("LOAD #0\n");
							
							*ip += 4;
							break;
						
						case OpGt:
							// x - y > 0
							
							printf("JUMG %i\n", *ip + 3);
							printf("LOAD #0\n");
							printf("JUMP %i\n", *ip + 4);
							printf("LOAD #1\n");
							
							*ip += 4;
							break;
						
						case OpLe:
							// x - y <= 0 <=> !((x - y) > 0)
							
							printf("JUMG %i\n", *ip + 3);
							printf("LOAD #1\n");
							printf("JUMP %i\n", *ip + 4);
							printf("LOAD #0\n");
							
							*ip += 4;
							break;
						
						case OpLt:
							// x - y < 0
							
							printf("JUML %i\n", *ip + 3);
							printf("LOAD #0\n");
							printf("JUMP %i\n", *ip + 4);
							printf("LOAD #1\n");
							
							*ip += 4;
							break;
						
						case OpEq:
							// x - y == 0
							
							printf("JUMZ %i\n", *ip + 3);
							printf("LOAD #0\n");
							printf("JUMP %i\n", *ip + 4);
							printf("LOAD #1\n");
							
							*ip += 4;
							break;
						
						case OpNe:
							// x - y != 0
							
							printf("JUMZ %i\n", *ip + 3);
							printf("LOAD #1\n");
							printf("JUMP %i\n", *ip + 4);
							printf("LOAD #0\n");
							
							*ip += 4;
							break;
						
						case OpAdd:
						case OpSub:
						case OpMul:
						case OpDiv:
						case OpMod:
							break;
						
						case OpAnd:
						case OpOr:
						case OpXor:
							break;
					}
					
					break;
				
				case Logic:
					printf("NOP ; TEST (");
					print_asa(p->tag_binary_op.lhs);
					printf(")\n");
					++(*ip);
					
					codegen_nc(p->tag_binary_op.lhs, sp, ip);
					printf("JUMZ %i\n", *ip + p->tag_binary_op.rhs->ninst + 4);
					++(*ip);
					
					printf("NOP ; TEST (");
					print_asa(p->tag_binary_op.rhs);
					printf(")\n");
					++(*ip);
					
					codegen_nc(p->tag_binary_op.rhs, sp, ip);
					printf("JUMP %i\n", *ip + 2);
					printf("LOAD #0\n");
					
					*ip += 2;
					break;
			}
			
			break;
		}
		
		case TagUnaryOp: {
			codegen_nc(p->tag_unary_op.expr, sp, ip);
			printf("STORE %i\n", *sp + 1);
			printf("LOAD #0\n");
			printf("SUB %i\n", *sp + 1);
			
			*ip += 3;
			break;
		}
		
		case TagAssign: {
			codegen_nc(p->tag_assign.expr, sp, ip);
			
			ts *var = ts_retrouver_id(p->tag_assign.identifier);
			if(var == NULL) {
				ts_ajouter_id(p->tag_assign.identifier, 1);
				var = ts_retrouver_id(p->tag_assign.identifier);
			}
			
			printf("STORE %i\n", var->adr);
			++(*ip);
			break;
		}
		
		case TagPrint: {
			codegen_nc(p->tag_print.expr, sp, ip);
			printf("WRITE\n");
			++(*ip);
			break;
		}
		
		case TagBlock: {
			printf("NOP ; ");
			print_asa(p->tag_block.stmt);
			printf("\n");
			
			++(*ip);
			
			codegen_nc(p->tag_block.stmt, sp, ip);
			codegen_nc(p->tag_block.next, sp, ip);
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
	int ip = 1;
	codegen_nc(p, &sp, &ip);
	
	printf("STOP ; FIN\n");
}
