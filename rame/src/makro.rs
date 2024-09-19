
#[macro_export]
macro_rules! inst {
    (READ) => { $crate::Instruction::Read };
    (WRITE) => { $crate::Instruction::Write };
    (LOAD #$n:literal) => { $crate::Instruction::Load($crate::Value::Constant($n)) };
    (LOAD $n:literal) => { $crate::Instruction::Load($crate::Value::Register($crate::Register::Direct($n))) };
    (LOAD @$n:literal) => { $crate::Instruction::Load($crate::Value::Register($crate::Register::Indirect($n))) };
    (STORE $n:literal) => { $crate::Instruction::Store($crate::Register::Direct($n)) };
    (STORE @$n:literal) => { $crate::Instruction::Store($crate::Register::Indirect($n)) };
    (INC $n:literal) => { $crate::Instruction::Increment($crate::Register::Direct($n)) };
    (INC @$n:literal) => { $crate::Instruction::Increment($crate::Register::Indirect($n)) };
    (DEC $n:literal) => { $crate::Instruction::Decrement($crate::Register::Direct($n)) };
    (DEC @$n:literal) => { $crate::Instruction::Decrement($crate::Register::Indirect($n)) };
    (ADD #$n:literal) => { $crate::Instruction::Add($crate::Value::Constant($n)) };
    (ADD $n:literal) => { $crate::Instruction::Add($crate::Value::Register($crate::Register::Direct($n))) };
    (ADD @$n:literal) => { $crate::Instruction::Add($crate::Value::Register($crate::Register::Indirect($n))) };
    (SUB #$n:literal) => { $crate::Instruction::Sub($crate::Value::Constant($n)) };
    (SUB $n:literal) => { $crate::Instruction::Sub($crate::Value::Register($crate::Register::Direct($n))) };
    (SUB @$n:literal) => { $crate::Instruction::Sub($crate::Value::Register($crate::Register::Indirect($n))) };
    (MUL #$n:literal) => { $crate::Instruction::Mul($crate::Value::Constant($n)) };
    (MUL $n:literal) => { $crate::Instruction::Mul($crate::Value::Register($crate::Register::Direct($n))) };
    (MUL @$n:literal) => { $crate::Instruction::Mul($crate::Value::Register($crate::Register::Indirect($n))) };
    (DIV #$n:literal) => { $crate::Instruction::Div($crate::Value::Constant($n)) };
    (DIV $n:literal) => { $crate::Instruction::Div($crate::Value::Register($crate::Register::Direct($n))) };
    (DIV @$n:literal) => { $crate::Instruction::Div($crate::Value::Register($crate::Register::Indirect($n))) };
    (MOD #$n:literal) => { $crate::Instruction::Mod($crate::Value::Constant($n)) };
    (MOD $n:literal) => { $crate::Instruction::Mod($crate::Value::Register($crate::Register::Direct($n))) };
    (MOD @$n:literal) => { $crate::Instruction::Mod($crate::Value::Register($crate::Register::Indirect($n))) };
    (JUMP $n:literal) => { $crate::Instruction::Jump($crate::Address::Constant($n)) };
    (JUMP @$n:literal) => { $crate::Instruction::Jump($crate::Address::Register($n)) };
    (JUMZ $n:literal) => { $crate::Instruction::JumpZero($crate::Address::Constant($n)) };
    (JUMZ @$n:literal) => { $crate::Instruction::JumpZero($crate::Address::Register($n)) };
    (JUML $n:literal) => { $crate::Instruction::JumpLtz($crate::Address::Constant($n)) };
    (JUML @$n:literal) => { $crate::Instruction::JumpLtz($crate::Address::Register($n)) };
    (JUMG $n:literal) => { $crate::Instruction::JumpGtz($crate::Address::Constant($n)) };
    (JUMG @$n:literal) => { $crate::Instruction::JumpGtz($crate::Address::Register($n)) };
    (STOP) => { $crate::Instruction::Stop };
    (NOP) => { $crate::Instruction::Nop };
}

#[macro_export]
macro_rules! rocode {
    ($T:ty; $($inst:ident)+) => {
        $crate::RoCode::<$T>::from(::std::vec![$($crate::inst!($inst)),+].as_slice())
    };
    
    ($($inst:ident)*) => {
        $crate::rocode!(i32; $($inst)*)
    };
    
    ($T:ty$(;)?) => {
        $crate::RoCode::<$T>::default()
    };
}

#[macro_export]
macro_rules! ram {
    ($T:ty; $($input:literal)+; $($inst:ident)+) => {
        $crate::run::Ram::new($crate::rocode!($T; $($inst)+), [$($input),+])
    };
    
    ($($input:literal)+; $($inst:ident)+) => {
        $crate::run::Ram::new($crate::rocode!(i32; $($inst)+), [$($input),+])
    };
    
    ($T:ty; $($inst:ident)+) => {
        $crate::run::Ram::without_inputs($crate::rocode!($T; $($inst)+))
    };
    
    ($($inst:ident)*) => {
        $crate::ram!(i32; $($inst)*)
    };
}

// ram!(u32; 1 2 3 4;
//  ADD
//  STOP
// )
