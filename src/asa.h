#ifndef ASA_H
#define ASA_H

#include <string.h>
#include <stdio.h>
#include <stdlib.h>

#include "ts.h"

/**
 * Les valeurs possibles d'un noeud.
 */
typedef enum {
	/**
	 * Un nombre entier.
	 */
	TagInt,
	
	/**
	 * Une variable.
	 */
	TagVar,
	
	/**
	 * Une opération binaire.
	 */
	TagBinaryOp,
	
	/**
	 * Une affectation.
	 */
	TagAssign,
	
	/**
	 * La fonction intrinsèque `AFFICHER`.
	 */
	TagPrint,
	
	/**
	 * Un bloc d'instructions.
	 */
	TagBlock
} NodeTag;

/**
 * Renvoie `1` si l'étiquette spécifiée est une feuille, sinon
 * renvoie `0`.
 */
int is_leaf(NodeTag tag);


/**
 * Un opérateur binaire.
 */
typedef enum {
	OpAdd,
	OpSub,
	OpMul,
	OpDiv,
	OpMod
} BinaryOp;

/**
 * Renvoie le symbole associé à un opérateur binaire.
 */
const char* binop_symbol(BinaryOp binop);

/**
 * Un noeud d'un arbre syntaxique abstrait.
 */
typedef struct asa {
	/**
	 * L'étiquette du type somme.
	 */
	NodeTag tag;
	
	/**
	 * Le nombre d'instructions en machine RAM
	 * générées par ce noeud.
	 */
	int ninst;
	
	union {
		/**
		 * La valeur d'un noeud `TagInt`.
		 */
		struct {
			/**
			 * La constante.
			 */
			int value;
		} tag_int;
		
		/**
		 * La valeur d'un noeud `TagVar`.
		 */
		struct {
			/**
			 * L'identifiant de la variable.
			 */
			char identifier[32];
		} tag_var;
		
		/**
		 * La valeur d'un noeud `TagBinaryOp`.
		 */
		struct {
			/**
			 * L'opérateur binaire.
			 */
			BinaryOp op;
			
			/**
			 * L'opérande gauche.
			 */
			struct asa *lhs;
			
			/**
			 * L'opérande droite.
			 */
			struct asa *rhs;
		} tag_binary_op;
		
		/**
		 * La valeur d'un noeud `TagAssign`
		 */
		struct {
			/**
			 * L'identifiant de la variable receveuse.
			 */
			char identifier[32];
			
			/**
			 * L'expression à évaluer.
			 */
			struct asa *expr;
		} tag_assign;
		
		/**
		 * La valeur d'un noeud `TagPrint`.
		 */
		struct {
			/**
			 * L'expression à évaluer puis afficher.
			 */
			struct asa *expr;
		} tag_print;
		
		/**
		 * La valeur d'un noeud `TagBlock`.
		 */
		struct {
			/**
			 * L'instruction actuelle.
			 */
			struct asa *stmt; // nonnull, tag != TagBlock
			
			/**
			 * Les instructions suivantes.
			 */
			struct asa *next; // nullable, tag == TagBlock
		} tag_block;
	};
} asa;


/**
 * Créer une nouvelle feuille `TagInt` avec la valeur spécifiée.
 */
asa* create_int_leaf(int value);

/**
 * Créer une nouvelle feuille `TagVar` avec l'identifiant spécifié.
 */
asa* create_var_leaf(const char id[32]);

/**
 * Créer un nouveau noeud `TagBinaryOp` avec les valeurs spécifiées.
 */
asa* create_binop_node(BinaryOp binop, asa *lhs, asa *rhs);

/**
 * Créer un nouveau noeud `TagAssign` avec les valeurs spécifiées.
 */
asa* create_assign_node(const char id[32], asa *expr);

/**
 * Créer un nouveau noeud `TagPrint` avec l'expression spécifiée.
 */
asa* create_print_node(asa *expr);

/**
 * Transforme deux noeuds en un noeud `TagBlock` équivalent.
 */
asa* make_block_node(asa *p, asa *q);

/**
 * Affiche le noeud dans la sortie standard.
 */
void print_asa(asa *p);

/**
 * Libère les ressources allouées à un noeud.
 */
void free_asa(asa *p);

extern ts * tsymb;

// fonction d'erreur utilisée également par Bison
void yyerror(const char * s);

#endif
