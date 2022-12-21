#ifndef TS_H
#define TS_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define PILE_MAX 32

typedef struct ts{
  char *id; // la chaine de car de l'ID
  int adr;  // l'adresse mémoire de l'ID (dans la machine RAM)
  int size; // le nombre de cases mémoires allouées à l'ID, -1 pour un sclaire.
  struct ts *next;
} ts;   

extern ts * tsymb;

void ts_ajouter_scalaire(const char *id);
void ts_ajouter_tableau(const char *id, int size);

// retourne un pointeur vers le noeud contenant l'id <id>, 0 sinon
ts * ts_retrouver_id(const char *id);
// libere la mémoire de la table de symbole
void ts_free_table();
// affiche la table dans la sortie standard
void ts_print();


#endif
