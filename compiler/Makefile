SHELL = /bin/bash
CC = clang
CXX = clang++

SRC = src
OUT = out

OBJS = ts.o asa.o llvm/lowering.o llvm/codegen.o ram/codegen.o parser.o lexer.o

OPT_LEVEL ?= 0
CFLAGS = -Wall -Wextra -Werror -g -O$(OPT_LEVEL) -I$(SRC)
CXXFLAGS = $(shell llvm-config --cppflags)
LFLAGS = -lfl -lLLVM-18

mkdirs = $(OUT)/grass

all: $(mkdirs) arc

$(mkdirs):
	mkdir -p $(OUT)/ram/
	mkdir -p $(OUT)/llvm/
	touch $@

arc: $(addprefix $(OUT)/,$(OBJS)) $(OUT)/driver.o
	$(CXX) $^ -o $@ $(LFLAGS)

$(OUT)/libarc.a: $(addprefix $(OUT)/,$(OBJS))
	ar rcs $@ $^

$(OUT)/%.o: $(SRC)/%.c $(SRC)/%.h
	$(CC) -c $< $(CFLAGS) -o $@

$(OUT)/%.o: $(SRC)/%.cxx $(SRC)/%.hxx
	$(CXX) -c $< $(CFLAGS) $(CXXFLAGS) -o $@

$(OUT)/%.o: $(SRC)/%.cxx $(SRC)/%.h
	$(CXX) -c $< $(CFLAGS) $(CXXFLAGS) -o $@

$(OUT)/%.o: $(OUT)/%.c
	$(CC) -c $< $(CFLAGS) -o $@

$(OUT)/driver.o: $(SRC)/driver.c
	$(CC) -c $< $(CFLAGS) -o $@

$(OUT)/parser.c: $(SRC)/parser.y
	bison $< -d -o $@

$(OUT)/lexer.c: $(SRC)/lexer.lex $(OUT)/parser.h
	flex -o $@ $<

examples: $(addprefix $(OUT)/ram/, sum.out max.out bubble_sort.out sub.out pow.out fibo.out)

out/ram/%.out: examples/%.algo arc $(mkdirs)
	./arc $< > $@

clean:
	rm -rf $(OUT)
	rm -f arc
