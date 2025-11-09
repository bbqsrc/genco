use anyhow::Result;
use genco::prelude::*;

fn main() -> Result<()> {
    let _foundation = swift::import("Foundation", "FileManager");

    // Ownership keywords
    let non_copy = swift::non_copyable();
    let consuming = swift::consuming();
    let borrowing = swift::borrowing();

    // Concurrency features
    let actor_kw = swift::actor_type();
    let async_mod = swift::async_modifier();
    let await_kw = swift::await_keyword();
    let nonisolated_mod = swift::nonisolated_modifier();
    let main_actor = swift::main_actor();

    // Type modifiers
    let some_type = swift::some_type();
    let any_type = swift::any_type();

    // Modern decorators
    let observable = swift::observable();
    let typed_throws = swift::typed_throws("NetworkError");
    let global_actor = swift::global_actor("DatabaseActor");

    // Access control
    let package_mod = swift::package_access();

    let file_tokens: swift::Tokens = quote! {
        // Non-copyable file descriptor with ownership keywords
        struct FileDescriptor: $non_copy {
            private let fd: Int32

            init(path: String) throws {
                fd = open(path, O_RDONLY)
                guard fd >= 0 else {
                    throw FileError.openFailed
                }
            }

            func read($borrowing self, into buffer: UnsafeMutablePointer<UInt8>, count: Int) -> Int {
                Darwin.read(fd, buffer, count)
            }

            $consuming func close() {
                Darwin.close(fd)
            }

            deinit {
                if fd >= 0 {
                    Darwin.close(fd)
                }
            }
        }

        // Custom global actor for database operations
        $global_actor
        $actor_kw DatabaseManager {
            private var connections: [Connection] = []

            func execute(query: String) $async_mod $typed_throws -> [Row] {
                let connection = $await_kw getConnection()
                return $await_kw connection.query(query)
            }

            $nonisolated_mod func maxConnections() -> Int {
                return 10
            }
        }

        // Observable model with modern Swift features
        $observable
        class DataModel {
            var users: [$any_type Identifiable] = []
            var selectedUser: User?
        }

        // MainActor-isolated file manager
        $main_actor
        class FileManager: ObservableObject {
            $(swift::property_wrapper("Published", None::<&str>)) var openFiles: [String] = []
            private var descriptors: [FileDescriptor] = []

            func openFile(path: String) $async_mod throws {
                let descriptor = try FileDescriptor(path: path)
                descriptors.append(descriptor)
                openFiles.append(path)
            }

            func readFile($borrowing descriptor: FileDescriptor) -> Data {
                var buffer = [UInt8](repeating: 0, count: 1024)
                let bytesRead = buffer.withUnsafeMutableBufferPointer { ptr in
                    descriptor.read(into: ptr.baseAddress!, count: 1024)
                }
                return Data(buffer.prefix(bytesRead))
            }

            func closeFile($consuming descriptor: FileDescriptor) {
                descriptor.close()
            }
        }

        // SwiftUI view with opaque return type
        $main_actor
        struct FileViewer: View {
            $(swift::property_wrapper("State", None::<&str>)) private var selectedFile: String = ""
            $(swift::property_wrapper("ObservedObject", None::<&str>)) var manager: FileManager

            var body: $some_type View {
                List(manager.openFiles, id: $$(r"\.self")) { file in
                    Text(file)
                        .onTapGesture {
                            selectedFile = file
                        }
                }
            }
        }

        // Package-level utility function
        $package_mod func sanitizePath(_ path: String) -> String {
            return path.replacingOccurrences(of: "..", with: "")
        }
    };

    println!("{}", file_tokens.to_file_string()?);

    Ok(())
}
