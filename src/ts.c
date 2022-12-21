#include "ts.h"


ts * tsymb = NULL;
static int mem_offset = 33;


void ts_ajouter_id(const char *id, int size)
{
  ts *new = malloc(sizeof(ts));
  new->id = malloc(strlen(id)+1);
  strcpy(new->id, id);
  new->adr = mem_offset;
  new->size = size;
  mem_offset += size != -1 ? size : 1;
  new->next = tsymb;
  tsymb = new;
}

void ts_ajouter_scalaire(const char *id) {
	ts_ajouter_id(id, -1);
}

void ts_ajouter_tableau(const char *id, int size) {
	if(size < 0) {
		extern const char *input;
		extern int yylineno;
		
		fprintf(stderr, "%s:%i: '%s' doit avoir une taille positive\n", input, yylineno, id);
		exit(1);
	}
	
	ts_ajouter_id(id, size);
}

ts* ts_retrouver_id(const char *id)
{
  ts *t = tsymb;
  while (t!=NULL){
    if (strcmp(t->id, id)==0){
      return t;
    }
    t = t->next;
  }
  return (ts*)0;
}

void ts_free_table()
{
  ts *next, *t = tsymb;

  while (t!=NULL){
    next = t->next;
    free(t->id);
    free(t);
    t = next;
  }
}

void ts_print()
{
  ts *t = tsymb;
  if (t!=NULL){
    printf("{%s : %d}", t->id, t->adr);
    t = t->next;
  }
  while (t!=NULL){
    printf("-->{%s : %d}", t->id, t->adr);
    t = t->next;
  }

}
