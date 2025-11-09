use anyhow::Result;
use genco::prelude::*;

fn main() -> Result<()> {
    let _foundation = swift::import("Foundation", "FileManager");
    let non_copy = swift::non_copyable();
    let consuming = swift::consuming();
    let borrowing = swift::borrowing();

    let file_tokens: swift::Tokens = quote! {
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

        // MainActor-isolated observable object
        $(swift::main_actor())
        class FileManager: ObservableObject {
            $(swift::property_wrapper("Published", None::<&str>)) var openFiles: [String] = []
            private var descriptors: [FileDescriptor] = []

            func openFile(path: String) throws {
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
        $(swift::main_actor())
        struct FileViewer: View {
            $(swift::property_wrapper("State", None::<&str>)) private var selectedFile: String = ""
            $(swift::property_wrapper("Published", None::<&str>)) var files: [String] = []

            var body: some View {
                List(files, id: $$(r"\.self")) { file in
                    Text(file)
                        .onTapGesture {
                            selectedFile = file
                        }
                }
            }
        }
    };

    println!("{}", file_tokens.to_file_string()?);

    Ok(())
}
