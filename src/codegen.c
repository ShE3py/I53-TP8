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
 * Génère le code pour la machine RAM correspondant à l'arbre syntaxique abstrait spécifié.
 * Cette fonction est récursive.
 *
 * Paramètres :
 * - `p`: le noeud à générer
 * - `ip`: instruction pointer, le numéro de l'instruction actuelle
 *
 * Aucune idée de pourquoi j'avais utilisé `nc` comme suffixe.
 */
void codegen_nc(asa *p, int *ip) {
	if(!p) {
		return;
	}
	
	if(p == NOP) {
		return;
	}
	
	const int before_codegen_ip = *ip;
	
	switch(p->tag) {
		case TagInt: {
			printf("LOAD #%i\n", p->tag_int.value);
			++(*ip);
			
			break;
		}
		
		case TagVar: {
			symbol var = st_find_or_internal_error(p->tag_var.identifier);
			
			printf("LOAD 1\n");
			printf("ADD #%i\n", var.base_adr);
			printf("LOAD @0\n");
			*ip += 3;
			
			break;
		}
		
		case TagIndex: {
			symbol var = st_find_or_internal_error(p->tag_index.identifier);
			
			if(p->tag_index.index->tag == TagInt) {
				printf("LOAD 1\n");
				printf("ADD #%i\n", var.base_adr + p->tag_index.index->tag_int.value);
				printf("LOAD @0\n");
				
				*ip += 3;
			}
			else {
				codegen_nc(p->tag_index.index, ip);
				printf("STORE @2\n");
				printf("LOAD 1\n");
				printf("ADD @2\n");
				printf("ADD #%i\n", var.base_adr);
				printf("LOAD @0\n");
				
				*ip += 5;
			}
			
			break;
		}
		
		case TagBinaryOp: {
			switch(binop_kind(p->tag_binary_op.op)) {
				case Arithmetic:
				case Comparative:
					// pour les opérateurs arithmétiques, on peut calculer
					// directement l'expression
					
					// pour les opérateurs comparatifs, on calcul en premier lieu
					// x - y (binop_name(op comparatif) == 'SUB')
					
					
					codegen_nc(p->tag_binary_op.rhs, ip);
					printf("STORE @2\n");
					printf("INC 2\n");
					*ip += 2;
					
					codegen_nc(p->tag_binary_op.lhs, ip);
					printf("DEC 2\n");
					printf("%s @2\n", binop_name(p->tag_binary_op.op));
					*ip += 2;
					
					// on génère ensuite le code de comparaison pour les opérateurs
					// comparatifs
					
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
							// pas de code de comparaison à générer pour les opérateurs
							// arithmétiques
							break;
						
						case OpAnd:
						case OpOr:
						case OpXor:
							fprintf(stderr, "entered unreachable code\n");
							exit(1);
					}
					
					break;
				
				case Logic:
					// pour les opérateurs logiques,
					// on short-circuit dès qu'on a évaluer l'opérande gauche
					
					switch(p->tag_binary_op.op) {
						case OpAnd:
							// si opérande gauche == zéro,
							// short-circuit à la toute fin
							// (ACC = 0)
							
							// sinon, ACC = opérande droite
							
							printf("NOP ; TEST (");
							print_asa(p->tag_binary_op.lhs);
							printf(")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.lhs, ip);
							printf("JUMZ %i\n", *ip + p->tag_binary_op.rhs->ninst + 2);
							++(*ip);
							
							printf("NOP ; TEST (");
							print_asa(p->tag_binary_op.rhs);
							printf(")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.rhs, ip);
							break;
						
						case OpOr:
							// si opérande gauche == 1,
							// short-circuit à la toute fin
							// (ACC = 1)
							
							// sinon, ACC = opérande droite
							
							printf("NOP ; TEST (");
							print_asa(p->tag_binary_op.lhs);
							printf(")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.lhs, ip);
							printf("JUMZ %i\n", *ip + 2);
							++(*ip);
							
							printf("JUMP %i\n", *ip + p->tag_binary_op.rhs->ninst + 2);
							++(*ip);
							
							printf("NOP ; TEST (");
							print_asa(p->tag_binary_op.rhs);
							printf(")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.rhs, ip);
							break;
						
						case OpXor:
							// on doit obligatoirement évaluer les deux
							// opérandes pour le OU EXCLUSIF
							
							// R[*sp] = opérande gauche,
							// ACC = opérande droite
							
							// si ACC = 0,
							// alors ACC = R[*sp]
							//
							// si ACC = 1,
							// alors ACC = 1 - R[*sp]
							//
							// donc ACC = 1 si R[*sp] = 0,
							// et ACC = 0 si R[*sp] = 1
							
							printf("NOP ; TEST (");
							print_asa(p->tag_binary_op.lhs);
							printf(")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.lhs, ip);
							printf("STORE @2\n");
							printf("INC 2\n");
							*ip += 2;
							
							printf("NOP ; TEST (");
							print_asa(p->tag_binary_op.rhs);
							printf(")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.rhs, ip);
							printf("NOP ; OU EXCLUSIF\n");
							printf("DEC 2\n");
							printf("JUMZ %i\n", *ip + 5);
							printf("SUB @2\n");
							printf("JUMP %i\n", *ip + 6);
							printf("LOAD @2\n");
							
							*ip += 6;
							break;
						
						case OpAdd:
						case OpSub:
						case OpMul:
						case OpDiv:
						case OpMod:
							fprintf(stderr, "entered unreachable code\n");
							exit(1);
						
						case OpGe:
						case OpGt:
						case OpLe:
						case OpLt:
						case OpEq:
						case OpNe:
							fprintf(stderr, "entered unreachable code\n");
							exit(1);
					}
					
					break;
			}
			
			break;
		}
		
		case TagUnaryOp: {
			codegen_nc(p->tag_unary_op.expr, ip);
			
			printf("STORE @2\n");
			printf("LOAD #0\n");
			printf("SUB @2\n");
			
			*ip += 3;
			break;
		}
		
		case TagAssignScalar: {
			symbol var = st_find_or_internal_error(p->tag_assign_scalar.identifier);
			
			printf("LOAD 1\n");
			printf("ADD #%i\n", var.base_adr);
			printf("STORE @2\n");
			printf("INC 2\n");
			*ip += 4;
			
			codegen_nc(p->tag_assign_scalar.expr, ip);
			
			printf("STORE @2\n");
			printf("DEC 2\n");
			printf("LOAD @2\n");
			printf("STORE 3\n");
			printf("INC 2\n");
			printf("LOAD @2\n");
			printf("STORE @3\n");
			printf("DEC 2\n");
			*ip += 8;
			break;
		}
		
		case TagAssignIndexed: {
			symbol var = st_find_or_internal_error(p->tag_assign_indexed.identifier);
			
			codegen_nc(p->tag_assign_indexed.index, ip);
			printf("ADD 1\n");
			printf("ADD #%i\n", var.base_adr);
			printf("STORE @2\n");
			printf("INC 2\n");
			
			*ip += 4;
			codegen_nc(p->tag_assign_indexed.expr, ip);
			printf("STORE @2\n");
			printf("DEC 2\n");
			printf("LOAD @2\n");
			printf("STORE 3\n");
			printf("INC 2\n");
			printf("LOAD @2\n");
			printf("STORE @3\n");
			printf("DEC 2\n");
			
			*ip += 8;
			break;
		}
		
		case TagAssignIntList: {
			symbol var = st_find_or_internal_error(p->tag_assign_int_list.identifier);
			
			asa_list_node *n = p->tag_assign_int_list.values.head;
			
			for(int i = 0; i < var.size; ++i) {
				codegen_nc(n->value, ip);
				printf("STORE @2\n");
				printf("LOAD 1\n");
				printf("ADD #%i\n", var.base_adr + i);
				printf("STORE 3\n");
				printf("LOAD @2\n");
				printf("STORE @3\n");
				
				*ip += 6;
				n = n->next;
			}
			
			break;
		}
		
		case TagAssignArray: {
			symbol dst = st_find_or_internal_error(p->tag_assign_array.dst);
			symbol src = st_find_or_internal_error(p->tag_assign_array.src);
			
			printf("LOAD 1\n");
			printf("ADD #%i\n", dst.base_adr);
			printf("STORE 3\n");
			
			for(int i = 0; i < dst.size; ++i) {
				printf("LOAD 1\n");
				printf("ADD #%i\n", src.base_adr + i);
				printf("LOAD @0\n");
				
				printf("STORE @3\n");
				printf("INC 3\n");
			}
			
			*ip += 3 + dst.size * 5;
			break;
		}
		
		case TagTest: {
			codegen_nc(p->tag_test.expr, ip);
			
			printf("JUMZ %i\n", *ip + (p->tag_test.therefore ? p->tag_test.therefore->ninst : 0) + 2 + (p->tag_test.alternative ? 1 : 0));
			printf("NOP ; ALORS\n");
			
			*ip += 2;
			codegen_nc(p->tag_test.therefore, ip);
			
			if(p->tag_test.alternative) {
				printf("JUMP %i\n", *ip + p->tag_test.alternative->ninst + 2);
				printf("NOP ; SINON\n");
				
				*ip += 2;
				codegen_nc(p->tag_test.alternative, ip);
			}
			
			printf("NOP ; FSI\n");
			
			++(*ip);
			break;
		}
		
		case TagWhile: {
			codegen_nc(p->tag_while.expr, ip);
			
			printf("JUMZ %i\n", *ip + p->tag_while.body->ninst + 2);
			++(*ip);
			
			codegen_nc(p->tag_while.body, ip);
			
			printf("JUMP %i\n", before_codegen_ip);
			++(*ip);
			
			break;
		}
		
		case TagRead: {
			symbol var = st_find_or_internal_error(p->tag_read.identifier);
			
			printf("LOAD 1\n");
			printf("ADD #%i\n", var.base_adr);
			printf("STORE 3\n");
			
			printf("READ\n");
			printf("STORE @3\n");
			
			*ip += 5;
			break;
		}
		
		case TagReadIndexed: {
			symbol var = st_find_or_internal_error(p->tag_read_indexed.identifier);
			
			codegen_nc(p->tag_read_indexed.index, ip);
			
			printf("STORE @2\n");
			printf("LOAD 1\n");
			printf("ADD #%i\n", var.base_adr);
			printf("ADD @2\n");
			printf("STORE 3\n");
			
			printf("READ\n");
			printf("STORE @3\n");
			
			*ip += 7;
			break;
		}
		
		case TagReadArray: {
			symbol var = st_find_or_internal_error(p->tag_read_array.identifier);
			
			printf("LOAD 1\n");
			printf("ADD #%i\n", var.base_adr);
			printf("STORE 3\n");
			
			for(int i = 0; i < var.size; ++i) {
				printf("READ\n");
				printf("STORE @3\n");
				printf("INC 3\n");
			}
			
			*ip += 3 + var.size * 3;
			break;
		}
		
		case TagPrint: {
			codegen_nc(p->tag_print.expr, ip);
			printf("WRITE\n");
			++(*ip);
			break;
		}
		
		case TagPrintArray: {
			symbol var = st_find_or_internal_error(p->tag_print_array.identifier);
			
			printf("LOAD 1\n");
			printf("ADD #%i\n", var.base_adr);
			printf("STORE 3\n");
			
			for(int i = 0; i < var.size; ++i) {
				printf("LOAD @3\n");
				printf("WRITE\n");
				printf("INC 3\n");
			}
			
			*ip += 3 + var.size * 3;
			break;
		}
		
		case TagBlock: {
			printf("NOP ; ");
			print_asa(p->tag_block.stmt);
			printf("\n");
			
			++(*ip);
			
			codegen_nc(p->tag_block.stmt, ip);
			codegen_nc(p->tag_block.next, ip);
			break;
		}
		
		case TagFn: {
			st_make_current(p->tag_fn.st);
			
			printf("NOP ; ");
			print_asa(p);
			
			printf("\nLOAD 1\n");
			printf("ADD #%i\n", st_temp_offset());
			printf("STORE 2\n");
			
			printf("NOP ; DEBUT\n");
			*ip += 5;
			
			codegen_nc(p->tag_fn.body, ip);
			printf("STOP ; FIN\n");
			++(*ip);
			break;
		}
	}
	
	if((before_codegen_ip + p->ninst) != *ip) {
		fprintf(stderr, "generated %i instructions for current node, but ninst is %i\n", *ip - before_codegen_ip, p->ninst);
		exit(1);
	}
	
	if(p->ninst == 0) {
		fprintf(stderr, "warning: generated no instruction for current node, no-ops should be `NULL`\n");
	}
}

/**
 * Génère le code pour la machine RAM correspondant au programme spécifié.
 */
void codegen(asa_list fns) {
	printf("LOAD #4\n");
	printf("STORE 1\n");
	
	int ip = 2;
	
	asa_list_node *n = fns.head;
	while(n) {
		codegen_nc(n->value, &ip);
		
		n = n->next;
	}
}
