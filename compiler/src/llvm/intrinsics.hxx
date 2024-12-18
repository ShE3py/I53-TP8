#ifndef RAME_LLVM_INTRINSICS_HXX
#define RAME_LLVM_INTRINSICS_HXX

// Instrincts starts with `0` as users can't define identifiers startings with a digit.

namespace intrinsics {
	static const char *const WRITE = "0WRITE";
	static const char *const READ = "0READ";
}

#endif // RAME_LLVM_INTRINSICS_HXX
