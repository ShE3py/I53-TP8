#include "ts.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * La table de symboles courante.
 */
static symbol_table *cst = NULL;

/**
 * Créer une nouvelle table vide de symboles.
 */
symbol_table* st_empty() {
	symbol_table *st = malloc(sizeof(symbol_table));
	st->head = NULL;
	st->mem_offset = 0;
	
	return st;
}

/**
 * Définie une table de symboles comme étant celle courante.
 */
void st_make_current(symbol_table *st) {
	cst = st;
}

/**
 * Créer et active une nouvelle table vide de symboles, et renvoie la table de symboles précédemment courante.
 */
symbol_table* st_pop_push_empty() {
	symbol_table *old = cst;
	
	cst = st_empty();
	return old;
}

/**
 * Enregistre un nouveau symbole dans une table de symboles.
 */
static symbol st_create_symbol(symbol_table *st, const char id[32], int size) {
	if(!st) {
		fprintf(stderr, "called `st_create_symbol()` with `st == NULL`\n");
		exit(1);
	}
	
	symbol_table_node *n = malloc(sizeof(symbol_table_node));
	strcpy(n->value.identifier, id);
	n->value.base_adr = st->mem_offset;
	n->value.size = size;
	n->next = NULL;
	
	symbol_table_node *m = st->head;
	if(!m) {
		st->head = n;
	}
	else {
		while(1) {
			if(strcmp(m->value.identifier, id) == 0) {
				extern const char *infile;
				extern int yylineno;
				
				fprintf(stderr, "%s:%i: variable dupliquée: '%s'\n", infile, yylineno, id);
				free(n);
				exit(1);
			}
			
			if(!m->next) {
				m->next = n;
				break;
			}
			
			m = m->next;
		}
	}
	
	st->mem_offset += (size == SCALAR_SIZE) ? 1 : size;
	return n->value;
}

/**
 * Enregistre un nouveau scalaire dans la table de symboles courante.
 */
symbol st_create_scalar(const char id[32]) {
	return st_create_symbol(cst, id, SCALAR_SIZE);
}

/**
 * Enregistre un nouveau tableau statique dans la table de symboles courante.
 */
symbol st_create_array(const char id[32], int size) {
	if(size < 0) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: '%s' doit avoir une taille positive\n", infile, yylineno, id);
		exit(1);
	}
	
	return st_create_symbol(cst, id, size);
}

/**
 * Renvoie le symbole avec l'identifiant spécifié présumé dans la table de symboles courante, ou
 * `NULL` si le symbole n'existe pas.
 */
symbol* st_find(const char id[32]) {
	if(!cst) {
		fprintf(stderr, "no current st\n");
		exit(1);
	}
	
	symbol_table_node *n = cst->head;
	while(n) {
		if(strcmp(n->value.identifier, id) == 0) {
			return &n->value;
		}
		
		n = n->next;
	}
	
	return NULL;
}

/**
 * Créer ou renvoie un nouveau scalaire dans la table de symboles courante.
 */
symbol st_find_or_create_scalar(const char id[32]) {
	symbol *s = st_find(id);
	if(s) {
		if(s->size != SCALAR_SIZE) {
			extern const char *infile;
			extern int yylineno;
			
			fprintf(stderr, "%s:%i: '%s' doit être un scalaire\n", infile, yylineno, id);
			exit(1);
		}
		
		return *s;
	}
	else {
		return st_create_scalar(id);
	}
}

/**
 * Créer ou renvoie un nouveau tableau dans la table de symboles courantes.
 * Écrit un message d'erreur puis quitte le programme si jamais le tableau existe déjà
 * et que la taille ne correspond pas au paramètre de cette fonction.
 */
symbol st_find_or_create_array(const char id[32], int size) {
	symbol *s = st_find(id);
	if(s) {
		if(s->size != size) {
			extern const char *infile;
			extern int yylineno;
			
			if(size < 0) {
				fprintf(stderr, "%s:%i: '%s' doit avoir une taille positive\n", infile, yylineno, id);
			}
			else {
				fprintf(stderr, "%s:%i: '%s' doit être un tableau de taille %i, taille actuelle: %i\n", infile, yylineno, id, size, s->size);
			}
			
			exit(1);
		}
		
		return *s;
	}
	else {
		return st_create_array(id, size);
	}
}

/**
 * Renvoie le symbole avec l'identifiant spécifié présumé dans la table de symboles courante, ou
 * écrit un message d'erreur puis quitte le programme si le symbole n'existe pas.
 *
 * Cette fonction doit être appelée pendant la création de l'asa.
 */
symbol st_find_or_yyerror(const char id[32]) {
	symbol *s = st_find(id);
	if(!s) {
		extern const char *infile;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: variable inconnue: '%s'\n", infile, yylineno, id);
		exit(1);
	}
	
	return *s;
}

/**
 * Renvoie le symbole avec l'identifiant spécifié présumé dans la table de symboles courante, ou
 * écrit un message d'erreur puis quitte le programme si le symbole n'existe pas.
 */
symbol st_find_or_internal_error(const char id[32]) {
	symbol *s = st_find(id);
	if(!s) {
		fprintf(stderr, "illegal state: '%s' should exists at this stage but it does not\n", id);
		exit(1);
	}
	
	return *s;
}

/**
 * Renvoie l'adresse en mémoire de la première variable intermédiaire.
 */
int st_temp_offset() {
	if(!cst) {
		fprintf(stderr, "no current st\n");
		exit(1);
	}
	
	return cst->mem_offset;
}

/**
 * Affiche la table de symboles actuelle.
 */
void st_print_current() {
	st_print(cst);
}

/**
 * Affiche une table de symboles.
 */
void st_print(symbol_table *st) {
	if(!st) {
		printf("NULL\n");
	}
	else {
		symbol_table_node *n = st->head;
		if(!n) {
			printf("{ }\n");
		}
		else {
			printf("{ %s", n->value.identifier);
			
			while(n->next) {
				n = n->next;
				
				printf(", %s", n->value.identifier);
			}
			
			printf(" }\n");
		}
	}
}

/**
 * Libère la mémoire allouée à la table de symboles courante.
 */
void st_destroy_current() {
	st_destroy(cst);
}

/**
 * Libère la mémoire allouée à une table de symboles.
 */
void st_destroy(symbol_table *st) {
	if(!st) {
		return;
	}
	else if(st == cst) {
		cst = NULL;
	}
	
	symbol_table_node *n = st->head;
	while(n) {
		symbol_table_node *m = n->next;
		free(n);
		n = m;
	}
	
	free(st);
}
