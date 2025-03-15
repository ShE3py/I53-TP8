#ifndef TS_H
#define TS_H

#include <stdio.h>

#define SCALAR_SIZE -1

#ifdef __cplusplus
extern "C" {
#endif

/**
 * L'enregistrement d'une variable.
 */
typedef struct {
    /**
     * L'identifiant du symbole.
     */
    char identifier[32];
    
    /**
     * L'adresse mémoire du symbole.
     */
    int base_adr;
    
    /**
     * Le nombre de cellules allouées au symbole dans le cas d'un tableau.
     * `SCALAR_SIZE` pour un scalaire.
     */
    int size;
} symbol;

/**
 * Un élément d'une liste chaînée de symboles.
 */
typedef struct symbol_table_node {
    symbol value;
    struct symbol_table_node *next;
} symbol_table_node;

/**
 * Une table de symboles.
 */
typedef struct {
    /**
     * Le premier élément de la liste chaînée.
     */
    symbol_table_node *head;
    
    /**
     * L'adresse de base en mémoire de la prochaine variable.
     */
    int mem_offset;
} symbol_table;

/**
 * Créer une nouvelle table vide de symboles.
 */
symbol_table* st_empty();

/**
 * Définie une table de symboles comme étant celle courante.
 */
void st_make_current(symbol_table *st);

/**
 * Renvoie la table de symboles actuelle.
 */
symbol_table* st_current();

/**
 * Créer et active une nouvelle table vide de symboles, et renvoie la table de symboles précédemment courante.
 */
symbol_table* st_pop_push_empty();

/**
 * Enregistre un nouveau scalaire dans la table de symboles courante.
 */
symbol st_create_scalar(const char id[32]);

/**
 * Enregistre un nouveau tableau statique dans la table de symboles courante.
 */
symbol st_create_array(const char id[32], int size);

/**
 * Renvoie le symbole avec l'identifiant spécifié présumé dans la table de symboles courante, ou
 * `NULL` si le symbole n'existe pas.
 */
symbol* st_find(const char id[32]);

/**
 * Créer ou renvoie un nouveau scalaire dans la table de symboles courante.
 */
symbol st_find_or_create_scalar(const char id[32]);

/**
 * Créer ou renvoie un nouveau tableau dans la table de symboles courantes.
 * Écrit un message d'erreur puis quitte le programme si jamais le tableau existe déjà
 * et que la taille ne correspond pas au paramètre de cette fonction.
 */
symbol st_find_or_create_array(const char id[32], int size);

/**
 * Renvoie le symbole avec l'identifiant spécifié présumé dans la table de symboles courante, ou
 * écrit un message d'erreur puis quitte le programme si le symbole n'existe pas.
 *
 * Cette fonction doit être appelée pendant la création de l'asa.
 */
symbol st_find_or_yyerror(const char id[32]);

/**
 * Renvoie le symbole avec l'identifiant spécifié présumé dans la table de symboles courante, ou
 * écrit un message d'erreur puis quitte le programme si le symbole n'existe pas.
 */
symbol st_find_or_internal_error(const char id[32]);

/**
 * Renvoie l'adresse en mémoire de la première variable intermédiaire.
 */
int st_temp_offset();

/**
 * Affiche la table de symboles actuelle.
 */
void st_fprint_current(FILE *stream);

/**
 * Affiche une table de symboles.
 */
void st_fprint(FILE *stream, symbol_table *st);

/**
 * Libère la mémoire allouée à la table de symboles courante.
 */
void st_destroy_current();

/**
 * Libère la mémoire allouée à une table de symboles.
 */
void st_destroy(symbol_table *st);

#ifdef __cplusplus
}
#endif

#endif // TS_H

