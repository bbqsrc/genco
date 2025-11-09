use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    println!("Testing Java 11 Syntax Support in quote! macro\n");
    println!("{}", "=".repeat(60));

    // Test 1: Basic class structure
    println!("\n1. Basic Class Structure:");
    let test1: java::Tokens = quote! {
        public class Test {
            private int x;
            public void method() {}
        }
    };
    println!("{}", test1.to_file_string()?);

    // Test 2: Generics
    println!("\n2. Generics:");
    let list = java::import("java.util", "List");
    let test2: java::Tokens = quote! {
        $list<String> names;
        Map<String, Integer> map;
        List<? extends Number> numbers;
        List<? super Integer> ints;
    };
    println!("{}", test2.to_file_string()?);

    // Test 3: Annotations
    println!("\n3. Annotations:");
    let test3: java::Tokens = quote! {
        @Override
        @Deprecated
        @SuppressWarnings("unchecked")
        @Retention(RetentionPolicy.RUNTIME)
        @Target({ElementType.METHOD, ElementType.FIELD})
        public void annotatedMethod() {}
    };
    println!("{}", test3.to_file_string()?);

    // Test 4: Lambda expressions
    println!("\n4. Lambda Expressions:");
    let test4: java::Tokens = quote! {
        Runnable r = () -> System.out.println("Hello");
        Function<String, Integer> f = s -> s.length();
        BiFunction<Integer, Integer, Integer> add = (a, b) -> a + b;
    };
    println!("{}", test4.to_file_string()?);

    // Test 5: Method references
    println!("\n5. Method References:");
    let test5: java::Tokens = quote! {
        Function<String, Integer> f1 = String::length;
        Supplier<String> s = String::new;
        BiFunction<String, String, Boolean> eq = String::equals;
    };
    println!("{}", test5.to_file_string()?);

    // Test 6: Diamond operator
    println!("\n6. Diamond Operator:");
    let test6: java::Tokens = quote! {
        List<String> list = new ArrayList<>();
        Map<String, List<Integer>> map = new HashMap<>();
    };
    println!("{}", test6.to_file_string()?);

    // Test 7: Try-with-resources
    println!("\n7. Try-with-resources:");
    let test7: java::Tokens = quote! {
        try (BufferedReader br = new BufferedReader(new FileReader("file.txt"))) {
            return br.readLine();
        } catch (IOException e) {
            e.printStackTrace();
        }
    };
    println!("{}", test7.to_file_string()?);

    // Test 8: var keyword (Java 10)
    println!("\n8. Local Variable Type Inference (var):");
    let test8: java::Tokens = quote! {
        var list = new ArrayList<String>();
        var stream = list.stream();
        for (var item : list) {
            System.out.println(item);
        }
    };
    println!("{}", test8.to_file_string()?);

    // Test 9: var in lambda (Java 11)
    println!("\n9. Var in Lambda Parameters:");
    let test9: java::Tokens = quote! {
        BiFunction<Integer, Integer, Integer> add = (var a, var b) -> a + b;
    };
    println!("{}", test9.to_file_string()?);

    // Test 10: Static imports (as strings)
    println!("\n10. Static Import Simulation:");
    let test10: java::Tokens = quote! {
        // Would need: import static java.lang.Math.PI;
        double circumference = 2 * PI * radius;
    };
    println!("{}", test10.to_file_string()?);

    // Test 11: Nested classes
    println!("\n11. Nested Classes:");
    let test11: java::Tokens = quote! {
        public class Outer {
            private class Inner {
                void innerMethod() {}
            }

            static class StaticNested {
                void nestedMethod() {}
            }
        }
    };
    println!("{}", test11.to_file_string()?);

    // Test 12: Interfaces with default methods
    println!("\n12. Interface with Default Methods:");
    let test12: java::Tokens = quote! {
        interface MyInterface {
            void abstractMethod();

            default void defaultMethod() {
                System.out.println("Default");
            }

            static void staticMethod() {
                System.out.println("Static");
            }
        }
    };
    println!("{}", test12.to_file_string()?);

    // Test 13: Enum
    println!("\n13. Enum:");
    let test13: java::Tokens = quote! {
        enum Color {
            RED(255, 0, 0),
            GREEN(0, 255, 0),
            BLUE(0, 0, 255);

            private final int r, g, b;

            Color(int r, int g, int b) {
                this.r = r;
                this.g = g;
                this.b = b;
            }
        }
    };
    println!("{}", test13.to_file_string()?);

    // Test 14: Multi-catch
    println!("\n14. Multi-catch:");
    let test14: java::Tokens = quote! {
        try {
            // code
        } catch (IOException | SQLException e) {
            e.printStackTrace();
        }
    };
    println!("{}", test14.to_file_string()?);

    // Test 15: String concatenation and special characters
    println!("\n15. String Handling:");
    let test15: java::Tokens = quote! {
        String s1 = "Hello";
        String s2 = "World";
        String s3 = s1 + " " + s2;
        String unicode = "π ≈ 3.14";
        String emoji = "😊";
    };
    println!("{}", test15.to_file_string()?);

    // Test 16: Array initialization
    println!("\n16. Array Initialization:");
    let test16: java::Tokens = quote! {
        int[] arr1 = {1, 2, 3, 4, 5};
        String[] arr2 = new String[] {"a", "b", "c"};
        int[][] matrix = {{1, 2}, {3, 4}};
    };
    println!("{}", test16.to_file_string()?);

    // Test 17: Complex generic types
    println!("\n17. Complex Generic Types:");
    let test17: java::Tokens = quote! {
        Map<String, List<Set<Integer>>> complex;
        Function<List<String>, Map<String, Integer>> mapper;
    };
    println!("{}", test17.to_file_string()?);

    // Test 18: Varargs
    println!("\n18. Varargs:");
    let test18: java::Tokens = quote! {
        public void method(String... args) {
            for (String arg : args) {
                System.out.println(arg);
            }
        }
    };
    println!("{}", test18.to_file_string()?);

    // Test 19: Instanceof and casting
    println!("\n19. Instanceof and Casting:");
    let test19: java::Tokens = quote! {
        if (obj instanceof String) {
            String s = (String) obj;
        }
    };
    println!("{}", test19.to_file_string()?);

    // Test 20: Ternary operator
    println!("\n20. Ternary Operator:");
    let test20: java::Tokens = quote! {
        int max = (a > b) ? a : b;
        String result = condition ? "yes" : "no";
    };
    println!("{}", test20.to_file_string()?);

    // Test 21: Access modifiers combinations
    println!("\n21. Access Modifiers:");
    let test21: java::Tokens = quote! {
        public static final int CONSTANT = 42;
        private volatile boolean flag;
        protected synchronized void method() {}
        public abstract class AbstractClass {}
        public final class FinalClass {}
    };
    println!("{}", test21.to_file_string()?);

    // Test 22: Module system syntax (Java 9+)
    println!("\n22. Module System (if applicable):");
    let test22: java::Tokens = quote! {
        // This would be in module-info.java:
        // module com.example.app {
        //     requires java.sql;
        //     requires transitive java.logging;
        //     exports com.example.api;
        //     opens com.example.internal;
        //     uses com.example.spi.Service;
        //     provides com.example.spi.Service with com.example.impl.ServiceImpl;
        // }
    };
    println!("{}", test22.to_file_string()?);

    // Test 23: Problematic case - Dollar signs in identifiers
    println!("\n23. POTENTIAL ISSUE - Dollar Signs:");
    println!("Java allows $ in identifiers, but quote! uses $ for interpolation");
    println!("Example: Inner class references like Outer$Inner");

    // This will likely fail or behave unexpectedly:
    // let test23 = quote! {
    //     Class<?> inner = Outer$Inner.class;
    // };

    // Workaround using string escape:
    let test23: java::Tokens = quote! {
        // Need to escape: Outer$$Inner for Outer$Inner
    };
    println!("{}", test23.to_file_string()?);

    println!("\n{}", "=".repeat(60));
    println!("Testing complete!");

    Ok(())
}
