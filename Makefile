SHELL = /bin/bash
CC = gcc

SRC = src
OUT = out

OBJS = ts.o asa.o codegen.o parser.o lexer.o

CFLAGS = -Wall -Werror=switch -g -I$(SRC)
LFLAGS = -lfl

mkdirs = $(OUT)/grass

all: $(mkdirs) arc

$(mkdirs):
	mkdir $(OUT)/
	touch $@

arc: $(addprefix $(OUT)/,$(OBJS))
	$(CC) $^ -o $@ $(LFLAGS)

$(OUT)/%.o: $(SRC)/%.c $(SRC)/%.h
	$(CC) -c $< $(CFLAGS) -o $@

$(OUT)/%.o: $(OUT)/%.c
	$(CC) -c $< $(CFLAGS) -o $@

$(OUT)/parser.c: $(SRC)/parser.y
	bison $< -d -o $@

$(OUT)/lexer.c: $(SRC)/lexer.lex $(OUT)/parser.h
	flex -o $@ $<

clean:
	rm -rf $(OUT)
	rm -f arc
