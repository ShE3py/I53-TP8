CC = gcc
CFLAGS = -Wall -Werror=switch -g
OBJ = ts.o asa.o parser.o lexer.o

all: arc

arc: $(OBJ)
	$(CC) $(OBJ) -o $@ $(CFLAGS) -lfl 

*.o: *.c
	$(CC) -c $< $(CFLAGS)

parser.h: parser.y

parser.c: parser.y
	bison $< -o $@ -d

lexer.c: lexer.lex parser.h
	flex -o $@ $^

clean:
	rm -f *~ *.o parser.c parser.h lexer.c a.out arc parser.output

