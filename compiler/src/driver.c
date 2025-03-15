#include <stdio.h>
#include <string.h>
#include <unistd.h>

extern void arc_compile_file(const char *infile, const char *outfile);

int main(int argc, char *argv[]) {
    if(argc == 0) {
        return 1;
    }

    const char *outfile = "a.out";
    
    int opt;
    while((opt = getopt(argc, argv, "o:")) != -1) {
        switch(opt) {
            case 'o':
                outfile = strdup(optarg);
                break;
            
            default:
                return 1;
        }
    }
    
    if(optind == argc) {
        fprintf(stderr, "%s infile [-o outfile]\n", argv[0]);
        return 1;
    }
    
    const char *infile = argv[optind];
    arc_compile_file(infile, outfile);
}

