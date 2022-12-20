#ifndef ASA_H
#define ASA_H

#include <string.h>
#include <stdio.h>
#include <stdlib.h>

#include "ts.h"

typedef enum {typeNb, typeOp, typeAffect, typeBloc, typeVar, typeAfficher} typeNoeud;

typedef struct {
  int val;
} feuilleNb;

typedef struct {
  int ope;
  struct asa *noeud[2];
} noeudOp;

typedef struct {
	char id[32];
	struct asa *expr;
} noeudAffect;

typedef struct noeudBloc {
	struct asa *p;   // nonnull,    p.type != typeBloc
	struct asa *svt; // nullable, svt.type == typeBloc
} noeudBloc;

typedef struct {
	char id[32];
} noeudVar;

typedef struct {
	struct asa *expr; // nonnull
} noeudAfficher;

typedef struct asa{
  typeNoeud type;
  int ninst;
 
  union {
    feuilleNb nb;
    noeudOp op;
    noeudAffect affect;
    noeudBloc bloc;
    noeudVar var;
    noeudAfficher af;
  };
  
} asa;

// fonction d'erreur utilisée également par Bison
void yyerror(const char * s);

/*
  Les fonctions creer_<type> construise un noeud de l'arbre
  abstrait du type correspondant et renvoie un pointeur celui-ci
 */
asa* creer_feuilleNb(int value);
asa* creer_noeudOp(int ope, asa *p1, asa *p2);
asa* creer_noeudAffect(const char id[32], asa *expr);
asa* creer_noeudBloc(asa *p, asa *q);
asa* creer_noeudVar(const char id[32]);
asa* creer_noeudAfficher(asa *expr);

void print_asa(asa *p);
void free_asa(asa *p);

// produit du code pour la machine RAM à partir de l'arbre abstrait
// ET de la table de symbole
void codegen(asa *p);

extern ts * tsymb;

#endif
