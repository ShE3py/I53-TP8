#include "codegen.h"

#pragma GCC diagnostic ignored "-Wmain"
#pragma GCC diagnostic ignored "-Wunused-function"

extern FILE *outfile;

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
 * Un noeud d'un liste chaînée d'adresses de fonctions.
 */
typedef struct fn_location_node {
	/**
	 * La fonction.
	 */
	asa *value;
	
	/**
	 * L'adresse de la fonction dans le segment de code.
	 */
	int adr;
	
	/**
	 * La fonction suivante.
	 */
	struct fn_location_node *next;
} fn_location_node;

/**
 * Renvoie les informations de la fonction spécifiée,
 * ou écrit un message d'erreur et ferme le programme si jamais la fonction n'existe pas.
 */
fn_location_node* get_fn(const char id[32]);

/**
 * L'adresse de la fonction de saut dynamique.
 */
static int dyn_jump_adr;

/**
 * Ajoute le point de retour à la fonction de saut dynamique.
 */
static void add_dyn_jump_adr(int adr);

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
			fprintf(outfile, "LOAD #%i\n", p->tag_int.value);
			++(*ip);
			
			break;
		}
		
		case TagVar: {
			symbol var = st_find_or_internal_error(p->tag_var.identifier);
			
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "ADD #%i\n", var.base_adr);
			fprintf(outfile, "LOAD @0 ; %s\n", var.identifier);
			*ip += 3;
			
			break;
		}
		
		case TagIndex: {
			symbol var = st_find_or_internal_error(p->tag_index.identifier);
			
			if(p->tag_index.index->tag == TagInt) {
				fprintf(outfile, "LOAD 1\n");
				fprintf(outfile, "ADD #%i\n", var.base_adr + p->tag_index.index->tag_int.value);
				fprintf(outfile, "LOAD @0 ; %s[%i]\n", var.identifier, p->tag_index.index->tag_int.value);
			}
			else {
				codegen_nc(p->tag_index.index, ip);
				fprintf(outfile, "ADD 1\n");
				fprintf(outfile, "ADD #%i\n", var.base_adr);
				fprintf(outfile, "LOAD @0 ; %s[", var.identifier);
				fprint_asa(outfile, p->tag_index.index);
				fprintf(outfile, "]\n");
			}
			
			*ip += 3;
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
					fprintf(outfile, "STORE @2\n");
					fprintf(outfile, "INC 2\n");
					*ip += 2;
					
					codegen_nc(p->tag_binary_op.lhs, ip);
					fprintf(outfile, "DEC 2\n");
					fprintf(outfile, "%s @2\n", binop_name(p->tag_binary_op.op));
					*ip += 2;
					
					// on génère ensuite le code de comparaison pour les opérateurs
					// comparatifs
					
					switch(p->tag_binary_op.op) {
						// remarque: il y a toujours une instruction après ce noeud,
						// a minima un `STOP`, donc `*ip + 4` existe bien toujours.
						
						case OpGe:
							// x - y >= 0 <=> !((x - y) < 0)
							
							fprintf(outfile, "JUML %i\n", *ip + 3);
							fprintf(outfile, "LOAD #1\n");
							fprintf(outfile, "JUMP %i\n", *ip + 4);
							fprintf(outfile, "LOAD #0\n");
							
							*ip += 4;
							break;
						
						case OpGt:
							// x - y > 0
							
							fprintf(outfile, "JUMG %i\n", *ip + 3);
							fprintf(outfile, "LOAD #0\n");
							fprintf(outfile, "JUMP %i\n", *ip + 4);
							fprintf(outfile, "LOAD #1\n");
							
							*ip += 4;
							break;
						
						case OpLe:
							// x - y <= 0 <=> !((x - y) > 0)
							
							fprintf(outfile, "JUMG %i\n", *ip + 3);
							fprintf(outfile, "LOAD #1\n");
							fprintf(outfile, "JUMP %i\n", *ip + 4);
							fprintf(outfile, "LOAD #0\n");
							
							*ip += 4;
							break;
						
						case OpLt:
							// x - y < 0
							
							fprintf(outfile, "JUML %i\n", *ip + 3);
							fprintf(outfile, "LOAD #0\n");
							fprintf(outfile, "JUMP %i\n", *ip + 4);
							fprintf(outfile, "LOAD #1\n");
							
							*ip += 4;
							break;
						
						case OpEq:
							// x - y == 0
							
							fprintf(outfile, "JUMZ %i\n", *ip + 3);
							fprintf(outfile, "LOAD #0\n");
							fprintf(outfile, "JUMP %i\n", *ip + 4);
							fprintf(outfile, "LOAD #1\n");
							
							*ip += 4;
							break;
						
						case OpNe:
							// x - y != 0
							
							fprintf(outfile, "JUMZ %i\n", *ip + 3);
							fprintf(outfile, "LOAD #1\n");
							fprintf(outfile, "JUMP %i\n", *ip + 4);
							fprintf(outfile, "LOAD #0\n");
							
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
							
							fprintf(outfile, "NOP ; TEST (");
							fprint_asa(outfile, p->tag_binary_op.lhs);
							fprintf(outfile, ")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.lhs, ip);
							fprintf(outfile, "JUMZ %i\n", *ip + p->tag_binary_op.rhs->ninst + 2);
							++(*ip);
							
							fprintf(outfile, "NOP ; TEST (");
							fprint_asa(outfile, p->tag_binary_op.rhs);
							fprintf(outfile, ")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.rhs, ip);
							break;
						
						case OpOr:
							// si opérande gauche == 1,
							// short-circuit à la toute fin
							// (ACC = 1)
							
							// sinon, ACC = opérande droite
							
							fprintf(outfile, "NOP ; TEST (");
							fprint_asa(outfile, p->tag_binary_op.lhs);
							fprintf(outfile, ")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.lhs, ip);
							fprintf(outfile, "JUMZ %i\n", *ip + 2);
							++(*ip);
							
							fprintf(outfile, "JUMP %i\n", *ip + p->tag_binary_op.rhs->ninst + 2);
							++(*ip);
							
							fprintf(outfile, "NOP ; TEST (");
							fprint_asa(outfile, p->tag_binary_op.rhs);
							fprintf(outfile, ")\n");
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
							
							fprintf(outfile, "NOP ; TEST (");
							fprint_asa(outfile, p->tag_binary_op.lhs);
							fprintf(outfile, ")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.lhs, ip);
							fprintf(outfile, "STORE @2\n");
							fprintf(outfile, "INC 2\n");
							*ip += 2;
							
							fprintf(outfile, "NOP ; TEST (");
							fprint_asa(outfile, p->tag_binary_op.rhs);
							fprintf(outfile, ")\n");
							++(*ip);
							
							codegen_nc(p->tag_binary_op.rhs, ip);
							fprintf(outfile, "NOP ; OU EXCLUSIF\n");
							fprintf(outfile, "DEC 2\n");
							fprintf(outfile, "JUMZ %i\n", *ip + 5);
							fprintf(outfile, "SUB @2\n");
							fprintf(outfile, "JUMP %i\n", *ip + 6);
							fprintf(outfile, "LOAD @2\n");
							
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
		    
		    switch(p->tag_unary_op.op) {
		        case OpNeg:
			        fprintf(outfile, "STORE @2\n");
			        fprintf(outfile, "LOAD #0\n");
			        fprintf(outfile, "SUB @2\n");
			        *ip += 3;
			        break;
			    
			    case OpNot:
			        fprintf(outfile, "JUMZ %d\n", *ip + 3);
			        fprintf(outfile, "LOAD #0\n");
			        fprintf(outfile, "JUMP %d\n", *ip + 4);
			        fprintf(outfile, "LOAD #1\n");
			        *ip += 4;
			        break;
		    }
			
			break;
		}
		
		case TagAssignScalar: {
			symbol var = st_find_or_internal_error(p->tag_assign_scalar.identifier);
			
			codegen_nc(p->tag_assign_scalar.expr, ip);
			
			fprintf(outfile, "STORE @2\n");
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "ADD #%i\n", var.base_adr);
			fprintf(outfile, "STORE 3\n");
			fprintf(outfile, "LOAD @2\n");
			fprintf(outfile, "STORE @3 ; %s := ", var.identifier);
			fprint_asa(outfile, p->tag_assign_scalar.expr);
			fprintf(outfile, "\n");
			*ip += 6;
			break;
		}
		
		case TagAssignIndexed: {
			symbol var = st_find_or_internal_error(p->tag_assign_indexed.identifier);
			
			codegen_nc(p->tag_assign_indexed.expr, ip);
			fprintf(outfile, "STORE @2\n");
			fprintf(outfile, "INC 2\n");
			
			*ip += 2;
			codegen_nc(p->tag_assign_indexed.index, ip);
			fprintf(outfile, "DEC 2\n");
			fprintf(outfile, "ADD 1\n");
			fprintf(outfile, "ADD #%i\n", var.base_adr);
			fprintf(outfile, "STORE 3\n");
			fprintf(outfile, "LOAD @2\n");
			fprintf(outfile, "STORE @3\n ; ");
			fprint_asa(outfile, p);
			fprintf(outfile, "\n");
			
			*ip += 6;
			break;
		}
		
		case TagAssignIntList: {
			symbol var = st_find_or_internal_error(p->tag_assign_int_list.identifier);
			
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "ADD #%i\n", var.base_adr);
			fprintf(outfile, "STORE 3\n");
			*ip += 3;
			
			asa_list_node *n = p->tag_assign_int_list.values.head;
			
			for(int i = 0; i < var.size; ++i) {
				codegen_nc(n->value, ip);
				fprintf(outfile, "STORE @3 ; %s[%i] = ", var.identifier, i);
				fprint_asa(outfile, n->value);
				fprintf(outfile, "\n");
				fprintf(outfile, "INC 3\n");
				
				*ip += 2;
				n = n->next;
			}
			
			break;
		}
		
		case TagAssignArray: {
			symbol dst = st_find_or_internal_error(p->tag_assign_array.dst);
			symbol src = st_find_or_internal_error(p->tag_assign_array.src);
			
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "ADD #%i\n", dst.base_adr);
			fprintf(outfile, "STORE 3 ; &%s[0]\n", dst.identifier);
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "ADD #%i\n", src.base_adr);
			
			for(int i = 0; i < dst.size; ++i) {
				fprintf(outfile, "LOAD @0 ; %s[%d]\n", src.identifier, i);
				fprintf(outfile, "ADD #1\n");
				fprintf(outfile, "STORE @3 ; %1$s[%3$d] = %2$s[%3$d]\n", dst.identifier, src.identifier, i);
				fprintf(outfile, "INC 3\n");
			}
			
			*ip += 4 + dst.size * 4;
			break;
		}
		
		case TagTest: {
			codegen_nc(p->tag_test.expr, ip);
			
			fprintf(outfile, "JUMZ %i\n", *ip + (p->tag_test.therefore ? p->tag_test.therefore->ninst : 0) + 2 + (p->tag_test.alternative ? 1 : 0));
			fprintf(outfile, "NOP ; ALORS\n");
			
			*ip += 2;
			codegen_nc(p->tag_test.therefore, ip);
			
			if(p->tag_test.alternative) {
				fprintf(outfile, "JUMP %i\n", *ip + p->tag_test.alternative->ninst + 2);
				fprintf(outfile, "NOP ; SINON\n");
				
				*ip += 2;
				codegen_nc(p->tag_test.alternative, ip);
			}
			
			fprintf(outfile, "NOP ; FSI\n");
			
			++(*ip);
			break;
		}
		
		case TagWhile: {
			codegen_nc(p->tag_while.expr, ip);
			
			fprintf(outfile, "JUMZ %i\n", *ip + p->tag_while.body->ninst + 2);
			++(*ip);
			
			codegen_nc(p->tag_while.body, ip);
			
			fprintf(outfile, "JUMP %i\n", before_codegen_ip);
			++(*ip);
			
			break;
		}
		
		case TagRead: {
			symbol var = st_find_or_internal_error(p->tag_read.identifier);
			
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "ADD #%i\n", var.base_adr);
			fprintf(outfile, "STORE 3\n");
			
			fprintf(outfile, "READ\n");
			fprintf(outfile, "STORE @3 ; %s\n", var.identifier);
			
			*ip += 5;
			break;
		}
		
		case TagReadIndexed: {
			symbol var = st_find_or_internal_error(p->tag_read_indexed.identifier);
			
			codegen_nc(p->tag_read_indexed.index, ip);
			
			fprintf(outfile, "STORE @2\n");
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "ADD #%i\n", var.base_adr);
			fprintf(outfile, "ADD @2\n");
			fprintf(outfile, "STORE 3 ; &%s[", var.identifier);
			fprint_asa(outfile, p->tag_read_indexed.index);
			fprintf(outfile, "]\n");
			
			fprintf(outfile, "READ\n");
			fprintf(outfile, "STORE @3 %s[", var.identifier);
			fprint_asa(outfile, p->tag_read_indexed.index);
			fprintf(outfile, "]\n");
			
			*ip += 7;
			break;
		}
		
		case TagReadArray: {
			symbol var = st_find_or_internal_error(p->tag_read_array.identifier);
			
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "ADD #%i\n", var.base_adr);
			fprintf(outfile, "STORE 3 ; &%s[0]\n", var.identifier);
			
			for(int i = 0; i < var.size; ++i) {
				fprintf(outfile, "READ\n");
				fprintf(outfile, "STORE @3 ; %s[%d]\n", var.identifier, i);
				fprintf(outfile, "INC 3\n");
			}
			
			*ip += 3 + var.size * 3;
			break;
		}
		
		case TagPrint: {
			codegen_nc(p->tag_print.expr, ip);
			fprintf(outfile, "WRITE\n");
			++(*ip);
			break;
		}
		
		case TagPrintArray: {
			symbol var = st_find_or_internal_error(p->tag_print_array.identifier);
			
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "ADD #%i\n", var.base_adr);
			fprintf(outfile, "STORE 3 ; &%s[0]\n", var.identifier);
			
			for(int i = 0; i < var.size; ++i) {
				fprintf(outfile, "LOAD @3 ; %s[%d]\n", var.identifier, i);
				fprintf(outfile, "WRITE\n");
				fprintf(outfile, "INC 3\n");
			}
			
			*ip += 3 + var.size * 3;
			break;
		}
		
		case TagBlock: {
			fprintf(outfile, "NOP ; ");
			fprint_asa(outfile, p->tag_block.stmt);
			fprintf(outfile, "\n");
			
			++(*ip);
			
			codegen_nc(p->tag_block.stmt, ip);
			codegen_nc(p->tag_block.next, ip);
			break;
		}
		
		case TagFn: {
			st_make_current(p->tag_fn.st);
			
			fprintf(outfile, "NOP ; ");
			fprint_asa(outfile, p);
			
			fprintf(outfile, "\nNOP ; STACK ");
			st_fprint_current(outfile);
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "ADD #%i\n", st_temp_offset());
			fprintf(outfile, "STORE 2\n");
			
			fprintf(outfile, "NOP ; DEBUT\n");
			*ip += 6;
			
			codegen_nc(p->tag_fn.body, ip);
			fprintf(outfile, "STOP ; FIN\n");
			++(*ip);
			break;
		}
		
		case TagFnCall: {
			fprintf(outfile, "LOAD 1\n");
			fprintf(outfile, "STORE @2\n");
			fprintf(outfile, "INC 2\n");
			
			int jmp = *ip + 9 + p->tag_fn_call.args.ninst + (6 * p->tag_fn_call.args.len);
			add_dyn_jump_adr(jmp);
			fprintf(outfile, "LOAD #%i\n", jmp);
			fprintf(outfile, "STORE @2\n");
			fprintf(outfile, "INC 2\n");
			
			*ip += 6;
			
			fn_location_node *fn = get_fn(p->tag_fn_call.identifier);
			if(fn->value->tag_fn.params.len != p->tag_fn_call.args.len) {
				fprintf(stderr, "'%s()': %lu paramètres attendus, %lu paramètres donnés\n", p->tag_fn_call.identifier, fn->value->tag_fn.params.len, p->tag_fn_call.args.len);
				exit(1);
			}
			
			int args = p->tag_fn_call.args.len;
			if(args > 0) {
				asa **aargs = malloc(args * sizeof(asa*));
				
				// on empile directement les paramètres sur la pile,
				// vu qu'ils sont garantis d'avoir les adresses [0, args[
				
				// il faut d'abord inverser la liste
				
				asa_list_node *n = p->tag_fn_call.args.head;
				int i = args;
				while(n) {
					aargs[--i] = n->value;
					n = n->next;
				}
				
				for(i = 0; i < args; ++i) {
					codegen_nc(aargs[i], ip);
					fprintf(outfile, "STORE @2\n");
					
					fprintf(outfile, "LOAD 2\n");
					fprintf(outfile, "ADD #%i\n", args - i - 1);
					fprintf(outfile, "STORE 3\n");
					
					fprintf(outfile, "LOAD @2\n");
					fprintf(outfile, "STORE @3\n");
					
					*ip += 6;
				}
				
				free(aargs);
			}
			
			if(jmp != *ip + 3) {
				fprintf(stderr, "bad jump\n");
				exit(1);
			}
			
			fprintf(outfile, "LOAD 2\n");
			fprintf(outfile, "STORE 1\n");
			
			fprintf(outfile, "JUMP %i\n", fn->adr);
			
			fprintf(outfile, "LOAD 2\n");
			fprintf(outfile, "SUB #3\n");
			fprintf(outfile, "STORE 2\n");
			
			fprintf(outfile, "LOAD @0\n");
			fprintf(outfile, "STORE 1\n");
			
			fprintf(outfile, "LOAD 2\n");
			fprintf(outfile, "ADD #3\n");
			fprintf(outfile, "LOAD @0\n");
			
			*ip += 11;
			break;
		}
		
		case TagReturn: {
			if(p->tag_return.expr && p->tag_return.expr != NOP) {
				codegen_nc(p->tag_return.expr, ip);
			}
			else {
				fprintf(outfile, "LOAD #0\n");
				++(*ip);
			}
			
			fprintf(outfile, "STORE @2\n");
			fprintf(outfile, "DEC 1\n");
			fprintf(outfile, "LOAD @1\n");
			fprintf(outfile, "JUMP %i\n", dyn_jump_adr);
			
			*ip += 4;
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
 * La tête de la liste chaînée contenant les adresses des fonctions.
 */
static fn_location_node fn_locations;

/**
 * Détermine l'emplacement des fonctions dans le segment de code.
 */
static void allocate_fn_space(asa_list fns, int base_ip) {
	asa_list_node *main = NULL;
	
	asa_list_node *n = fns.head;
	while(n) {
		if(strcmp(n->value->tag_fn.identifier, "main") == 0) {
			main = n;
			break;
		}
		
		n = n->next;
	}
	
	if(!main) {
		fprintf(stderr, "erreur: pas de fonction principale définie\n");
		exit(1);
	}
	
	int ip = base_ip;
	fn_locations.value = main->value;
	fn_locations.adr = ip;
	fn_locations.next = NULL;
	
	ip += main->value->ninst;
	
	n = fns.head;
	while(n) {
		if(n->value != main->value) {
			fn_location_node *m = &fn_locations;
			while(1) {
				if(m->value != n->value && strcmp(n->value->tag_fn.identifier, m->value->tag_fn.identifier) == 0) {
					fprintf(stderr, "fonction dupliquée: '%s'\n", n->value->tag_fn.identifier);
					exit(1);
				}
				
				if(!m->next) {
					break;
				}
				
				m = m->next;
			}
			
			fn_location_node *o = malloc(sizeof(fn_location_node));
			o->value = n->value;
			o->adr = ip;
			o->next = NULL;
				
			m->next = o;
			ip += o->value->ninst;
		}
		
		n = n->next;
	}
	
	dyn_jump_adr = ip;
}


/**
 * Renvoie les informations de la fonction spécifiée,
 * ou écrit un message d'erreur et ferme le programme si jamais la fonction n'existe pas.
 */
fn_location_node* get_fn(const char id[32]) {
	fn_location_node *n = &fn_locations;
	do {
		if(strcmp(n->value->tag_fn.identifier, id) == 0) {
			return n;
		}
		
		n = n->next;
	}
	while(n);
	
	fprintf(stderr, "fonction inconnue: '%s'\n", id);
	exit(1);
}

/**
 * Un élément d'une liste chaînée de nombre entiers.
 */
typedef struct int_list_node {
	int value;
	struct int_list_node *next;
} int_list_node;

/**
 * Les adresses dans le segment de données dont il est possible
 * d'accéder dynamiquement.
 */
static int_list_node *dyn_jumps;

/**
 * Ajoute le point de retour à la fonction de saut dynamique.
 */
static void add_dyn_jump_adr(int adr) {
	int_list_node *n = malloc(sizeof(int_list_node));
	n->value = adr;
	n->next = NULL;
	
	if(!dyn_jumps) {
		dyn_jumps = n;
	}
	else {
		// sorted insertion
		int_list_node *m = dyn_jumps;
		while(m->next) {
			int_list_node *o = m->next;
			if(o->value > adr) {
//				m->next = n;  en sortie de boucle
				n->next = o;
				break;
			}
			
			m = o;
		}
		
		m->next = n;
	}
}

/**
 * Génère le code pour la fonction de sauts dynamiques.
 */
static void codegen_dyn_jump() {
	fprintf(outfile, "NOP ; BUILTIN JUMP @0\n");
	
	int_list_node *n = dyn_jumps;
	int sum = 0;
	
	while(n) {
		fprintf(outfile, "SUB #%i\n", n->value - sum);
		fprintf(outfile, "JUMZ %i\n", n->value);
		
		sum += n->value - sum;
		n = n->next;
	}
	
	fprintf(outfile, "STOP ; UNREACHABLE\n");
}

/**
 * Libère la liste des sauts dynamiques.
 */
static void destroy_dyn_jumps() {
    int_list_node *n = dyn_jumps;
    while(n) {
        int_list_node *m = n->next;
        free(n);
        
        n = m;
    }
    
    dyn_jumps = NULL;
}

static void print_fn_locations() {
	fprintf(outfile, "{\n");
	
	fn_location_node *n = &fn_locations;
	do {
		fprintf(outfile, "\t%s: %i\n", n->value->tag_fn.identifier, n->adr);
		n = n->next;
	}
	while(n);
	
	fprintf(outfile, "}\n");
}

static void print_int_list(int_list_node *head) {
	if(!head) {
		fprintf(outfile, "{}\n");
	}
	else {
		fprintf(outfile, "{%i", head->value);
		
		int_list_node *n = head->next;
		while(n) {
			fprintf(outfile, ", %i", n->value);
			n = n->next;
		}
		
		fprintf(outfile, "}\n");
	}
}

static void destroy_fn_space() {
	fn_location_node *n = fn_locations.next;
	while(n) {
		fn_location_node *m = n->next;
		free(n);
		
		n = m;
	}
}

/**
 * Génère le code pour la machine RAM correspondant au programme spécifié.
 */
void codegen(asa_list fns) {
	if(fns.len == 0) {
		fprintf(outfile, "STOP\n");
		fprintf(stderr, "avertissement: le fichier source est vide\n");
		exit(1);
	}
	
	fprintf(outfile, "LOAD #4\n");
	fprintf(outfile, "STORE 1\n");
	
	int ip = 2;
	
	allocate_fn_space(fns, ip);
	
	fn_location_node *n = &fn_locations;
	while(n) {
		codegen_nc(n->value, &ip);
		
		n = n->next;
	}
	
	codegen_dyn_jump();
	destroy_dyn_jumps();
	destroy_fn_space();
}

