#include "codegen.h"

#include <memory>
#include <iostream>
#include <map>
#include <sstream>

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/DerivedTypes.h>
#include <llvm/Support/TargetSelect.h>
#include <llvm/MC/TargetRegistry.h>
#include <llvm/TargetParser/Host.h>
#include <llvm/Target/TargetMachine.h>
#include <llvm/Target/TargetOptions.h>
#include <llvm/IR/LegacyPassManager.h>
#include <llvm/IR/Verifier.h>

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
            return llvm::PoisonValue::get(ty);
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
	    
	    case hir::TagFn: {
	        llvm::Function *F = llvmModule->getFunction(p.p.tag_fn.identifier);
	         if(!F) {
                llvm::errs() << "internal error: " << p.p.tag_fn_call.identifier << " was not created\n";
                exit(1);
            }
	        
	        st_make_current(p.p.tag_fn.st);
	        locals.clear();
	        
	        for(size_t i = 0; i < p.p.tag_fn.params.size(); ++i) {
	            locals[p.p.tag_fn.params[i]] = F->getArg(i);
	        }
	        
	        if(llvm::BasicBlock *body = llvm::dyn_cast<llvm::BasicBlock>(codegen_nc(*p.p.tag_fn.body))) {
	            body->insertInto(F);
	        }
	        
	        if(p.p.tag_fn.body->p.tag_block.body.back()->tag != hir::TagReturn)
	            llvmIrBuilder->CreateRet(llvm::ConstantInt::get(ty, 0, true));
	        
	        if(llvm::verifyFunction(*F, &llvm::errs())) {
	            F->print(llvm::errs());
                exit(1);
	        }
	        
	        return F;
        }
        
        case hir::TagFnCall: {
            llvm::Function *F = llvmModule->getFunction(p.p.tag_fn_call.identifier);
            if(!F) {
                llvm::errs() << "unknown function: " << p.p.tag_fn_call.identifier << "\n";
                exit(1);
            }
            
            if(F->arg_size() != p.p.tag_fn_call.args.size()) {
				fprintf(stderr, "'%s()': %lu paramètres attendus, %lu paramètres donnés\n", p.p.tag_fn_call.identifier.c_str(), F->arg_size(), p.p.tag_fn_call.args.size());
				exit(1);
			}
			
			std::vector<llvm::Value*> args;
			args.reserve(F->arg_size());
			for(const std::unique_ptr<hir::asa> &arg : p.p.tag_fn_call.args) {
			    args.push_back(codegen_nc(*arg));
			}
			
			return llvmIrBuilder->CreateCall(F, args);
        }
        
        case hir::TagReturn: {
            return llvmIrBuilder->CreateRet(codegen_nc(*p.p.tag_return.expr));
        }
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
            
            // Instrincts starts with `0` as users can't define identifiers startings with a digit.
	        llvm::FunctionType *FT = llvm::FunctionType::get(llvm::Type::getVoidTy(*llvmContext), ty, false);
	        WRITE = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, "0WRITE", *llvmModule);
	        
	        FT = llvm::FunctionType::get(ty, false);
	        READ = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, "0READ", *llvmModule);
            
            // Creating all functions
            std::vector<std::unique_ptr<hir::asa>> funs;
            funs.reserve(fns.len);
            ast::asa_list_node *n = fns.head;
            while(n) {
                funs.push_back(hir::lower(n->value));
                n = n->next;
            }
            
            for(const std::unique_ptr<hir::asa> &fun : funs) {
                if(fun->tag == hir::TagFn) {
                    llvm::FunctionType *FT = llvm::FunctionType::get(ty, std::vector<llvm::Type*>(fun->p.tag_fn.params.size(), ty), false);
	                llvm::Function::Create(FT, llvm::Function::ExternalLinkage, fun->p.tag_fn.identifier, *llvmModule);
                }
            }
            
            for(const std::unique_ptr<hir::asa> &fun : funs) {
                codegen_nc(*fun);
            }
            
            llvm::outs() << *llvmModule;
            
            if(llvm::verifyModule(*llvmModule, &llvm::errs())) {
                exit(1);
	        }
            
            for(llvm::Function &F : *llvmModule) {
                std::stringstream mangled;
                mangled << "_Z" << F.getName().size() << F.getName().begin();
                F.setName(mangled.str());
            }
            
            llvm::InitializeAllTargetInfos();
            llvm::InitializeAllTargets();
            llvm::InitializeAllTargetMCs();
            llvm::InitializeAllAsmParsers();
            llvm::InitializeAllAsmPrinters();
            
            std::string triple = llvm::sys::getDefaultTargetTriple();
            std::string err;
            const llvm::Target *target = llvm::TargetRegistry::lookupTarget(triple, err);
            if(!target) {
                llvm::errs() << "lookupTarget: " << err << "\n";
                exit(1);
            }
            
            llvm::TargetOptions opt;
            llvm::TargetMachine *machine = target->createTargetMachine(triple, "generic", "", opt, llvm::Reloc::Model::PIC_);
            
            llvmModule->setDataLayout(machine->createDataLayout());
            llvmModule->setTargetTriple(triple);
            
            extern FILE *outfile;
            llvm::raw_fd_ostream dest(fileno(outfile), false);
            
            llvm::legacy::PassManager pass;
            if(machine->addPassesToEmitFile(pass, dest, nullptr, llvm::CodeGenFileType::ObjectFile)) {
                llvm::errs() << "addPassesToEmitFile failed\n";
                exit(1);
            }
            
            pass.run(*llvmModule);
            dest.flush();
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

