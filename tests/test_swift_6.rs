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
