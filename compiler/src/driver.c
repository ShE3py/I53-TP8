#include <stdio.h>

extern int arc_compile_file(const char *filename);

int main(int argc, char *argv[]) {
    const char *out = "a.out";

	if(argc != 3 && argc != 4) {
	    if(argc > 0) {
		    fprintf(stderr, "%s <input>\n");
		}
		return 1;
	}
	
	return arc_compile_file(argv[1]);
}

