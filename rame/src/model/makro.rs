
#[macro_export]
macro_rules! inst {
    (READ) => { $crate::model::Instruction::Read };
    (WRITE) => { $crate::model::Instruction::Write };
    (LOAD #$n:literal) => { $crate::model::Instruction::Load($crate::model::Value::Constant($n)) };
    (LOAD $n:literal) => { $crate::model::Instruction::Load($crate::model::Value::Register($crate::model::Register::Direct($n))) };
    (LOAD @$n:literal) => { $crate::model::Instruction::Load($crate::model::Value::Register($crate::model::Register::Indirect($n))) };
    (STORE $n:literal) => { $crate::model::Instruction::Store($crate::model::Register::Direct($n)) };
    (STORE @$n:literal) => { $crate::model::Instruction::Store($crate::model::Register::Indirect($n)) };
    (INC $n:literal) => { $crate::model::Instruction::Increment($crate::model::Register::Direct($n)) };
    (INC @$n:literal) => { $crate::model::Instruction::Increment($crate::model::Register::Indirect($n)) };
    (DEC $n:literal) => { $crate::model::Instruction::Decrement($crate::model::Register::Direct($n)) };
    (DEC @$n:literal) => { $crate::model::Instruction::Decrement($crate::model::Register::Indirect($n)) };
    (ADD #$n:literal) => { $crate::model::Instruction::Add($crate::model::Value::Constant($n)) };
    (ADD $n:literal) => { $crate::model::Instruction::Add($crate::model::Value::Register($crate::model::Register::Direct($n))) };
    (ADD @$n:literal) => { $crate::model::Instruction::Add($crate::model::Value::Register($crate::model::Register::Indirect($n))) };
    (SUB #$n:literal) => { $crate::model::Instruction::Sub($crate::model::Value::Constant($n)) };
    (SUB $n:literal) => { $crate::model::Instruction::Sub($crate::model::Value::Register($crate::model::Register::Direct($n))) };
    (SUB @$n:literal) => { $crate::model::Instruction::Sub($crate::model::Value::Register($crate::model::Register::Indirect($n))) };
    (MUL #$n:literal) => { $crate::model::Instruction::Mul($crate::model::Value::Constant($n)) };
    (MUL $n:literal) => { $crate::model::Instruction::Mul($crate::model::Value::Register($crate::model::Register::Direct($n))) };
    (MUL @$n:literal) => { $crate::model::Instruction::Mul($crate::model::Value::Register($crate::model::Register::Indirect($n))) };
    (DIV #$n:literal) => { $crate::model::Instruction::Div($crate::model::Value::Constant($n)) };
    (DIV $n:literal) => { $crate::model::Instruction::Div($crate::model::Value::Register($crate::model::Register::Direct($n))) };
    (DIV @$n:literal) => { $crate::model::Instruction::Div($crate::model::Value::Register($crate::model::Register::Indirect($n))) };
    (MOD #$n:literal) => { $crate::model::Instruction::Mod($crate::model::Value::Constant($n)) };
    (MOD $n:literal) => { $crate::model::Instruction::Mod($crate::model::Value::Register($crate::model::Register::Direct($n))) };
    (MOD @$n:literal) => { $crate::model::Instruction::Mod($crate::model::Value::Register($crate::model::Register::Indirect($n))) };
    (JUMP $n:literal) => { $crate::model::Instruction::Jump($crate::model::Address::Constant($n)) };
    (JUMP @$n:literal) => { $crate::model::Instruction::Jump($crate::model::Address::Register($n)) };
    (JUMZ $n:literal) => { $crate::model::Instruction::JumpZero($crate::model::Address::Constant($n)) };
    (JUMZ @$n:literal) => { $crate::model::Instruction::JumpZero($crate::model::Address::Register($n)) };
    (JUML $n:literal) => { $crate::model::Instruction::JumpLtz($crate::model::Address::Constant($n)) };
    (JUML @$n:literal) => { $crate::model::Instruction::JumpLtz($crate::model::Address::Register($n)) };
    (JUMG $n:literal) => { $crate::model::Instruction::JumpGtz($crate::model::Address::Constant($n)) };
    (JUMG @$n:literal) => { $crate::model::Instruction::JumpGtz($crate::model::Address::Register($n)) };
    (STOP) => { $crate::model::Instruction::Stop };
    (NOP) => { $crate::model::Instruction::Nop };
}

#[macro_export]
macro_rules! rocode {
    ($T:ty; $($inst:ident)+) => {
        $crate::model::RoCode::<$T>::from(::std::vec![$($crate::inst!($inst)),+].as_slice())
    };
    
    ($($inst:ident)*) => {
        $crate::rocode!(i32; $($inst)*)
    };
    
    ($T:ty$(;)?) => {
        $crate::model::RoCode::<$T>::default()
    };
}

#[macro_export]
macro_rules! ram {
    ($T:ty; $($input:literal)+; $($inst:ident)+) => {
        $crate::runner::Ram::new($crate::rocode!($T; $($inst)+), [$($input),+])
    };
    
    ($($input:literal)+; $($inst:ident)+) => {
        $crate::runner::Ram::new($crate::rocode!(i32; $($inst)+), [$($input),+])
    };
    
    ($T:ty; $($inst:ident)+) => {
        $crate::runner::Ram::without_inputs($crate::rocode!($T; $($inst)+))
    };
    
    ($($inst:ident)*) => {
        $crate::ram!(i32; $($inst)*)
    };
}
