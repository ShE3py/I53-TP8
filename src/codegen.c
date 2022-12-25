#include "codegen.h"

#pragma GCC diagnostic ignored "-Wmain"
#pragma GCC diagnostic ignored "-Wunused-function"

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
			
			printf("\nNOP ; STACK\n");
			printf("LOAD 1\n");
			printf("ADD #%i\n", st_temp_offset());
			printf("STORE 2\n");
			
			printf("NOP ; DEBUT\n");
			*ip += 6;
			
			codegen_nc(p->tag_fn.body, ip);
			printf("STOP ; FIN\n");
			++(*ip);
			break;
		}
		
		case TagFnCall: {
			printf("LOAD 1\n");
			printf("STORE @2\n");
			printf("INC 2\n");
			
			int jmp = *ip + 9 + p->tag_fn_call.args.ninst + (6 * p->tag_fn_call.args.len);
			add_dyn_jump_adr(jmp);
			printf("LOAD #%i\n", jmp);
			printf("STORE @2\n");
			printf("INC 2\n");
			
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
					printf("STORE @2\n");
					
					printf("LOAD 2\n");
					printf("ADD #%i\n", args - i - 1);
					printf("STORE 3\n");
					
					printf("LOAD @2\n");
					printf("STORE @3\n");
					
					*ip += 6;
				}
				
				free(aargs);
			}
			
			if(jmp != *ip + 3) {
				fprintf(stderr, "bad jump\n");
				exit(1);
			}
			
			printf("LOAD 2\n");
			printf("STORE 1\n");
			
			printf("JUMP %i\n", fn->adr);
			
			printf("LOAD 2\n");
			printf("SUB #3\n");
			printf("STORE 2\n");
			
			printf("LOAD @0\n");
			printf("STORE 1\n");
			
			printf("LOAD 2\n");
			printf("ADD #3\n");
			printf("LOAD @0\n");
			
			*ip += 11;
			break;
		}
		
		case TagReturn: {
			if(p->tag_return.expr && p->tag_return.expr != NOP) {
				codegen_nc(p->tag_return.expr, ip);
			}
			else {
				printf("LOAD #0\n");
				++(*ip);
			}
			
			printf("STORE @2\n");
			printf("DEC 1\n");
			printf("LOAD @1\n");
			printf("JUMP %i\n", dyn_jump_adr);
			
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
	
	n = fns.head->next;
	while(n) {
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
	printf("NOP ; BUILTIN JUMP @0\n");
	
	int_list_node *n = dyn_jumps;
	int sum = 0;
	
	while(n) {
		printf("SUB #%i\n", n->value - sum);
		printf("JUMZ %i\n", n->value);
		
		sum += n->value;
		n = n->next;
	}
	
	printf("STOP\n");
}

static void print_fn_locations() {
	printf("{\n");
	
	fn_location_node *n = &fn_locations;
	do {
		printf("\t%s: %i\n", n->value->tag_fn.identifier, n->adr);
		n = n->next;
	}
	while(n);
	
	printf("}\n");
}

static void print_int_list(int_list_node *head) {
	if(!head) {
		printf("{}\n");
	}
	else {
		printf("{%i", head->value);
		
		int_list_node *n = head->next;
		while(n) {
			printf(", %i", n->value);
			n = n->next;
		}
		
		printf("}\n");
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
		printf("STOP\n");
		fprintf(stderr, "avertissement: le fichier source est vide\n");
		exit(1);
	}
	
	printf("LOAD #4\n");
	printf("STORE 1\n");
	
	int ip = 2;
	
	allocate_fn_space(fns, ip);
	
	asa_list_node *n = fns.head;
	while(n) {
		codegen_nc(n->value, &ip);
		
		n = n->next;
	}
	
	codegen_dyn_jump();
	destroy_fn_space();
}
