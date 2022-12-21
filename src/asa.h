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
	 * Une opération d'indexation.
	 */
	TagIndex,
	
	/**
	 * Une opération binaire.
	 */
	TagBinaryOp,
	
	/**
	 * Une opération unaire.
	 */
	TagUnaryOp,
	
	/**
	 * Une affectation.
	 */
	TagAssign,
	
	/**
	 * Une affectation à un élément d'un tableau.
	 */
	TagAssignIndexed,
	
	/**
	 * Une structure si-alors-sinon.
	 */
	TagTest,
	
	/**
	 * Une structure tant que-faire.
	 */
	TagWhile,
	
	/**
	 * La fonction intrinsèque `LIRE`.
	 */
	TagRead,
	
	/**
	 * La fonction intrinsèque `LIRE` sur un élément d'un tableau.
	 */
	TagReadIndexed,
	
	/**
	 * La fonction intrinsèque `LIRE` sur un tableau entier.
	 */
	TagReadArray,
	
	/**
	 * La fonction intrinsèque `AFFICHER`.
	 */
	TagPrint,
	
	/**
	 * La fonction intrinsèque `AFFICHER` sur un tableau.
	 */
	TagPrintArray,
	
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
 * Les types d'opérateurs.
 */
typedef enum {
	Arithmetic,
	Comparative,
	Logic
} OpKind;


/**
 * Un opérateur binaire.
 */
typedef enum {
	OpAdd,
	OpSub,
	OpMul,
	OpDiv,
	OpMod,
	
	OpGe,
	OpGt,
	OpLe,
	OpLt,
	OpEq,
	OpNe,
	
	OpAnd,
	OpOr,
	OpXor
} BinaryOp;

/**
 * Renvoie le symbole associé à un opérateur binaire.
 */
const char* binop_symbol(BinaryOp binop);

/**
 * Renvoie le type d'un opérateur binaire.
 */
OpKind binop_kind(BinaryOp binop);


/**
 * Un opérateur unaire.
 */
typedef enum {
	/**
	 * La négation arithmétique.
	 */
	OpNeg,
	
	/**
	 * La négation logique.
	 */
	OpNot
} UnaryOp;

/**
 * Renvoie le symbole associé à un opérateur unaire.
 */
const char* unop_symbol(UnaryOp unop);

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
		 * La valeur d'un noeud `TagIndex`.
		 */
		struct {
			/**
			 * L'identifiant de la variable.
			 */
			char identifier[32];
			
			/**
			 * L'indice.
			 */
			struct asa *index;
		} tag_index;
		
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
		 * La valeur d'un noeud `TagUnaryOp`.
		 */
		struct {
			/**
			 * L'opérateur unaire.
			 */
			UnaryOp op;
			
			/**
			 * L'opérande.
			 */
			struct asa *expr;
		} tag_unary_op;
		
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
		 * La valeur d'un noeud `TagAssignIndexed`.
		 */
		struct {
			/**
			 * L'identifiant du tableau à modifier.
			 */
			char identifier[32];
			
			/**
			 * L'indice de l'élément à modifier.
			 */
			struct asa *index;
			
			/**
			 * L'expression à évaluer.
			 */
			struct asa *expr;
		} tag_assign_indexed;
		
		/**
		 * La valeur d'un noeud `TagTest`.
		 */
		struct {
			/**
			 * L'expression à tester.
			 */
			struct asa *expr;
			
			/**
			 * Les instructions à exécuter si le test a réussi.
			 */
			struct asa *therefore; // nullable
			
			/**
			 * Les instructions à exécuter si le test a échoué.
			 */
			struct asa *alternative;  // nullable
		} tag_test;
		
		/**
		 * La valeur d'un noeud `TagWhile`.
		 */
		struct {
			/**
			 * L'expression à tester.
			 */
			struct asa *expr;
			
			/**
			 * Les instructions à exécuter dans le corps de la boucle.
			 */
			struct asa *body; // nonnull
		} tag_while;
		
		/**
		 * La valeur d'un noeud `TagRead`.
		 */
		struct {
			/**
			 * L'identifiant de la variable receveuse.
			 */
			char identifier[32];
		} tag_read;
		
		/**
		 * La valeur d'un noeud `TagReadIndexed`.
		 */
		struct {
			/**
			 * L'identifiant du tableau à modifier.
			 */
			char identifier[32];
			
			/**
			 * L'indice dans le tableau.
			 */
			struct asa *index;
		} tag_read_indexed;
		
		/**
		 * La valeur d'un noeud `TagReadArray`.
		 */
		struct {
			/**
			 * L'identifiant du tableau receveur.
			 */
			char identifier[32];
		} tag_read_array;
		
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
		 * La valeur d'un noeud `TagPrintArray`.
		 */
		struct {
			/**
			 * L'identifiant du tableau à afficher.
			 */
			char identifier[32];
		} tag_print_array;
		
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
 * Créer un nouveau noeud `TagIndex` avec les valeurs spécifiées.
 */
asa* create_index_node(const char id[32], asa *index);

/**
 * Créer un nouveau noeud `TagBinaryOp` avec les valeurs spécifiées.
 */
asa* create_binop_node(BinaryOp binop, asa *lhs, asa *rhs);

/**
 * Créer un nouveau noeud `TagUnaryOp` avec les valeurs spécifiées.
 */
asa* create_unop_node(UnaryOp unop, asa *expr);

/**
 * Créer un nouveau noeud `TagAssign` avec les valeurs spécifiées.
 */
asa* create_assign_node(const char id[32], asa *expr);

/**
 * Créer un nouveau noeud `TagAssignIndexed` avec les valeurs spécifiées.
 */
asa* create_assign_indexed_node(const char id[32], asa *index, asa *expr);

/**
 * Créer un nouveau noeud `TagTest` avec les valeurs spécifiées.
 */
asa* create_test_node(asa *expr, asa *therefore, asa *alternative);

/**
 * Créer un nouveau noeud `TagWhile` avec les valeurs spécifiées.
 */
asa* create_while_node(asa *expr, asa *body);

/**
 * Créer un nouveau noeud `TagRead` avec l'identifiant spécifié.
 */
asa* create_read_node(const char id[32]);

/**
 * Créer un nouveau noeud `TagReadIndexed` avec les valeurs spécifiées.
 */
asa* create_read_indexed_node(const char id[32], asa *index);

/**
 * Créer un nouveau noeud `TagReadArray` avec l'identifiant spécifié.
 */
asa* create_read_array_node(const char id[32]);

/**
 * Créer un nouveau noeud `TagPrint` avec l'expression spécifiée.
 */
asa* create_print_node(asa *expr);

/**
 * Créer un nouveau noeud `TagPrintArray` avec l'identifiant spécifié.
 */
asa* create_print_array_node(const char id[32]);

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

extern ts *tsymb;

#endif
