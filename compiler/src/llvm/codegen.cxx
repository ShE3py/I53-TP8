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
#include <llvm/Analysis/LoopAnalysisManager.h>
#include <llvm/Analysis/CGSCCPassManager.h>
#include <llvm/Passes/StandardInstrumentations.h>
#include <llvm/Passes/PassBuilder.h>
#include <llvm/Transforms/InstCombine/InstCombine.h>
#include <llvm/Transforms/Scalar/GVN.h>
#include <llvm/Transforms/Scalar/Reassociate.h>
#include <llvm/Transforms/Scalar/SimplifyCFG.h>

#include "lowering.hxx"

static std::unique_ptr<llvm::LLVMContext> llvmContext;
static std::unique_ptr<llvm::IRBuilder<>> llvmIrBuilder;
static std::unique_ptr<llvm::Module> llvmModule;
static std::unique_ptr<llvm::FunctionPassManager> llvmFpm;
static std::unique_ptr<llvm::FunctionAnalysisManager> llvmFam;
static llvm::IntegerType *ty;

// Intrinsics
static llvm::Function *WRITE, *READ;

// Variables
static std::map<std::string, llvm::AllocaInst*> locals;

llvm::Value* codegen_nc(const hir::asa &p) {
    switch(p.index()) {
	    case hir::tag_index_v<hir::TagInt>: {
	        return llvm::ConstantInt::get(ty, std::get<hir::TagInt>(p).value, true);
	    }
	    
	    case hir::tag_index_v<hir::TagVar>: {
	        auto node = std::get<hir::TagVar>(p);
	        
	        llvm::AllocaInst *A;
	        if(!(A = locals[node.identifier.c_str()])) {
	            std::cerr << "illegal state: '" << node.identifier << "' should exists at this stage but it does not" << std::endl;
		        exit(1);
	        }
	        
	        return llvmIrBuilder->CreateLoad(ty, A);
	    }
	    
	    case hir::tag_index_v<hir::TagIndex>: {
	        auto const &node = std::get<hir::TagIndex>(p);
	        
	        llvm::AllocaInst *A;
	        if(!(A = locals[node.identifier.c_str()])) {
	            std::cerr << "illegal state: '" << node.identifier << "' should exists at this stage but it does not" << std::endl;
		        exit(1);
	        }
	        
	        llvm::Value *index = llvmIrBuilder->CreateGEP(ty, A, codegen_nc(*node.index));
	        return llvmIrBuilder->CreateLoad(ty, index);
	    }
	    
	    case hir::tag_index_v<hir::TagBinaryOp>: {
	        auto const &node = std::get<hir::TagBinaryOp>(p);
	        llvm::Value *lhs = codegen_nc(*node.lhs);
	        llvm::Value *rhs = codegen_nc(*node.rhs);
	        
	        switch(node.op) {
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
        
        case hir::tag_index_v<hir::TagAssignScalar>: {
	        auto const &node = std::get<hir::TagAssignScalar>(p);
	        
            llvm::AllocaInst *A;
	        if(!(A = locals[node.identifier.c_str()])) {
	            std::cerr << "illegal state: '" << node.identifier << "' should exists at this stage but it does not" << std::endl;
		        exit(1);
	        }
	        
	        return llvmIrBuilder->CreateStore(codegen_nc(*node.expr), A);
        }
        
        case hir::tag_index_v<hir::TagAssignIndexed>: {
	        auto const &node = std::get<hir::TagAssignIndexed>(p);
	        
            llvm::AllocaInst *A;
	        if(!(A = locals[node.identifier.c_str()])) {
	            std::cerr << "illegal state: '" << node.identifier << "' should exists at this stage but it does not" << std::endl;
		        exit(1);
	        }
	        
	        llvm::Value *index = llvmIrBuilder->CreateGEP(ty, A, codegen_nc(*node.index));
	        return llvmIrBuilder->CreateStore(codegen_nc(*node.expr), index);
        }
        
        case hir::tag_index_v<hir::TagBlock>: {
	        auto const &node = std::get<hir::TagBlock>(p);
            
            for(const std::unique_ptr<hir::asa> &q : node.body) {
                codegen_nc(*q);
            }
            
            return nullptr;
        }
	    
	    case hir::tag_index_v<hir::TagFn>: {
	        auto const &node = std::get<hir::TagFn>(p);
	        
	        llvm::Function *F = llvmModule->getFunction(node.identifier);
	         if(!F) {
                llvm::errs() << "internal error: " << node.identifier << " was not created\n";
                exit(1);
            }
	        
	        llvm::BasicBlock *bb = llvm::BasicBlock::Create(*llvmContext, F->getName(), F);
            llvmIrBuilder->SetInsertPoint(bb);
	        
	        for(const std::string &param : node.params) {
	            locals[param] = llvmIrBuilder->CreateAlloca(ty, nullptr, param);
	        }
	        
	        symbol_table_node *n = node.st->head;
	        while(n) {
	            llvm::Type *Ty = (n->value.size == SCALAR_SIZE) ?
	                (llvm::Type*) ty :
	                (llvm::Type*) llvm::ArrayType::get(ty, n->value.size);
	            
	            locals[n->value.identifier] = llvmIrBuilder->CreateAlloca(Ty, nullptr, n->value.identifier);
	            n = n->next;
	        }
	        
	        auto const &block = std::get<hir::TagBlock>(*node.body);
	        for(const std::unique_ptr<hir::asa> &q : block.body) {
                codegen_nc(*q);
            }
	        
	        if(block.body.back()->index() != hir::tag_index_v<hir::TagReturn>)
	            llvmIrBuilder->CreateRet(llvm::ConstantInt::get(ty, 0, true));
	        
	        if(llvm::verifyFunction(*F, &llvm::errs())) {
	            F->print(llvm::errs());
                exit(1);
	        }
	        
	        llvmFpm->run(*F, *llvmFam);
	        return F;
        }
        
        case hir::tag_index_v<hir::TagFnCall>: {
	        auto const &node = std::get<hir::TagFnCall>(p);
	        
            llvm::Function *F = llvmModule->getFunction(node.identifier);
            if(!F) {
                llvm::errs() << "unknown function: " << node.identifier << "\n";
                exit(1);
            }
            
            if(F->arg_size() != node.args.size()) {
				fprintf(stderr, "'%s()': %lu paramètres attendus, %lu paramètres donnés\n", node.identifier.c_str(), F->arg_size(), node.args.size());
				exit(1);
			}
			
			std::vector<llvm::Value*> args;
			args.reserve(F->arg_size());
			for(const std::unique_ptr<hir::asa> &arg : node.args) {
			    // promote i1 to ty
			    args.push_back(llvmIrBuilder->CreateIntCast(codegen_nc(*arg), ty, true));
			}
			
			return llvmIrBuilder->CreateCall(F, args);
        }
        
        case hir::tag_index_v<hir::TagReturn>: {
	        auto const &node = std::get<hir::TagReturn>(p);
            return llvmIrBuilder->CreateRet(codegen_nc(*node.expr));
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
            
            llvmFpm = std::make_unique<llvm::FunctionPassManager>();
            auto TheLAM = std::make_unique<llvm::LoopAnalysisManager>();
            llvmFam = std::make_unique<llvm::FunctionAnalysisManager>();
            auto TheCGAM = std::make_unique<llvm::CGSCCAnalysisManager>();
            auto TheMAM = std::make_unique<llvm::ModuleAnalysisManager>();
            auto ThePIC = std::make_unique<llvm::PassInstrumentationCallbacks>();
            auto TheSI = std::make_unique<llvm::StandardInstrumentations>(*llvmContext, true);
            TheSI->registerCallbacks(*ThePIC, TheMAM.get());
            
            llvmFpm->addPass(llvm::InstCombinePass());
            llvmFpm->addPass(llvm::ReassociatePass());
            llvmFpm->addPass(llvm::GVNPass());
            llvmFpm->addPass(llvm::SimplifyCFGPass());
            
            llvm::PassBuilder PB;
            PB.registerModuleAnalyses(*TheMAM);
            PB.registerFunctionAnalyses(*llvmFam);
            PB.crossRegisterProxies(*TheLAM, *llvmFam, *TheCGAM, *TheMAM);
            
            ty = llvm::IntegerType::get(*llvmContext, 16);
            
            // Instrincts starts with `0` as users can't define identifiers startings with a digit.
	        llvm::FunctionType *FT = llvm::FunctionType::get(llvm::Type::getVoidTy(*llvmContext), ty, false);
	        WRITE = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, "intrinsics.WRITE", *llvmModule);
	        
	        FT = llvm::FunctionType::get(ty, false);
	        READ = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, "intrinsics.READ", *llvmModule);
            
            // Creating all functions
            std::vector<std::unique_ptr<hir::asa>> funs;
            funs.reserve(fns.len);
            ast::asa_list_node *n = fns.head;
            while(n) {
                funs.push_back(hir::lower(n->value));
                n = n->next;
            }
            
            for(const std::unique_ptr<hir::asa> &fun : funs) {
                if(const hir::TagFn *fn = std::get_if<hir::TagFn>(&*fun)) {
                    llvm::FunctionType *FT = llvm::FunctionType::get(ty, std::vector<llvm::Type*>(fn->params.size(), ty), false);
	                llvm::Function::Create(FT, llvm::Function::ExternalLinkage, fn->identifier, *llvmModule);
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

