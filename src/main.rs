use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::env;

mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;
mod preprocessor;
mod typechecker;

use crate::codegen::CodeGenerator;
use crate::error::Result;
use crate::lexer::Lexer;
use crate::parser::Parser as CParser;
use crate::preprocessor::Preprocessor;
use crate::typechecker::TypeChecker;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <input.c> [output]", args[0]);
        return Ok(());
    }

    let input = PathBuf::from(&args[1]);
    let output = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        let stem = input.file_stem().unwrap().to_string_lossy().to_string();
        PathBuf::from(stem)
    };

    println!("Compiling {} to {}", input.display(), output.display());

    // Read input file
    let source = fs::read_to_string(&input).map_err(|e| {
        error::CompilerError::IoError(e)
    })?;

    // Tokenize
    let mut lexer = Lexer::new(&source, input.to_string_lossy().to_string());
    let tokens = lexer.tokenize()?;

    println!("Tokenization complete: {} tokens", tokens.len());

    // Print tokens for debugging
    for token in &tokens {
        println!("Token: {:?} at {}:{}", token.kind, token.location.line, token.location.column);
    }

    // Preprocess
    let mut preprocessor = Preprocessor::new();

    // Add include paths
    preprocessor.add_include_path("include");

    let preprocessed_tokens = preprocessor.preprocess(tokens)?;

    println!("Preprocessing complete: {} tokens", preprocessed_tokens.len());

    // Parse
    let mut parser = CParser::new(&preprocessed_tokens);
    let ast = parser.parse_program()?;

    println!("Parsing complete");

    // Type check
    let mut typechecker = TypeChecker::new();
    typechecker.check_program(&ast)?;

    println!("Type checking complete");

    // Generate code
    let mut codegen = CodeGenerator::new();
    let assembly = codegen.generate(&ast)?;

    println!("Code generation complete");

    // Create output directories if they don't exist
    let asm_dir = PathBuf::from("output/asm");
    let bin_dir = PathBuf::from("output/bin");

    fs::create_dir_all(&asm_dir).map_err(|e| {
        error::CompilerError::IoError(e)
    })?;
    fs::create_dir_all(&bin_dir).map_err(|e| {
        error::CompilerError::IoError(e)
    })?;

    // Write assembly to file in the asm directory
    let asm_file = asm_dir.join(format!("{}.s", output.to_string_lossy()));

    fs::write(&asm_file, assembly).map_err(|e| {
        error::CompilerError::IoError(e)
    })?;

    // Assemble and link
    println!("Assembling and linking");

    // Set the output executable path to be in the bin directory
    let exe_file = bin_dir.join(format!("{}.exe", output.to_string_lossy()));

    let status = Command::new("gcc")
        .arg("-o")
        .arg(&exe_file)
        .arg(&asm_file)
        .status()
        .map_err(|e| {
            error::CompilerError::IoError(e)
        })?;

    if !status.success() {
        return Err(error::CompilerError::CodeGenError {
            message: "Assembly or linking failed".to_string(),
        });
    }

    println!("Compilation successful:");
    println!("  Assembly: {}", asm_file.display());
    println!("  Executable: {}", exe_file.display());

    Ok(())
}