#ifndef RAM_CODEGEN_H
#define RAM_CODEGEN_H

#include "asa.h"

/**
 * Génère le code pour la machine RAM correspondant au programme spécifié.
 */
void codegen_ram(asa_list fns);

/**
 * Génère le code pour la machine RAM correspondant au programme spécifié.
 */
void codegen_ram_asa(asa *p);

#endif // RAM_CODEGEN_H

