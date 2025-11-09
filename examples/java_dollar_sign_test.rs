use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    println!("Testing Dollar Sign Handling in Java\n");
    println!("{}", "=".repeat(60));

    // Test 1: Can we use $$ to escape a dollar sign?
    println!("\n1. Testing $$  escape sequence:");
    let test1: java::Tokens = quote! {
        // Using $$ should produce a single $
        Class<?> inner = Outer$$Inner.class;
    };
    println!("{}", test1.to_file_string()?);

    // Test 2: Dollar in class name reference
    println!("\n2. Full example with nested class:");
    let test2: java::Tokens = quote! {
        public class Example {
            public static void main(String[] args) {
                Class<?> innerClass = Outer$$Inner.class;
                String name = innerClass.getName();
                System.out.println(name);
            }
        }
    };
    println!("{}", test2.to_file_string()?);

    // Test 3: Multiple dollar signs
    println!("\n3. Multiple dollar signs:");
    let test3: java::Tokens = quote! {
        Outer$$Inner$$Nested$$Class reference;
    };
    println!("{}", test3.to_file_string()?);

    // Test 4: Dollar in identifier (uncommon but legal in Java)
    println!("\n4. Dollar in variable name:");
    let test4: java::Tokens = quote! {
        int $$count = 0;
        String $$name = "test";
    };
    println!("{}", test4.to_file_string()?);

    println!("\n{}", "=".repeat(60));
    println!("\nConclusion:");
    println!("- Use $$ in quote! to produce $ in Java code");
    println!("- Example: Outer$$Inner becomes Outer$Inner in output");
    println!("- This is the standard escape mechanism in genco");

    Ok(())
}
