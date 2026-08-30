Getting Started with FinLang



This guide shows you how to install FinLang, create your first program, check it, compile it, and run it.



1\. Prerequisites



FinLang is distributed through crates.io and installed using Cargo.



Check that Cargo is available:



cargo --version



2\. Installation



Install FinLang with:



cargo install finlang



After installation, verify that the command is available:



finlang



You should see:



Usage: finlang <check|run|compile|format|test> <file.fin>



FinLang can now be invoked from PowerShell.



3\. Create Your First Program



FinLang programs use the .fin file extension.



Create a file named:



hello.fin



Add the following program:



let add = fn(x: Amount, y: Amount) -> Amount {

&#x20;   x + y

} in add(100, 50)



Save the file.



4\. Check the Program



Before executing a program, check it with:



finlang check hello.fin



A successful check produces output similar to:



OK: hello.fin (finlang-0.1)



5\. Compile the Program



To inspect the compiler representation:



finlang compile hello.fin



This invokes the FinLang compiler and prints its canonical representation.



6\. Run the Program



Execute the program with:



finlang run hello.fin



The example produces:



result=U64(150)



The VM also reports the resulting financial state.



7\. Running Files from Another Directory



The .fin file does not need to be located in the current PowerShell directory.



For example:



finlang run C:\\path\\to\\hello.fin



FinLang reads the specified source file and executes it.



8\. Formatting



FinLang provides a formatting command:



finlang format hello.fin



9\. Policies



FinLang supports executable user-defined policies.



Example:



policy Minimum(x: Amount) { x >= 100 }

in validate(oracleRead(feedA), Minimum)



If feedA evaluates to 150, the policy condition succeeds because:



150 >= 100



Changing the policy to:



policy Minimum(x: Amount) { x >= 200 }

in validate(oracleRead(feedA), Minimum)



causes execution to reject the value:



policy 'Minimum' rejected value 150



Policies therefore participate directly in VM execution.



10\. Testing



The FinLang source repository contains automated tests covering the compiler, bytecode, VM, policies, language features, determinism, and other components.



From the FinLang source repository:



cargo test



11\. Basic Workflow



A typical FinLang workflow is:



Write .fin program

&#x20;      ↓

finlang check program.fin

&#x20;      ↓

finlang compile program.fin

&#x20;      ↓

finlang run program.fin



12\. Next Documentation



Continue with:



\* Language Syntax

\* Types

\* Expressions

\* Functions

\* Financial Operations

\* Policies

\* CLI Reference

\* Verification

\* Architecture



Links



GitHub:



https://github.com/adnanomar77/finlang



FinLang CLI:



https://crates.io/crates/finlang



FinLang Core:



https://crates.io/crates/finlang-core



Zenodo DOI:



https://doi.org/10.5281/zenodo.22181786

