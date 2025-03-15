#ifndef ASA_H
#define ASA_H

#include <stdio.h>

#include "ts.h"

#ifdef __cplusplus
namespace ast {
    extern "C" {
#endif

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
	 * Une affectation d'un scalaire à un scalaire.
	 */
	TagAssignScalar,
	
	/**
	 * Une affectation à un élément d'un tableau.
	 */
	TagAssignIndexed,
	
	/**
	 * Une affectation d'un tableau à une liste d'entiers.
	 */
	TagAssignIntList,
	
	/**
	 * Une affectation d'un tableau à un autre tableau.
	 */
	TagAssignArray,
	
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
	TagBlock,
	
	/**
	 * Une fonction.
	 */
	TagFn,
	
	/**
	 * Un appel de fonction.
	 */
	TagFnCall,
	
	/**
	 * Un retour de fonction.
	 */
	TagReturn
} NodeTag;

/**
 * Renvoie `1` si l'étiquette spécifiée est une feuille, sinon
 * renvoie `0`.
 */
int is_leaf(NodeTag tag);

/**
 * Renvoie l'identifiant C de l'étiquette spécifiée.
 */
const char* tag_name(NodeTag tag);


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
 * Un élément d'une liste chaînée d'expressions.
 */
typedef struct asa_list_node {
	struct asa *value;
	struct asa_list_node *next;
} asa_list_node;

/**
 * Une liste d'expressions, utilisée par `TagAssignIntList`.
 */
typedef struct {
	/**
	 * La longueur de la liste.
	 */
	size_t len;
	
	/**
	 * Le nombre d'instructions générés par tous
	 * les éléments de cette liste.
	 */
	size_t ninst;
	
	/**
	 * Le premier élément de la liste chaînée.
	 */
	asa_list_node *head;
	
	/**
	 * L'ajout d'un élément NoOp transforme toute cette liste en NoOp.
	 */
	int is_nop;
} asa_list;

/**
 * Un élément d'une liste chaînée d'identifiants.
 */
typedef struct id_list_node {
	char value[32];
	struct id_list_node *next;
} id_list_node;

/**
 * Une liste d'identifiants, utilisée par `TagFn`.
 */
typedef struct {
	/**
	 * La longueur de la liste.
	 */
	size_t len;
	
	/**
	 * Le premier élément de la liste chaînée.
	 */
	id_list_node *head;
} id_list;

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
		 * La valeur d'un noeud `TagAssignScalar`
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
		} tag_assign_scalar;
		
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
		 * La valeur d'un noeud `TagAssignIntList`.
		 */
		struct {
			/**
			 * L'identifiant du tableau à modifier.
			 */
			char identifier[32];
			
			/**
			 * La liste d'entiers.
			 */
			asa_list values;
		} tag_assign_int_list;
		
		/**
		 * La valeur d'un noeud `TagAssignArray`.
		 */
		struct {
			/**
			 * L'identifiant du tableau à modifier.
			 */
			char dst[32];
			
			/**
			 * L'identifiant du tableau source;
			 */
			char src[32];
		} tag_assign_array;
		
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
		
		/**
		 * La valeur d'un noeud `TagFn`.
		 */
		struct {
			/**
			 * Le nom de la fonction.
			 */
			char identifier[32];
			
			/**
			 * Les paramètres de la fonction.
			 */
			id_list params;
			
			/**
			 * Le corps de la fonction.
			 */
			struct asa *body;
			
			/**
			 * La table de symboles de la fonction.
			 */
			symbol_table *st;
		} tag_fn;
		
		/**
		 * La valeur d'un noeud `TagFnCall`.
		 */
		struct {
			/**
			 * Le nom de la fonction.
			 */
			char identifier[32];
			
			/**
			 * Les arguments de l'appel.
			 */
			asa_list args;
		} tag_fn_call;
		
		/**
		 * La valeur d'un noeud `TagReturn`.
		 */
		struct {
			/**
			 * L'expression à renvoyer.
			 */
			struct asa *expr;
		} tag_return;
	};
} asa;


/**
 * Un noeud qui ne génèrera aucune instruction.
 * `NOP` est contagieux ; `x + NOP` produira un `NOP`.
 */
static asa *const NOP = (asa *const) sizeof(asa);

/**
 * Créer une nouvelle liste vide.
 */
asa_list asa_list_empty();

/**
 * Créer une nouvelle liste à partir de son premier élément et des éléments suivants.
 */
asa_list asa_list_append(asa *head, asa_list next);

/**
 * Affiche une liste dans un fichier.
 */
void asa_list_fprint(FILE *stream, asa_list l);

/**
 * Libère les ressources allouées à une liste.
 */
void asa_list_destroy(asa_list l);

/**
 * Créer une nouvelle liste à partir de son premier élément et de ses éléments suivants.
 */
id_list id_list_append(const char id[32], id_list next);

/**
 * Créer une nouvelle liste vide.
 */
id_list id_list_empty();

/**
 * Affiche une liste dans un fichier.
 */
void id_list_fprint(FILE *stream, id_list l);

/**
 * Libère les ressources allouées à une liste.
 */
void id_list_destroy(id_list l);

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
 * Créer un nouveau noeud `TagAssignScalar` avec les valeurs spécifiées.
 */
asa* create_assign_scalar_node(const char id[32], asa *expr);

/**
 * Créer un nouveau noeud `TagAssignIndexed` avec les valeurs spécifiées.
 */
asa* create_assign_indexed_node(const char id[32], asa *index, asa *expr);

/**
 * Créer un nouveau noeud `TagAssignIntList` avec les valeurs spécifiées.
 */
asa* create_assign_int_list_node(const char id[32], asa_list values);

/**
 * Créer un nouveau noeud `TagAssignArray` avec les valeurs spécifiées.
 */
asa* create_assign_array_node(const char dst[32], const char src[32]);

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
 * Créer un nouveau noeud correspondant à la méthode spécifiée.
 */
asa* create_methodcall_node(const char varname[32], const char methodname[32]);

/**
 * Créer un nouveau noeud `TagFn` avec les valeurs spécifiées.
 */
asa* create_fn_node(const char id[32], id_list params, asa *body, symbol_table *st);

/**
 * Créer un nouveau noeud `TagFnCall` avec les paramètres spécifiés.
 */
asa* create_fncall_node(const char id[32], asa_list args);

/**
 * Créer un nouveau noeud `TagReturn` avec l'expression spécifiée.
 */
asa* create_return_node(asa *expr);

/**
 * Affiche le noeud dans un fichier.
 */
void fprint_asa(FILE *stream, asa *p);

/**
 * Libère les ressources allouées à un noeud.
 */
void free_asa(asa *p);

#ifdef __cplusplus
    }
}
#endif

#endif // ASA_H

