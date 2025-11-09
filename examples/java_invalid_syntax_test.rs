use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    println!("Testing Invalid Java Syntax Handling\n");
    println!("{}", "=".repeat(60));

    // Test 1: Completely nonsensical syntax
    println!("\n1. Nonsensical syntax - Should PASS tokenization:");
    let test1: java::Tokens = quote! {
        public class void int return if else abstract {
            private public protected class method() {
                return new void();
            }
        }
    };
    println!("✅ Tokenized successfully (but invalid Java):");
    println!("{}\n", test1.to_file_string()?);

    // Test 2: Unbalanced braces - Should FAIL tokenization
    println!("\n2. Unbalanced braces - Should FAIL:");
    // This won't compile because Rust itself checks balanced delimiters
    // let test2: java::Tokens = quote! {
    //     public class Test {
    //         void method() {
    //     }
    // };
    println!("❌ Cannot test - Rust compiler enforces balanced delimiters\n");

    // Test 3: Mismatched brackets
    println!("\n3. Mismatched brackets - Should FAIL:");
    // let test3: java::Tokens = quote! {
    //     int[] arr = {1, 2, 3];
    // };
    println!("❌ Cannot test - Rust compiler enforces matching delimiters\n");

    // Test 4: Invalid Java keywords in wrong positions
    println!("\n4. Keywords in wrong positions - Should PASS tokenization:");
    let test4: java::Tokens = quote! {
        abstract volatile synchronized final static class Test {
            public private protected void method() {}
            int float double byte short long char boolean void x;
        }
    };
    println!("✅ Tokenized successfully (but invalid Java):");
    println!("{}\n", test4.to_file_string()?);

    // Test 5: Invalid operators/expressions
    println!("\n5. Invalid operators - Should PASS tokenization:");
    let test5: java::Tokens = quote! {
        int x = ++++----****/////;
        boolean b = !!!!!!!!true;
        String s = ........field;
    };
    println!("✅ Tokenized successfully (but invalid Java):");
    println!("{}\n", test5.to_file_string()?);

    // Test 6: Type system violations
    println!("\n6. Type system violations - Should PASS tokenization:");
    let test6: java::Tokens = quote! {
        int x = "string";
        String s = 123;
        void v = new Object();
        int method() { return "not an int"; }
    };
    println!("✅ Tokenized successfully (but invalid Java):");
    println!("{}\n", test6.to_file_string()?);

    // Test 7: Missing semicolons
    println!("\n7. Missing semicolons - Should PASS tokenization:");
    let test7: java::Tokens = quote! {
        public class Test {
            int x
            String s
            void method() {
                return
            }
        }
    };
    println!("✅ Tokenized successfully (but invalid Java):");
    println!("{}\n", test7.to_file_string()?);

    // Test 8: Invalid generic syntax
    println!("\n8. Invalid generic syntax - Should PASS tokenization:");
    let test8: java::Tokens = quote! {
        List<> empty;
        Map<<String>> doubleAngle;
        Set<String, Integer, Boolean> tooMany;
    };
    println!("✅ Tokenized successfully (but invalid Java):");
    println!("{}\n", test8.to_file_string()?);

    // Test 9: Duplicate modifiers
    println!("\n9. Duplicate modifiers - Should PASS tokenization:");
    let test9: java::Tokens = quote! {
        public public public class Test {
            private private int x;
            final final final String s;
        }
    };
    println!("✅ Tokenized successfully (but invalid Java):");
    println!("{}\n", test9.to_file_string()?);

    // Test 10: Invalid annotation syntax (with balanced delimiters)
    println!("\n10. Invalid annotation syntax - Should PASS tokenization:");
    let test10: java::Tokens = quote! {
        @@@Override
        @@@@Deprecated()()()
        public void method() {}
    };
    println!("✅ Tokenized successfully (but invalid Java):");
    println!("{}\n", test10.to_file_string()?);

    println!("{}", "=".repeat(60));
    println!("\n🔍 CONCLUSION:");
    println!("The quote! macro does NOT validate Java syntax.");
    println!("It only ensures:");
    println!("  1. Balanced delimiters (enforced by Rust compiler)");
    println!("  2. Valid Rust token stream structure");
    println!("\nJava syntax validation must be done by:");
    println!("  - The Java compiler (javac)");
    println!("  - IDE tooling");
    println!("  - Custom validation in your Rust code");

    Ok(())
}
