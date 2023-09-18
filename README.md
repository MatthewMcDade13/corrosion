# Corrosion
Corrosion (.corr) Programming Language

#### Goals
- Syntax is somewhere between Javascript and Elixir. 
- Native embedable Rust bindings.
- Dynamically Typed
- Module System
- documentation generation and typespecs similar to Elixirs @moduledoc and @spec, respectively
- Macros
- Flux :: package manager & cli tool for Corrosion (simliar to npm/cargo, name tentative)
- Typed Structs (similar to Elixir defstruct or Javascript class)
- Traits/Behaviors or Interfaces (deicion tentative)
- Pattern matching / Destructuring
- Optional Types (nil exists but I may remove it in favor of optional types, nonetheless Optional types will be a feature no matter what)
- Pipe operator
- Pass instance to method function automatically (instance.method() and method(instance) are both legal syntax and express the same method/function call)


#### Open Considerations
- No Nil and Optional Types? or both?
- do Structs or Modules implement Traits/Behaviors? (javascript vs Elixir implementation of interface/behavior respectively)

#### Declarations
- var :: similar to Rust's 'let mut', allows reassignment, but not rebinding
- let :: similar to Rust's let, allows rebinding, but not reassignment
- const :: Does not allow rebinding, or reassignment, unique identifier in current scope. 
    I may also enfore the binding it points to to be immutable, but for now its not and will work similarly to Javascript's 'const'

#### Compile Targets
- ECMAScript/WebAssembly
- LLVM:
