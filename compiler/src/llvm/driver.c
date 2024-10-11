#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>

static size_t i = 0;

int16_t _Z50READ() {
    printf("E%zu = ", i);
    fflush(stdout);
    ++i;
    
    int16_t v = 0;
    scanf("%" SCNi16, &v);
    return v;
}

void _Z60WRITE(int16_t v) {
    printf("%" PRIi16 "\n", v);
}

extern int16_t _Z4main();

int main() {
    return _Z4main();
}

