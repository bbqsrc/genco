//! Tests for Swift 6.x features including ownership keywords, negative types, and decorators.

use genco::prelude::*;

#[test]
fn test_ownership_consuming() {
    let consuming = swift::consuming();
    let toks: swift::Tokens = quote!($consuming value: String);

    assert_eq!("consuming value: String", toks.to_string().unwrap());
}

#[test]
fn test_ownership_borrowing() {
    let borrowing = swift::borrowing();
    let toks: swift::Tokens = quote!($borrowing value: String);

    assert_eq!("borrowing value: String", toks.to_string().unwrap());
}

#[test]
fn test_ownership_inout() {
    let inout_mod = swift::inout_modifier();
    let toks: swift::Tokens = quote!($inout_mod value: String);

    assert_eq!("inout value: String", toks.to_string().unwrap());
}

#[test]
fn test_function_with_ownership() {
    let consuming = swift::consuming();
    let borrowing = swift::borrowing();
    let inout_mod = swift::inout_modifier();

    let toks: swift::Tokens = quote! {
        func process($consuming data: Data, $borrowing config: Config, $inout_mod cache: Cache) {
            // Process the data
        }
    };

    assert_eq!(
        vec![
            "func process(consuming data: Data, borrowing config: Config, inout cache: Cache) {",
            "",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

#[test]
fn test_non_copyable() {
    let non_copy = swift::non_copyable();
    let toks: swift::Tokens = quote! {
        struct FileHandle: $non_copy {
            let descriptor: Int32

            deinit {
                close(descriptor)
            }
        }
    };

    assert_eq!(
        vec![
            "struct FileHandle: ~Copyable {",
            "    let descriptor: Int32",
            "",
            "    deinit {",
            "        close(descriptor)",
            "    }",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

#[test]
fn test_non_sendable() {
    let non_sendable = swift::non_sendable();
    let toks: swift::Tokens = quote!($non_sendable);

    assert_eq!("~Sendable", toks.to_string().unwrap());
}

#[test]
fn test_protocol_conformance() {
    let sendable = swift::protocol_conformance("Sendable", false);
    let toks: swift::Tokens = quote!($sendable);

    assert_eq!("Sendable", toks.to_string().unwrap());
}

#[test]
fn test_protocol_conformance_negated() {
    let not_equatable = swift::protocol_conformance("Equatable", true);
    let toks: swift::Tokens = quote!($not_equatable);

    assert_eq!("~Equatable", toks.to_string().unwrap());
}

#[test]
fn test_generic_with_non_copyable() {
    let non_copy = swift::non_copyable();
    let consuming = swift::consuming();

    let toks: swift::Tokens = quote! {
        func process<T: $non_copy>($consuming value: T) {
            // Process move-only type
        }
    };

    assert_eq!(
        vec![
            "func process<T: ~Copyable>(consuming value: T) {",
            "",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

#[test]
fn test_property_wrapper_no_args() {
    let state = swift::property_wrapper("State", None::<&str>);
    let toks: swift::Tokens = quote!($state var count: Int = 0);

    assert_eq!("@State var count: Int = 0", toks.to_string().unwrap());
}

#[test]
fn test_property_wrapper_with_args() {
    let published = swift::property_wrapper("Published", Some("initialValue: 0"));
    let toks: swift::Tokens = quote!($published var count: Int);

    assert_eq!("@Published(initialValue: 0) var count: Int", toks.to_string().unwrap());
}

#[test]
fn test_multiple_property_wrappers() {
    let state = swift::property_wrapper("State", None::<&str>);
    let binding = swift::property_wrapper("Binding", None::<&str>);
    let published = swift::property_wrapper("Published", None::<&str>);

    let toks: swift::Tokens = quote! {
        struct ViewModel {
            $state var count: Int = 0
            $binding var name: String
            $published var isActive: Bool
        }
    };

    assert_eq!(
        vec![
            "struct ViewModel {",
            "    @State var count: Int = 0",
            "    @Binding var name: String",
            "    @Published var isActive: Bool",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

#[test]
fn test_attached_macro() {
    let member_macro = swift::attached_macro("member", "named(_:)");
    let toks: swift::Tokens = quote!($member_macro);

    assert_eq!("@attached(member, names: named(_:))", toks.to_string().unwrap());
}

#[test]
fn test_freestanding_macro() {
    let expr_macro = swift::freestanding_macro("expression");
    let toks: swift::Tokens = quote!($expr_macro);

    assert_eq!("@freestanding(expression)", toks.to_string().unwrap());
}

#[test]
fn test_result_builder() {
    let builder = swift::result_builder();
    let toks: swift::Tokens = quote! {
        $builder
        struct ViewBuilder {
            static func buildBlock(_ components: View...) -> some View {
                components
            }
        }
    };

    assert_eq!(
        vec![
            "@resultBuilder",
            "struct ViewBuilder {",
            "    static func buildBlock(_ components: View...) -> some View {",
            "        components",
            "    }",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

#[test]
fn test_main_actor() {
    let main_actor = swift::main_actor();
    let toks: swift::Tokens = quote! {
        $main_actor
        class ViewController {
            func updateUI() {
                // This runs on the main thread
            }
        }
    };

    assert_eq!(
        vec![
            "@MainActor",
            "class ViewController {",
            "    func updateUI() {",
            "",
            "    }",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

#[test]
fn test_swiftui_view_with_property_wrappers() {
    let state = swift::property_wrapper("State", None::<&str>);
    let binding = swift::property_wrapper("Binding", None::<&str>);
    let main_actor = swift::main_actor();

    let toks: swift::Tokens = quote! {
        $main_actor
        struct CounterView: View {
            $state private var count: Int = 0
            $binding var isEnabled: Bool

            var body: some View {
                VStack {
                    Text("Count: \\(count)")
                    Button("Increment") {
                        count += 1
                    }
                }
            }
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("@MainActor")));
    assert!(output.iter().any(|s| s.contains("@State private var count: Int = 0")));
    assert!(output.iter().any(|s| s.contains("@Binding var isEnabled: Bool")));
}

#[test]
fn test_move_only_type_with_consuming() {
    let non_copy = swift::non_copyable();
    let consuming = swift::consuming();

    let toks: swift::Tokens = quote! {
        struct FileDescriptor: $non_copy {
            private let fd: Int32

            init(path: String) throws {
                fd = open(path, O_RDONLY)
                guard fd >= 0 else {
                    throw FileError.openFailed
                }
            }

            func read($consuming self, into buffer: UnsafeMutablePointer<UInt8>, count: Int) -> Int {
                Darwin.read(fd, buffer, count)
            }

            deinit {
                close(fd)
            }
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("struct FileDescriptor: ~Copyable {")));
    assert!(output.iter().any(|s| s.contains("func read(consuming self, into buffer: UnsafeMutablePointer<UInt8>, count: Int) -> Int {")));
}

#[test]
fn test_macro_definition() {
    let attached = swift::attached_macro("member", "named(wrappedValue), named(projectedValue)");
    let freestanding = swift::freestanding_macro("expression");

    let toks: swift::Tokens = quote! {
        $attached
        public macro AddCompletionHandler() = #externalMacro(module: "MacroKit", type: "AddCompletionHandlerMacro")

        $freestanding
        public macro stringify<T>(_ value: T) -> (T, String) = #externalMacro(module: "MacroKit", type: "StringifyMacro")
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("@attached(member, names: named(wrappedValue), named(projectedValue))")));
    assert!(output.iter().any(|s| s.contains("@freestanding(expression)")));
}

#[test]
fn test_complex_swift_6_example() {
    let non_copy = swift::non_copyable();
    let consuming = swift::consuming();
    let borrowing = swift::borrowing();
    let main_actor = swift::main_actor();
    let published = swift::property_wrapper("Published", None::<&str>);

    let toks: swift::Tokens = quote! {
        // Non-copyable resource wrapper
        struct Resource: $non_copy {
            private let handle: Int32

            init(path: String) throws {
                self.handle = acquire(path)
            }

            func use($borrowing self) -> Data {
                read(handle)
            }

            $consuming func close() {
                release(handle)
            }

            deinit {
                if handle >= 0 {
                    release(handle)
                }
            }
        }

        // MainActor-isolated observable object
        $main_actor
        class ResourceManager: ObservableObject {
            $published var resources: [String] = []

            func addResource($consuming resource: Resource) {
                let data = resource.use()
                resources.append(data.description)
                resource.close()
            }
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("struct Resource: ~Copyable {")));
    assert!(output.iter().any(|s| s.contains("func use(borrowing self) -> Data {")));
    assert!(output.iter().any(|s| s.contains("consuming func close() {")));
    assert!(output.iter().any(|s| s.contains("@MainActor")));
    assert!(output.iter().any(|s| s.contains("@Published var resources: [String] = []")));
    assert!(output.iter().any(|s| s.contains("func addResource(consuming resource: Resource) {")));
}

#[test]
fn test_all_ownership_modifiers_in_one_function() {
    let consuming = swift::consuming();
    let borrowing = swift::borrowing();
    let inout_mod = swift::inout_modifier();

    let toks: swift::Tokens = quote! {
        func complexOperation(
            $consuming ownedValue: Data,
            $borrowing sharedValue: Config,
            $inout_mod mutableValue: Cache,
            normalValue: String
        ) -> Result {
            // Complex operation using all parameter types
        }
    };

    assert_eq!(
        vec![
            "func complexOperation(",
            "    consuming ownedValue: Data,",
            "    borrowing sharedValue: Config,",
            "    inout mutableValue: Cache,",
            "    normalValue: String",
            ") -> Result {",
            "",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

// ========== Async/Await and Concurrency Tests ==========

#[test]
fn test_async_modifier() {
    let async_mod = swift::async_modifier();
    let toks: swift::Tokens = quote!(func fetchData() $async_mod -> Data);

    assert_eq!("func fetchData() async -> Data", toks.to_string().unwrap());
}

#[test]
fn test_await_keyword() {
    let await_kw = swift::await_keyword();
    let toks: swift::Tokens = quote!(let data = $await_kw fetchData());

    assert_eq!("let data = await fetchData()", toks.to_string().unwrap());
}

#[test]
fn test_throws_modifier() {
    let throws_mod = swift::throws_modifier();
    let toks: swift::Tokens = quote!(func riskyOperation() $throws_mod -> Result);

    assert_eq!("func riskyOperation() throws -> Result", toks.to_string().unwrap());
}

#[test]
fn test_async_throws_function() {
    let async_mod = swift::async_modifier();
    let throws_mod = swift::throws_modifier();
    let await_kw = swift::await_keyword();

    let toks: swift::Tokens = quote! {
        func loadData() $async_mod $throws_mod -> Data {
            let response = $await_kw URLSession.shared.data(from: url)
            return response.0
        }
    };

    assert_eq!(
        vec![
            "func loadData() async throws -> Data {",
            "    let response = await URLSession.shared.data(from: url)",
            "    return response.0",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

#[test]
fn test_actor_type() {
    let actor_kw = swift::actor_type();

    let toks: swift::Tokens = quote! {
        $actor_kw Counter {
            var value: Int = 0

            func increment() {
                value += 1
            }
        }
    };

    assert_eq!(
        vec![
            "actor Counter {",
            "    var value: Int = 0",
            "",
            "    func increment() {",
            "        value += 1",
            "    }",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

#[test]
fn test_nonisolated_modifier() {
    let nonisolated_mod = swift::nonisolated_modifier();

    let toks: swift::Tokens = quote! {
        actor MyActor {
            $nonisolated_mod func helper() -> String {
                return "I don't need actor isolation"
            }
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("nonisolated func helper() -> String {")));
}

#[test]
fn test_isolated_modifier() {
    let isolated_mod = swift::isolated_modifier();
    let toks: swift::Tokens = quote!(func process($isolated_mod actor: MyActor));

    assert_eq!("func process(isolated actor: MyActor)", toks.to_string().unwrap());
}

// ========== Type Modifiers Tests ==========

#[test]
fn test_any_type() {
    let any_type = swift::any_type();
    let toks: swift::Tokens = quote!(let items: [$any_type Collection]);

    assert_eq!("let items: [any Collection]", toks.to_string().unwrap());
}

#[test]
fn test_some_type() {
    let some_type = swift::some_type();
    let toks: swift::Tokens = quote!(var body: $some_type View);

    assert_eq!("var body: some View", toks.to_string().unwrap());
}

#[test]
fn test_swiftui_view_with_some() {
    let some_type = swift::some_type();
    let state = swift::property_wrapper("State", None::<&str>);

    let toks: swift::Tokens = quote! {
        struct ContentView: View {
            $state private var count: Int = 0

            var body: $some_type View {
                Text("Count: \\(count)")
            }
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("var body: some View {")));
}

#[test]
fn test_any_protocol_collection() {
    let any_type = swift::any_type();

    let toks: swift::Tokens = quote! {
        func processShapes(shapes: [$any_type Shape]) {
            for shape in shapes {
                shape.draw()
            }
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("func processShapes(shapes: [any Shape]) {")));
}

// ========== Typed Throws Tests ==========

#[test]
fn test_typed_throws() {
    let typed_throws = swift::typed_throws("NetworkError");
    let toks: swift::Tokens = quote!(func fetch() $typed_throws -> Data);

    assert_eq!("func fetch() throws(NetworkError) -> Data", toks.to_string().unwrap());
}

#[test]
fn test_async_typed_throws() {
    let async_mod = swift::async_modifier();
    let typed_throws = swift::typed_throws("APIError");
    let await_kw = swift::await_keyword();

    let toks: swift::Tokens = quote! {
        func loadUser(id: String) $async_mod $typed_throws -> User {
            let data = $await_kw fetch(url: "/users/\\(id)")
            return try decode(data)
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("func loadUser(id: String) async throws(APIError) -> User {")));
}

// ========== Observable Tests ==========

#[test]
fn test_observable() {
    let observable = swift::observable();

    let toks: swift::Tokens = quote! {
        $observable
        class UserModel {
            var name: String = ""
            var age: Int = 0
        }
    };

    assert_eq!(
        vec![
            "@Observable",
            "class UserModel {",
            "    var name: String = \"\"",
            "    var age: Int = 0",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

// ========== Global Actor Tests ==========

#[test]
fn test_global_actor() {
    let ui_actor = swift::global_actor("UIActor");

    let toks: swift::Tokens = quote! {
        $ui_actor
        class UIManager {
            func updateUI() {}
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("@UIActor")));
}

#[test]
fn test_custom_database_actor() {
    let db_actor = swift::global_actor("DatabaseActor");

    let toks: swift::Tokens = quote! {
        $db_actor
        class DatabaseManager {
            func execute(query: String) {}
        }
    };

    assert_eq!(
        vec![
            "@DatabaseActor",
            "class DatabaseManager {",
            "    func execute(query: String) {}",
            "}",
        ],
        toks.to_file_vec().unwrap()
    );
}

// ========== Package Access Tests ==========

#[test]
fn test_package_access() {
    let package_mod = swift::package_access();
    let toks: swift::Tokens = quote!($package_mod class InternalUtility {});

    assert_eq!("package class InternalUtility {}", toks.to_string().unwrap());
}

#[test]
fn test_package_access_function() {
    let package_mod = swift::package_access();

    let toks: swift::Tokens = quote! {
        $package_mod func helperFunction() -> String {
            return "internal helper"
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("package func helperFunction() -> String {")));
}

// ========== Combined Feature Tests ==========

#[test]
fn test_comprehensive_swift_6_example() {
    let actor_kw = swift::actor_type();
    let async_mod = swift::async_modifier();
    let throws_mod = swift::throws_modifier();
    let await_kw = swift::await_keyword();
    let nonisolated_mod = swift::nonisolated_modifier();
    let main_actor = swift::main_actor();
    let some_type = swift::some_type();
    let any_type = swift::any_type();
    let observable = swift::observable();

    let toks: swift::Tokens = quote! {
        $actor_kw DataStore {
            private var cache: [String: Data] = [:]

            func fetch(key: String) $async_mod $throws_mod -> Data? {
                if let cached = cache[key] {
                    return cached
                }
                let data = $await_kw loadFromNetwork(key: key)
                cache[key] = data
                return data
            }

            $nonisolated_mod func description() -> String {
                return "DataStore"
            }
        }

        $observable
        class ViewModel {
            var items: [$any_type Identifiable] = []
        }

        $main_actor
        struct ContentView: View {
            var body: $some_type View {
                Text("Hello")
            }
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("actor DataStore {")));
    assert!(output.iter().any(|s| s.contains("func fetch(key: String) async throws -> Data? {")));
    assert!(output.iter().any(|s| s.contains("nonisolated func description() -> String {")));
    assert!(output.iter().any(|s| s.contains("@Observable")));
    assert!(output.iter().any(|s| s.contains("var items: [any Identifiable] = []")));
    assert!(output.iter().any(|s| s.contains("@MainActor")));
    assert!(output.iter().any(|s| s.contains("var body: some View {")));
}

#[test]
fn test_all_new_features_combined() {
    let actor_kw = swift::actor_type();
    let async_mod = swift::async_modifier();
    let typed_throws = swift::typed_throws("NetworkError");
    let await_kw = swift::await_keyword();
    let isolated_mod = swift::isolated_modifier();
    let some_type = swift::some_type();
    let any_type = swift::any_type();
    let global_actor = swift::global_actor("NetworkActor");
    let package_mod = swift::package_access();

    let toks: swift::Tokens = quote! {
        $global_actor
        $actor_kw NetworkManager {
            $package_mod func request(url: String) $async_mod $typed_throws -> $some_type Response {
                return $await_kw URLSession.shared.data(from: URL(string: url)!)
            }
        }

        func processData($isolated_mod manager: NetworkManager, items: [$any_type Codable]) $async_mod {
            for item in items {
                print(item)
            }
        }
    };

    let output = toks.to_file_vec().unwrap();
    assert!(output.iter().any(|s| s.contains("@NetworkActor")));
    assert!(output.iter().any(|s| s.contains("actor NetworkManager {")));
    assert!(output.iter().any(|s| s.contains("package func request(url: String) async throws(NetworkError) -> some Response {")));
    assert!(output.iter().any(|s| s.contains("func processData(isolated manager: NetworkManager, items: [any Codable]) async {")));
}
