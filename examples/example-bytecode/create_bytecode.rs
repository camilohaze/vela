/*!
Example Bytecode Program

This creates a simple bytecode file that calculates 6 * 7 = 42
*/

use vela_vm::{Bytecode, CodeObject, Constant, Instruction};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytecode = Bytecode::new();
    
    // Add constants
    bytecode.add_constant(Constant::Int(6));
    bytecode.add_constant(Constant::Int(7));
    
    // Create main code object
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    // Generate bytecode: LOAD_CONST 0, LOAD_CONST 1, MUL, RETURN
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::Mul);
    bytecode.push(Instruction::Return);
    
    // Serialize to file
    let bytes = bytecode.to_bytes()?;
    std::fs::write("example.velac", bytes)?;
    
    println!("✓ Created example.velac (6 * 7 = 42)");
    println!("Run with: vela run example.velac");
    Ok(())
}
