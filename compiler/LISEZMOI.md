Un compilateur langage algorithmique → machine RAM.  
Flex est utilisé pour l'analyse lexicale, et Bison pour l'analyse syntaxique.

Les actions sémantiques dans Bison construisent l'arbre syntaxique abstrait, et font
l'analyse sémantique pendant la création des nœuds.

L'abre est ensuite converti en instructions RAM via un `codegen(...)`, qui fait lui
aussi quelques éléments de l'analyse sémantique, en vérifiant par exemple que toutes les
fonctions existent (à l'instar d'un éditeur de liens) et que la fonction principale
existe.

Le langage ignore les espaces, cependant ne pas en mettre sur les opérations risque
de causer des problèmes :
```
1-1
```
Car l'analyse lexicale produira `[(Nombre, 1), (Nombre, -1)]`, c.-à-d. la négation et
non la soustraction.

### Utilisation du programme
```bash
make
./arc fichier > a.out
```

### Description des fichiers

- `src/asa.[ch]`: l'arbre syntaxique abstrait, et analyse sémantique pendant la
  construction de celui-ci.
- `src/codegen.[ch]`: génération du code à partir d'un ASA.
- `src/ts.[ch]`: table des symboles, uniquement les variables.
- `src/parser.y`: l'analyse syntaxique
- `src/lexer.lex`: l'analyse lexicale

### Structure de la mémoire

Registre 0: accumulateur  
Registre 1: pointeur sur le haut de la pile actuelle (inclusif)  
Registre 2: pointeur sur le bas de la pile actuelle (exclusif), variables intermédiaires y compris  
Registre 3: adresse d'écriture d'une variable  
Registre 4+: pile dynamique

Le registre 1 est modifié à chaque appel/retour de fonction.  
Le registre 2 est modifié à chaque apparition/disapparition d'une variable temporaire.  
Le registre 3 est utilisé pour les affectations, notamment lorsque l'adresse est
calculée dynamiquement, l'ACC est stocké dans `R[3]`, puis la nouvelle valeur est calculée
dans l'ACC et stockée ensuite via `STORE @3`.

Les variables sont stockées à partir du registre 4.

### Gestion des variables

Toutes les variables doivent être déclarées avant leur premier usage, et ne peuvent
pas être redéclarées.

#### Scalaires

Un scalaire `x` se déclare comme suit :
```
VAR x;
```

Il est possible d'affecter une valeur pendant ou après la déclaration du scalaire :
```
VAR x := 0;
x := 12;
```

L'opérateur `<-` peut être utilisé à la place de `:=`.

Lire un scalaire sur la bande d'entrée déclare implicitement la variable si
celle-ci n'a pas encore été déclarée :
```
LIRE x;
```

L'on peut aussi afficher une expression :
```
AFFICHER x + 1;
```

La portée d'une variable est toute la fonction :
```
SI 1 == 1 ALORS
    VAR x := 12;
FSI

AFFICHER x;  # 12
```

#### Tableaux

Les tableaux statiques peuvent se déclarer comme suit :
```
VAR t[5];
```

La taille doit être un nombre entier (pas d'expressions, même constantes).  
Dû au fait que la grammaire ne soit pas typée, certaines syntaxes sont modifiées
pour les tableaux :
```
# Lit cinq entiers sur la bande d'entrée et les stockes dans t
LIRE [t];

# Affiche les cinq entiers de t sur la bande de sortie
AFFICHER [t];
```

Les tableaux peuvent aussi s'indexer manuellement :
```
AFFICHER t[0];
LIRE t[1];
```

Il est possible de déclarer un tableau pendant un `LIRE` avec cette syntaxe :
```
LIRE[3] s;
```

Il est possible de récupérer la taille d'un tableau avec la méthode `len()` :
```
AFFICHER s.len();  # 3
```

Et d'affecter directement les valeurs d'un tableau :
```
s := { 1, 2, 3 };
VAR u := { 1, 2 };
```

Les deux tailles doivent évidemment correspondre, et sont vérifiées pendant
l'analyse sémantique.

### Structures conditionnelles

```
SI expr ALORS
    ...
FSI
```
```
SI expr ALORS
    ...
SINON
    ...
FSI
```

```
TQ expr FAIRE
    ...
FTQ
```

### Opérateurs logiques

- `NON expr`
- `expr ET expr`
- `expr OU expr`
- `expr OU EXCLUSIF expr`

Les `ET` et `OU` logiques sont short-circuiting, c.-à-d. que `FAUX ET expr` et
`VRAI ou expr` n'évalueront pas `expr` mais s'évalueront immédiatement à
respectivement `FAUX` et `VRAI`.

### Fonctions

Les fonctions se définissent comme suit :
```
FONCTION g()
DÉBUT
    RENVOYER;
FIN

FONCTION f(x, y)
DÉBUT
    RENVOYER x + y;
FIN
```

Ne pas faire `RENVOYER` fait que le programme se `QUIT` à la fin de la fonction.  
Toutes les fonctions renvoient une valeur, soit l'expression spécifiée, soit `0`.

Les fonctions s'appellent classiquement :
```
AFFICHER h();
```

Utiliser une fonction qui n'est pas déclarée affichera une erreur pendant l'analyse
sémantique.

La fonction principale s'appelle `main` et n'a aucun paramètre. Les fonctions
peuvent être définies dans n'importe quel ordre.

### Détails sur la pile

#### Variables

Chaque variable est associée à une adresse de base, et la lecture d'une variable est
généralement :
1. Lecture du pointeur de pile `R[1]`
2. Ajout de l'adresse de base
3. Chargement de la valeur

Par exemple, pour `x` avec `adr_base` = `0` :
```
LOAD 1
ADD #0
LOAD @0
```

#### Variables temporaires

Les variables temporaires suivent le même principe, sauf qu'elles utilisent le bas
de la pile `R[2]` :
```
DEC 2
LOAD @2
```

#### Appels de fonctions

1. On empile le pointeur de pile `R[1]`
2. On empile l'adresse de retour
3. On empile les paramètres
4. On descend le pointeur de pile sur la fin de la pile `R[1] := R[2]`
5. On `JUMP` sur l'adresse de la fonction appelée.

#### Prélude chez la fonction appelée

1. On recalcule la fin de la pile `R[2] := R[1] + #variables`

Les paramètres d'une fonction ont toujours les premières adresses `0`...`#params`,
ce qui fait que les variables empilées sont déjà aux adresses qu'elles devraient
être.

#### Retour de fonction (chez la fonction appelée)

1. On empile la valeur de retour
2. On monte le pointeur de pile d'un élément vers le haut `R[1] := R[1] - 1`
3. On charge le sommet de la pile, qui est l'adresse de retour `R[R[1]]`
4. On saute à cette instruction.

#### Retour de fonction (chez la fonction appelante)

1. On monte la fin de la pile de trois éléments `R[2] := R[2] - 3`
2. On rétabli notre sommet de pile `R[1] := R[R[2]]`
3. On charge la valeur de retour `R[R[2] + 3]`

#### Sauts dynamiques

La machine RAM ne possède pas d'instruction pour effectuer un saut dynamique, ma
solution a été de simplement enregistrer toutes les adresses de retour possibles dans
une liste, puis après de générer une procédure s'occupant du saut.

Exemple pour les adresses 25 et 83 :
```
NOP ; BUILTIN JUMP @0
SUB #25
JUMZ 25
SUB #58
JUMZ 83
STOP ; UNREACHABLE
```

#### NoOps

Les tableaux de taille zéro produisent des NoOps, c'est-à-dire aucune instruction.  
Les NoOps sont contagieux, c.-à-d. que `NoOp + expr` = `NoOp`.

### Exemples

Des exemples sont disponibles dans le dossier `examples/` ;

- `sum.algo`: la somme des nombres d'un tableau à 3 cases.
- `max.algo`: le plus grand nombre parmi les 5 premiers nombres sur la bande d'entrée.
- `bubble_sort.algo`: un tri à bulles sur les 5 premiers nombres de la bande d'entrée,
  le résultat est ensuite écrit sur la bande de sortie.
- `sub.algo`: une fonction qui renvoie la différence entre ses deux paramètres.
- `pow.algo`: une fonction d'exponentiation rapide.
- `fibo.algo`: une fonction de Fibonacci récursive.

Tous les exemples peuvent être compilés vers le dossier `out/ram/` avec la commande suivante :
```bash
make examples
```
