use std::collections::HashMap;
use std::fmt::Write;

use crate::ast::{BinaryOp, Node, Type, UnaryOp};
use crate::error::{codegen_error, Result};

/// Code generator for x86-64 assembly
pub struct CodeGenerator {
    output: String,
    label_count: usize,
    string_literals: Vec<String>,
    variables: HashMap<String, Variable>,
    current_function: Option<String>,
    stack_offset: usize,
}

/// Represents a variable in the generated code
#[derive(Debug, Clone)]
struct Variable {
    offset: usize,
    type_: Type,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            label_count: 0,
            string_literals: Vec::new(),
            variables: HashMap::new(),
            current_function: None,
            stack_offset: 0,
        }
    }

    /// Generate a unique label
    fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!(".{}{}", prefix, self.label_count);
        self.label_count += 1;
        label
    }

    /// Get the size of a type in bytes
    fn size_of(&self, type_: &Type) -> usize {
        match type_ {
            Type::Void => 0,
            Type::Char => 1,
            Type::Int => 4,
            Type::Long => 8,
            Type::Pointer(_) => 8,
            Type::Array(base, Some(size)) => self.size_of(base) * size,
            Type::Array(_, None) => panic!("Cannot determine size of array with unknown size"),
            Type::Function(_, _, _) => 8, // Function pointers are 8 bytes
            Type::Struct(_, members) => {
                let mut size = 0;
                for (_, member_type) in members {
                    size += self.size_of(member_type);
                }
                size
            }
        }
    }

    /// Align the stack offset to the given alignment
    fn align_to(&self, n: usize, align: usize) -> usize {
        (n + align - 1) & !(align - 1)
    }

    /// Generate code for a program
    pub fn generate(&mut self, program: &Node) -> Result<String> {
        self.emit_header();

        match program {
            Node::Program(declarations) => {
                // First pass: collect all function declarations
                for decl in declarations {
                    if let Node::FunctionDecl {
                        name,
                        return_type,
                        params,
                        ..
                    } = decl
                    {
                        // Check if the function is variadic (has ... in parameters)
                        let is_variadic = params.iter().any(|(name, _)| name == "...");

                        // Remove the ... parameter if present
                        let param_types: Vec<Type> = params.iter()
                            .filter(|(name, _)| name != "...")
                            .map(|(_, t)| t.clone())
                            .collect();

                        let func_type = Type::Function(Box::new(return_type.clone()), param_types, is_variadic);
                        self.variables.insert(name.clone(), Variable {
                            offset: 0,
                            type_: func_type,
                        });
                    }
                }

                // Second pass: generate code for all declarations
                for decl in declarations {
                    self.generate_declaration(decl)?;
                }

                // Generate string literals
                if !self.string_literals.is_empty() {
                    writeln!(self.output, "\n.section .rodata").unwrap();
                    for (i, s) in self.string_literals.iter().enumerate() {
                        writeln!(self.output, ".LC{}:", i).unwrap();

                        // Replace newlines with explicit escape sequences
                        let escaped = s.replace("\n", "\\n");

                        // Split the string into multiple .ascii directives to avoid newline issues
                        writeln!(self.output, "    .ascii \"{}\"", escaped).unwrap();
                        writeln!(self.output, "    .byte 0").unwrap(); // Null terminator
                    }
                }

                Ok(self.output.clone())
            }
            _ => Err(codegen_error("Expected program node")),
        }
    }

    /// Emit the assembly header
    fn emit_header(&mut self) {
        writeln!(self.output, "    .intel_syntax noprefix").unwrap();
        writeln!(self.output, "    .text").unwrap();
        writeln!(self.output, "    .globl main").unwrap();

        // Declare external functions
        writeln!(self.output, "    .extern puts").unwrap();
        writeln!(self.output, "    .extern printf").unwrap();
        writeln!(self.output, "    .extern scanf").unwrap();
        writeln!(self.output, "    .extern putchar").unwrap();
        writeln!(self.output, "    .extern getchar").unwrap();
        writeln!(self.output, "    .extern atoi").unwrap();
    }

    /// Generate code for a declaration
    fn generate_declaration(&mut self, node: &Node) -> Result<()> {
        match node {
            Node::VarDecl {
                name,
                type_,
                initializer: _,
                ..
            } => {
                // Global variable
                writeln!(self.output, "    .data").unwrap();
                writeln!(self.output, "    .globl {}", name).unwrap();
                writeln!(self.output, "{}:", name).unwrap();

                match type_ {
                    Type::Char => {
                        writeln!(self.output, "    .byte 0").unwrap();
                    }
                    Type::Int => {
                        writeln!(self.output, "    .long 0").unwrap();
                    }
                    Type::Long => {
                        writeln!(self.output, "    .quad 0").unwrap();
                    }
                    Type::Array(base, Some(size)) => {
                        let elem_size = self.size_of(base);
                        writeln!(self.output, "    .zero {}", elem_size * size).unwrap();
                    }
                    _ => {
                        return Err(codegen_error(format!(
                            "Unsupported global variable type: {:?}",
                            type_
                        )));
                    }
                }

                writeln!(self.output, "    .text").unwrap();

                Ok(())
            }
            Node::FunctionDecl {
                name,
                params,
                body,
                ..
            } => {
                if let Some(body) = body {
                    self.current_function = Some(name.clone());
                    self.variables.clear();
                    self.stack_offset = 0;

                    // Function prologue
                    writeln!(self.output, "{}:", name).unwrap();
                    writeln!(self.output, "    push rbp").unwrap();
                    writeln!(self.output, "    mov rbp, rsp").unwrap();

                    // Allocate space for parameters
                    // Windows x64 calling convention uses rcx, rdx, r8, r9 for the first 4 parameters
                    let param_registers = ["rcx", "rdx", "r8", "r9"];
                    for (i, (param_name, param_type)) in params.iter().enumerate() {
                        self.stack_offset += 8; // All parameters take 8 bytes on the stack
                        self.variables.insert(
                            param_name.clone(),
                            Variable {
                                offset: self.stack_offset,
                                type_: param_type.clone(),
                            },
                        );

                        if i < param_registers.len() {
                            // Parameter is passed in a register
                            writeln!(self.output, "    push {}", param_registers[i]).unwrap();
                        } else {
                            // Parameter is passed on the stack
                            // TODO: Implement stack parameters
                            return Err(codegen_error("Stack parameters not implemented yet"));
                        }
                    }

                    // Generate code for the function body
                    self.generate_node(body)?;

                    // Function epilogue
                    writeln!(self.output, ".{}ret:", name).unwrap();
                    writeln!(self.output, "    mov rsp, rbp").unwrap();
                    writeln!(self.output, "    pop rbp").unwrap();
                    writeln!(self.output, "    ret").unwrap();

                    self.current_function = None;
                }

                Ok(())
            }
            _ => Err(codegen_error("Expected declaration")),
        }
    }

    /// Generate x86-64 assembly code for an AST node
    ///
    /// This is the core code generation function that recursively traverses the AST
    /// and emits appropriate assembly instructions for each node type. The function
    /// handles various node types including:
    /// - Literals (integer, character, string)
    /// - Variables and identifiers
    /// - Binary and unary expressions
    /// - Function calls
    /// - Control flow statements (if, while, for)
    /// - Block statements and variable declarations
    ///
    /// The generated code follows the Windows x64 calling convention and uses
    /// the RAX register to store the result of expressions.
    ///
    /// # Arguments
    /// * `node` - Reference to the AST node to generate code for
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if code generation fails
    fn generate_node(&mut self, node: &Node) -> Result<()> {
        match node {
            Node::IntLiteral(value, _) => {
                // Load the integer literal value directly into RAX register
                // This makes the value available for subsequent operations
                writeln!(self.output, "    mov rax, {}", value).unwrap();
                Ok(())
            }
            Node::CharLiteral(value, _) => {
                // Convert character to its ASCII/UTF-8 value and load into RAX
                // Characters are treated as 8-bit values but stored in 64-bit register
                writeln!(self.output, "    mov rax, {}", *value as u8).unwrap();
                Ok(())
            }
            Node::StringLiteral(value, _) => {
                // Store string in the .rodata section and get its index
                let index = self.string_literals.len();
                self.string_literals.push(value.clone());

                // Load the effective address (pointer) to the string into RAX
                // Uses RIP-relative addressing which is position-independent
                // .LC{index} is the label that will be defined in the .rodata section
                writeln!(self.output, "    lea rax, [rip + .LC{}]", index).unwrap();
                Ok(())
            }
            Node::Identifier(name, _location) => {
                if let Some(var) = self.variables.get(name) {
                    match var.type_ {
                        Type::Char | Type::Int | Type::Long => {
                            // For scalar types, load the value from the stack into RAX
                            // rbp is the base pointer, and var.offset is the variable's position on the stack
                            writeln!(self.output, "    mov rax, [rbp-{}]", var.offset).unwrap();
                        }
                        Type::Pointer(_) => {
                            // For pointers, load the value of the pointer (which is an address)
                            writeln!(self.output, "    mov rax, [rbp-{}]", var.offset).unwrap();
                        }
                        Type::Array(_, _) => {
                            // For arrays, load the address of the array
                            // lea (Load Effective Address) calculates the address without dereferencing
                            writeln!(self.output, "    lea rax, [rbp-{}]", var.offset).unwrap();
                        }
                        _ => {
                            return Err(codegen_error(format!(
                                "Unsupported variable type: {:?}",
                                var.type_
                            )));
                        }
                    }
                    Ok(())
                } else {
                    // For global variables, load the value from the global memory location
                    // The name directly references a label in the data section
                    writeln!(self.output, "    mov rax, [{}]", name).unwrap();
                    Ok(())
                }
            }
            Node::BinaryExpr {
                op,
                left,
                right,
                location: _,
            } => {
                match op {
                    BinaryOp::Assign => {
                        // Assignment operator requires special handling
                        match &**left {
                            Node::Identifier(name, _) => {
                                // First, evaluate the right-hand side expression
                                // This will put the result in RAX
                                self.generate_node(right)?;

                                // Then store the value from RAX into the variable's memory location
                                if let Some(var) = self.variables.get(name) {
                                    // For local variables, store at [rbp-offset]
                                    // This writes the 64-bit RAX value to the stack location of the variable
                                    writeln!(self.output, "    mov [rbp-{}], rax", var.offset).unwrap();
                                } else {
                                    // For global variables, store at the global label
                                    writeln!(self.output, "    mov [{}], rax", name).unwrap();
                                }
                            },
                            Node::UnaryExpr { op: UnaryOp::Dereference, expr, .. } => {
                                // For pointer dereference (*p = value), we need to:
                                // 1. Evaluate the right-hand side and save it
                                self.generate_node(right)?;
                                writeln!(self.output, "    push rax").unwrap();  // Save the value to assign

                                // 2. Evaluate the pointer expression to get the address
                                self.generate_node(expr)?;
                                // Now RAX contains the address to store to

                                // 3. Pop the value and store it at the address
                                writeln!(self.output, "    pop rcx").unwrap();  // Get the value to assign
                                writeln!(self.output, "    mov [rax], rcx").unwrap();  // Store the value at the address
                            },
                            _ => {
                                return Err(codegen_error("Left operand of assignment must be an identifier or dereferenced pointer"));
                            }
                        }
                    }
                    _ => {
                        // For all other binary operations, we need both operands' values

                        // First, evaluate the left operand and save its value on the stack
                        // This frees up RAX for evaluating the right operand
                        self.generate_node(left)?;
                        writeln!(self.output, "    push rax").unwrap();  // Save left operand value

                        // Then, evaluate the right operand (result will be in RAX)
                        self.generate_node(right)?;

                        // Pop the left operand value into RCX
                        // Now: left value in RCX, right value in RAX
                        writeln!(self.output, "    pop rcx").unwrap();

                        // Generate the specific operation based on the operator type
                        match op {
                            BinaryOp::Add => {
                                // Addition: RAX = RCX + RAX
                                // Adds the value in RCX (left operand) to RAX (right operand)
                                writeln!(self.output, "    add rax, rcx").unwrap();
                            }
                            BinaryOp::Subtract => {
                                // Subtraction: RAX = RCX - RAX
                                // Note the order: left operand (RCX) - right operand (RAX)
                                writeln!(self.output, "    sub rcx, rax").unwrap();
                                writeln!(self.output, "    mov rax, rcx").unwrap();  // Move result to RAX
                            }
                            BinaryOp::Multiply => {
                                // Signed multiplication: RAX = RAX * RCX
                                // imul performs signed integer multiplication
                                writeln!(self.output, "    imul rax, rcx").unwrap();
                            }
                            BinaryOp::Divide => {
                                // Division: RAX = RCX / RAX
                                // x86 div instruction requires special setup:
                                writeln!(self.output, "    mov rdx, 0").unwrap();    // Clear RDX (will hold remainder)
                                writeln!(self.output, "    mov rax, rcx").unwrap();  // Move dividend to RAX
                                writeln!(self.output, "    div rax").unwrap();       // Divide RAX by original RAX value
                                // Result is stored in RAX (quotient) and RDX (remainder)
                            }
                            BinaryOp::Modulo => {
                                // Modulo: RAX = RCX % RAX
                                // Uses the same div instruction as division but returns the remainder
                                writeln!(self.output, "    mov rdx, 0").unwrap();    // Clear RDX
                                writeln!(self.output, "    mov rax, rcx").unwrap();  // Move dividend to RAX
                                writeln!(self.output, "    div rax").unwrap();       // Divide RAX by original RAX value
                                writeln!(self.output, "    mov rax, rdx").unwrap();  // Move remainder from RDX to RAX
                            }
                            BinaryOp::Equal => {
                                // Equality comparison: RAX = (RCX == RAX) ? 1 : 0
                                writeln!(self.output, "    cmp rcx, rax").unwrap();   // Compare left and right operands
                                writeln!(self.output, "    sete al").unwrap();       // Set AL to 1 if equal, 0 if not
                                writeln!(self.output, "    movzx rax, al").unwrap(); // Zero-extend AL to RAX (clears upper bits)
                            }
                            BinaryOp::NotEqual => {
                                // Inequality comparison: RAX = (RCX != RAX) ? 1 : 0
                                writeln!(self.output, "    cmp rcx, rax").unwrap();   // Compare left and right operands
                                writeln!(self.output, "    setne al").unwrap();      // Set AL to 1 if not equal, 0 if equal
                                writeln!(self.output, "    movzx rax, al").unwrap(); // Zero-extend AL to RAX
                            }
                            BinaryOp::Less => {
                                // Less than comparison: RAX = (RCX < RAX) ? 1 : 0
                                writeln!(self.output, "    cmp rcx, rax").unwrap();   // Compare left and right operands
                                writeln!(self.output, "    setl al").unwrap();       // Set AL to 1 if less, 0 if not
                                writeln!(self.output, "    movzx rax, al").unwrap(); // Zero-extend AL to RAX
                            }
                            BinaryOp::LessEqual => {
                                // Less than or equal comparison: RAX = (RCX <= RAX) ? 1 : 0
                                writeln!(self.output, "    cmp rcx, rax").unwrap();   // Compare left and right operands
                                writeln!(self.output, "    setle al").unwrap();      // Set AL to 1 if less or equal, 0 if not
                                writeln!(self.output, "    movzx rax, al").unwrap(); // Zero-extend AL to RAX
                            }
                            BinaryOp::Greater => {
                                // Greater than comparison: RAX = (RCX > RAX) ? 1 : 0
                                writeln!(self.output, "    cmp rcx, rax").unwrap();   // Compare left and right operands
                                writeln!(self.output, "    setg al").unwrap();       // Set AL to 1 if greater, 0 if not
                                writeln!(self.output, "    movzx rax, al").unwrap(); // Zero-extend AL to RAX
                            }
                            BinaryOp::GreaterEqual => {
                                // Greater than or equal comparison: RAX = (RCX >= RAX) ? 1 : 0
                                writeln!(self.output, "    cmp rcx, rax").unwrap();   // Compare left and right operands
                                writeln!(self.output, "    setge al").unwrap();      // Set AL to 1 if greater or equal, 0 if not
                                writeln!(self.output, "    movzx rax, al").unwrap(); // Zero-extend AL to RAX
                            }
                            BinaryOp::LogicalAnd => {
                                // Logical AND with short-circuit evaluation
                                // If left operand is false (0), result is false without evaluating right operand
                                let end_label = self.generate_label("land");

                                // Check if left operand (RCX) is false (0)
                                writeln!(self.output, "    cmp rcx, 0").unwrap();
                                // If left is false, jump to end and result will be 0
                                writeln!(self.output, "    je {}", end_label).unwrap();

                                // Left is true, so result depends on right operand (RAX)
                                writeln!(self.output, "    cmp rax, 0").unwrap();
                                // Set AL to 1 if right is non-zero (true)
                                writeln!(self.output, "    setne al").unwrap();
                                // Zero-extend AL to RAX for the final result
                                writeln!(self.output, "    movzx rax, al").unwrap();

                                // Target for the short-circuit jump
                                writeln!(self.output, "{}:", end_label).unwrap();
                            }
                            BinaryOp::LogicalOr => {
                                // Logical OR with short-circuit evaluation
                                // If left operand is true (non-0), result is true without evaluating right operand
                                let end_label = self.generate_label("lor");

                                // Check if left operand (RCX) is true (non-0)
                                writeln!(self.output, "    cmp rcx, 0").unwrap();
                                // If left is true, jump to end and result will be 1
                                writeln!(self.output, "    jne {}", end_label).unwrap();

                                // Left is false, so result depends on right operand (RAX)
                                writeln!(self.output, "    cmp rax, 0").unwrap();
                                // Set AL to 1 if right is non-zero (true)
                                writeln!(self.output, "    setne al").unwrap();
                                // Zero-extend AL to RAX for the final result
                                writeln!(self.output, "    movzx rax, al").unwrap();

                                // Target for the short-circuit jump
                                writeln!(self.output, "{}:", end_label).unwrap();
                            }
                            BinaryOp::BitwiseAnd => {
                                // Bitwise AND: RAX = RAX & RCX
                                // Performs bitwise AND between left and right operands
                                writeln!(self.output, "    and rax, rcx").unwrap();
                            }
                            BinaryOp::BitwiseOr => {
                                // Bitwise OR: RAX = RAX | RCX
                                // Performs bitwise OR between left and right operands
                                writeln!(self.output, "    or rax, rcx").unwrap();
                            }
                            BinaryOp::BitwiseXor => {
                                // Bitwise XOR: RAX = RAX ^ RCX
                                // Performs bitwise exclusive OR between left and right operands
                                writeln!(self.output, "    xor rax, rcx").unwrap();
                            }
                            BinaryOp::ShiftLeft => {
                                // Shift left: RAX = RCX << (RAX & 0x3F)
                                // x86 shift instructions use CL (lowest byte of RCX) for shift count
                                writeln!(self.output, "    mov rcx, rax").unwrap();  // Move shift count to RCX
                                writeln!(self.output, "    mov rax, rcx").unwrap();  // Move value to shift into RAX
                                writeln!(self.output, "    shl rax, cl").unwrap();   // Shift RAX left by CL bits
                            }
                            BinaryOp::ShiftRight => {
                                // Shift right: RAX = RCX >> (RAX & 0x3F)
                                // Logical shift right (zeros are shifted in from the left)
                                writeln!(self.output, "    mov rcx, rax").unwrap();  // Move shift count to RCX
                                writeln!(self.output, "    mov rax, rcx").unwrap();  // Move value to shift into RAX
                                writeln!(self.output, "    shr rax, cl").unwrap();   // Shift RAX right by CL bits
                            }
                            BinaryOp::Assign => unreachable!(),
                        }
                    }
                }

                Ok(())
            }
            Node::UnaryExpr {
                op,
                expr,
                location: _,
            } => {
                // Special case for address-of operator
                if let UnaryOp::AddressOf = op {
                    // Address-of operator: RAX = &expr
                    // Special case: we need the address, not the value
                    if let Node::Identifier(name, _) = &**expr {
                        if let Some(var) = self.variables.get(name) {
                            // For local variables, calculate address relative to RBP
                            // lea (Load Effective Address) calculates the address without dereferencing
                            // We don't need to load the value first, just the address
                            writeln!(self.output, "    lea rax, [rbp-{}]", var.offset).unwrap();

                        } else {
                            // For global variables, get the address of the global label
                            writeln!(self.output, "    lea rax, [{}]", name).unwrap();
                        }
                    } else {
                        return Err(codegen_error("Cannot take address of non-lvalue"));
                    }
                    return Ok(());
                }

                // For other unary operators, first evaluate the expression to get its value in RAX
                self.generate_node(expr)?;

                match op {
                    UnaryOp::Negate => {
                        // Arithmetic negation: RAX = -RAX
                        // Negates the value in RAX (two's complement)
                        writeln!(self.output, "    neg rax").unwrap();
                    }
                    UnaryOp::LogicalNot => {
                        // Logical NOT: RAX = !RAX (0 becomes 1, non-0 becomes 0)
                        writeln!(self.output, "    cmp rax, 0").unwrap();      // Compare RAX with 0
                        writeln!(self.output, "    sete al").unwrap();        // Set AL to 1 if RAX is 0, otherwise 0
                        writeln!(self.output, "    movzx rax, al").unwrap();  // Zero-extend AL to RAX
                    }
                    UnaryOp::BitwiseNot => {
                        // Bitwise NOT: RAX = ~RAX (flips all bits)
                        writeln!(self.output, "    not rax").unwrap();
                    }
                    UnaryOp::Dereference => {
                        // Dereference: RAX = *RAX (load value from address in RAX)
                        // Treats RAX as a pointer and loads the value it points to
                        // For pointers to integers, we need to load the value at the address
                        writeln!(self.output, "    mov rax, [rax]").unwrap();
                    }
                    UnaryOp::AddressOf => {
                        // This case is handled separately above
                        unreachable!("AddressOf should be handled before match");
                    }
                }

                Ok(())
            }
            Node::FunctionCall {
                name,
                args,
                location: _,
            } => {
                // Function call using Windows x64 calling convention

                // Save all volatile registers that might be modified by the callee
                // This preserves their values across the function call
                writeln!(self.output, "    push rbx").unwrap();  // Non-volatile register
                writeln!(self.output, "    push rsi").unwrap();  // Non-volatile register
                writeln!(self.output, "    push rdi").unwrap();  // Non-volatile register
                writeln!(self.output, "    push rcx").unwrap();  // Volatile register (1st param)
                writeln!(self.output, "    push rdx").unwrap();  // Volatile register (2nd param)
                writeln!(self.output, "    push r8").unwrap();   // Volatile register (3rd param)
                writeln!(self.output, "    push r9").unwrap();   // Volatile register (4th param)
                writeln!(self.output, "    push r10").unwrap();  // Volatile register
                writeln!(self.output, "    push r11").unwrap();  // Volatile register

                // Prepare arguments according to Windows x64 calling convention
                // First 4 args go in registers: RCX, RDX, R8, R9
                // Additional args are pushed on the stack in reverse order
                let arg_registers = ["rcx", "rdx", "r8", "r9"];
                for (i, arg) in args.iter().enumerate() {
                    // Evaluate the argument expression (result in RAX)
                    self.generate_node(arg)?;

                    if i < arg_registers.len() {
                        // For the first 4 arguments, move from RAX to the appropriate register
                        writeln!(self.output, "    mov {}, rax", arg_registers[i]).unwrap();
                    } else {
                        // For additional arguments, push onto the stack
                        writeln!(self.output, "    push rax").unwrap();
                    }
                }

                // Call the function by name
                // This will jump to the function and save the return address
                writeln!(self.output, "    call {}", name).unwrap();

                // Clean up stack space used for arguments beyond the first 4
                // Each argument takes 8 bytes on the stack
                if args.len() > arg_registers.len() {
                    let stack_args = args.len() - arg_registers.len();
                    writeln!(self.output, "    add rsp, {}", stack_args * 8).unwrap();
                }

                // Restore all saved registers in reverse order
                // This ensures the register state is the same as before the call
                writeln!(self.output, "    pop r11").unwrap();
                writeln!(self.output, "    pop r10").unwrap();
                writeln!(self.output, "    pop r9").unwrap();
                writeln!(self.output, "    pop r8").unwrap();
                writeln!(self.output, "    pop rdx").unwrap();
                writeln!(self.output, "    pop rcx").unwrap();
                writeln!(self.output, "    pop rdi").unwrap();
                writeln!(self.output, "    pop rsi").unwrap();
                writeln!(self.output, "    pop rbx").unwrap();

                // Function return value is already in RAX per calling convention
                Ok(())
            }
            Node::ExpressionStmt(expr) => {
                // Expression statement - evaluate the expression but discard the result
                // The value is left in RAX but not used by the caller
                self.generate_node(expr)?;
                Ok(())
            }
            Node::ReturnStmt(value, _) => {
                // Return statement - evaluate the expression and jump to function epilogue

                // If there's a return value, evaluate it (result will be in RAX)
                if let Some(expr) = value {
                    self.generate_node(expr)?;
                    // The result is already in RAX, which is the return value register
                }

                if let Some(func_name) = &self.current_function {
                    // Jump to the function's epilogue (return label)
                    // This skips any remaining code in the function
                    writeln!(self.output, "    jmp .{}ret", func_name).unwrap();
                } else {
                    return Err(codegen_error("Return statement outside of function"));
                }

                Ok(())
            }
            Node::IfStmt {
                condition,
                then_branch,
                else_branch,
                location: _,
            } => {
                // If-else statement with conditional branching
                // Create unique labels for the else branch and end of if statement
                let else_label = self.generate_label("else");
                let end_label = self.generate_label("endif");

                // Generate code for the condition expression
                // Result will be in RAX
                self.generate_node(condition)?;

                // Compare the condition result with 0 (false)
                writeln!(self.output, "    cmp rax, 0").unwrap();
                // If condition is false (0), jump to else branch
                writeln!(self.output, "    je {}", else_label).unwrap();

                // Generate code for the 'then' branch (executed if condition is true)
                self.generate_node(then_branch)?;
                // After executing 'then' branch, skip the 'else' branch
                writeln!(self.output, "    jmp {}", end_label).unwrap();

                // Else branch starts here
                writeln!(self.output, "{}:", else_label).unwrap();
                // Generate code for the 'else' branch if it exists
                if let Some(else_branch) = else_branch {
                    self.generate_node(else_branch)?;
                }

                // End of the if-else statement
                writeln!(self.output, "{}:", end_label).unwrap();

                Ok(())
            }
            Node::WhileStmt {
                condition,
                body,
                location: _,
            } => {
                // While loop implementation
                // Create unique labels for the loop start and end
                let start_label = self.generate_label("while");
                let end_label = self.generate_label("endwhile");

                // Loop start label - the condition is checked at the beginning of each iteration
                writeln!(self.output, "{}:", start_label).unwrap();

                // Generate code for the condition expression
                self.generate_node(condition)?;
                // Compare condition result with 0 (false)
                writeln!(self.output, "    cmp rax, 0").unwrap();
                // If condition is false, exit the loop
                writeln!(self.output, "    je {}", end_label).unwrap();

                // Generate code for the loop body
                self.generate_node(body)?;
                // After executing the body, jump back to check the condition again
                writeln!(self.output, "    jmp {}", start_label).unwrap();

                // Loop end label - execution continues here when the condition becomes false
                writeln!(self.output, "{}:", end_label).unwrap();

                Ok(())
            }
            Node::ForStmt {
                init,
                condition,
                increment,
                body,
                location: _,
            } => {
                // For loop implementation with three components: init, condition, increment
                // Create unique labels for loop start, end, and increment section
                let start_label = self.generate_label("for");      // Start of loop (condition check)
                let end_label = self.generate_label("endfor");    // End of loop
                let inc_label = self.generate_label("forinc");    // Increment section

                // 1. Initialization - executed once before the loop starts
                if let Some(init) = init {
                    self.generate_node(init)?;
                }

                // Loop start label - condition is checked at the beginning of each iteration
                writeln!(self.output, "{}:", start_label).unwrap();

                // 2. Condition check - if false, exit the loop
                // If no condition is provided, the loop runs indefinitely (until break)
                if let Some(condition) = condition {
                    self.generate_node(condition)?;
                    writeln!(self.output, "    cmp rax, 0").unwrap();  // Compare with false (0)
                    writeln!(self.output, "    je {}", end_label).unwrap();  // Exit if condition is false
                }

                // 3. Loop body - the main code to execute in each iteration
                self.generate_node(body)?;

                // 4. Increment section - executed after each iteration
                writeln!(self.output, "{}:", inc_label).unwrap();
                if let Some(increment) = increment {
                    self.generate_node(increment)?;
                }

                // Jump back to the condition check for the next iteration
                writeln!(self.output, "    jmp {}", start_label).unwrap();

                // Loop end label - execution continues here when the loop exits
                writeln!(self.output, "{}:", end_label).unwrap();

                Ok(())
            }
            Node::BlockStmt(statements, _) => {
                // Block statement - a sequence of statements executed in order
                // No special assembly setup is needed for blocks - just generate code for each statement
                for stmt in statements {
                    self.generate_node(stmt)?;
                }

                Ok(())
            }
            Node::VarDecl {
                name,
                type_,
                initializer,
                location: _,
            } => {
                // Local variable declaration with optional initialization

                // Calculate the size of the variable based on its type
                let size = self.size_of(type_);

                // Determine the alignment requirement for the variable type
                let align = match type_ {
                    Type::Char => 1,                // 1-byte alignment for char
                    Type::Int => 4,                // 4-byte alignment for int
                    Type::Long => 8,               // 8-byte alignment for long
                    Type::Pointer(_) => 8,         // 8-byte alignment for pointers
                    Type::Array(_, _) => 8,        // 8-byte alignment for arrays
                    _ => 8,                        // Default to 8-byte alignment
                };

                // Adjust the stack offset to maintain proper alignment
                // This ensures all variables are properly aligned in memory
                self.stack_offset = self.align_to(self.stack_offset + size, align);

                // Register the variable in our symbol table with its stack offset
                self.variables.insert(
                    name.clone(),
                    Variable {
                        offset: self.stack_offset,  // Distance from base pointer
                        type_: type_.clone(),      // Type information for later use
                    },
                );

                // Allocate space on the stack for the variable
                // This decreases RSP to make room for the variable
                writeln!(self.output, "    sub rsp, {}", size).unwrap();

                // If there's an initializer, evaluate it and store the result
                if let Some(init) = initializer {
                    // Evaluate the initializer expression (result in RAX)
                    self.generate_node(init)?;

                    // Store the value from RAX into the variable's stack location
                    // For pointers, we need to store the address
                    writeln!(self.output, "    mov [rbp-{}], rax", self.stack_offset).unwrap();
                }

                Ok(())
            }
            Node::FunctionDecl { .. } => {
                // Function declarations are handled separately in generate_declaration
                // This case should only be reached for nested function declarations,
                // which are not supported in C
                Ok(())
            }
            Node::Program(_) => {
                // Program nodes should only appear at the top level and are handled by generate()
                // If we encounter a nested program node, it's an error in the AST
                panic!("Nested program node");
            }
        }
    }
}
