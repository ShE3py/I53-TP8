#include "codegen.h"

#include <memory>
#include <iostream>
#include <map>

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/DerivedTypes.h>

#include "lowering.hxx"

static std::unique_ptr<llvm::LLVMContext> llvmContext;
static std::unique_ptr<llvm::IRBuilder<>> llvmIrBuilder;
static std::unique_ptr<llvm::Module> llvmModule;
static llvm::IntegerType *ty;

// Intrinsics
static llvm::Function *WRITE, *READ;

// Variables
static std::map<std::string, llvm::Value*> locals;

llvm::Value* codegen_nc(const hir::asa &p) {
    switch(p.tag) {
	    case hir::TagInt: {
	        return llvm::ConstantInt::get(ty, p.p.tag_int.value, true);
	    }
	    
	    case hir::TagVar: {
	        symbol *var = st_find(p.p.tag_var.identifier.c_str());
	        llvm::Value *l;
	        if(!var || !(l = locals[var->identifier])) {
	            std::cerr << "illegal state: '" << p.p.tag_var.identifier << "' should exists at this stage but it does not" << std::endl;
		        exit(1);
	        }
	        
	        return l;
	    }
	    
	    case hir::TagBinaryOp: {
	        llvm::Value *lhs = codegen_nc(*p.p.tag_binary_op.lhs);
	        llvm::Value *rhs = codegen_nc(*p.p.tag_binary_op.rhs);
	        
	        switch(p.p.tag_binary_op.op) {
	            case ast::OpAdd: return llvmIrBuilder->CreateAdd(lhs, rhs);
	            case ast::OpSub: return llvmIrBuilder->CreateSub(lhs, rhs);
	            case ast::OpMul: return llvmIrBuilder->CreateMul(lhs, rhs);
	            case ast::OpDiv: return llvmIrBuilder->CreateSDiv(lhs, rhs);
	            case ast::OpMod: return llvmIrBuilder->CreateSRem(lhs, rhs);
	            
	            case ast::OpGe: return llvmIrBuilder->CreateCmp(llvm::CmpInst::Predicate::ICMP_SGE, lhs, rhs);
	            case ast::OpGt: return llvmIrBuilder->CreateCmp(llvm::CmpInst::Predicate::ICMP_SGT, lhs, rhs);
	            case ast::OpLe: return llvmIrBuilder->CreateCmp(llvm::CmpInst::Predicate::ICMP_SLE, lhs, rhs);
	            case ast::OpLt: return llvmIrBuilder->CreateCmp(llvm::CmpInst::Predicate::ICMP_SLT, lhs, rhs);
	            case ast::OpEq: return llvmIrBuilder->CreateCmp(llvm::CmpInst::Predicate::ICMP_EQ, lhs, rhs);
	            case ast::OpNe: return llvmIrBuilder->CreateCmp(llvm::CmpInst::Predicate::ICMP_NE, lhs, rhs);
	            
	            case ast::OpAnd: return llvmIrBuilder->CreateAnd(lhs, rhs);
	            case ast::OpOr: return llvmIrBuilder->CreateOr(lhs, rhs);
	            case ast::OpXor: return llvmIrBuilder->CreateXor(lhs, rhs);
	        }
	        
	        std::cerr << "entered unreachable code" << std::endl;
	        exit(1);
        }
        
        case hir::TagRead: {
            llvm::Value *ret = llvmIrBuilder->CreateCall(READ, std::nullopt, p.p.tag_read.identifier);
            locals[p.p.tag_read.identifier] = ret;
            return nullptr;
        }
        
        case hir::TagPrint: {
            llvm::Value *expr = codegen_nc(*p.p.tag_print.expr);
            return llvmIrBuilder->CreateCall(WRITE, expr);
        }
        
        case hir::TagBlock: {
            llvm::BasicBlock *bb = llvm::BasicBlock::Create(*llvmContext);
            llvmIrBuilder->SetInsertPoint(bb);
            
            for(const std::unique_ptr<hir::asa> &q : p.p.tag_block.body) {
                codegen_nc(*q);
            }
            
            return bb;
        }
	    
	    case hir::TagFn:
	        llvm::FunctionType *FT = llvm::FunctionType::get(ty, std::vector<llvm::Type*>(p.p.tag_fn.params.size(), ty), false);
	        llvm::Function *F = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, p.p.tag_fn.identifier, *llvmModule);
	        
	        st_make_current(p.p.tag_fn.st);
	        locals.clear();
	        
	        llvm::BasicBlock *body = static_cast<llvm::BasicBlock*>(codegen_nc(*p.p.tag_fn.body));
	        body->insertInto(F);
	        
	        return F;
    }
    
    std::cerr << "entered unreachable code" << std::endl;
    exit(1);
}

/**
 * Generate LLVM IR for the specified program.
 */
extern "C" {
    void codegen_llvm(ast::asa_list fns) {
        try {
            llvmContext = std::make_unique<llvm::LLVMContext>();
            llvmModule = std::make_unique<llvm::Module>("", *llvmContext);
            llvmIrBuilder = std::make_unique<llvm::IRBuilder<>>(*llvmContext);
            ty = llvm::IntegerType::get(*llvmContext, 16);
            
	        llvm::FunctionType *FT = llvm::FunctionType::get(llvm::Type::getVoidTy(*llvmContext), ty, false);
	        WRITE = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, "WRITE", *llvmModule);
	        
	        FT = llvm::FunctionType::get(ty, false);
	        READ = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, "READ", *llvmModule);
            
            ast::asa_list_node *n = fns.head;
            while(n) {
                codegen_nc(*hir::lower(n->value))->print(llvm::outs());
                n = n->next;
            }
        }
        catch(const std::exception &e) {
            std::cerr << e.what() << std::endl;
            exit(1);
        }
        catch(...) {
            std::cerr << "unhandled ... exception" << std::endl;
            exit(1);
        }
    }
}

