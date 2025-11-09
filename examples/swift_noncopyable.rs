use anyhow::Result;
use genco::prelude::*;

fn main() -> Result<()> {
    // Non-copyable type constraint
    let non_copy = swift::non_copyable();

    // Ownership keywords
    let consuming = swift::consuming();
    let borrowing = swift::borrowing();
    let inout_mod = swift::inout_modifier();

    // Availability attribute
    let available_ios15 = swift::available("iOS 15, *");
    let available_ios16 = swift::available("iOS 16, macOS 13, *");

    // Async/concurrency
    let async_mod = swift::async_modifier();
    let throws_mod = swift::throws_modifier();
    let await_kw = swift::await_keyword();

    let toks: swift::Tokens = quote! {
        import Foundation

        // Non-copyable file handle structure
        struct FileHandle: $non_copy {
            private let descriptor: Int32
            private let path: String

            init(path: String) throws {
                self.path = path
                self.descriptor = open(path, O_RDWR)
                guard self.descriptor >= 0 else {
                    throw FileError.openFailed(path)
                }
            }

            deinit {
                if descriptor >= 0 {
                    close(descriptor)
                }
            }
        }

        // Extension with various parameter ownership combinations
        extension FileHandle {
            // Consuming function - takes ownership and closes the file
            $consuming func close() {
                Darwin.close(descriptor)
            }

            // Borrowing function - reads without taking ownership
            $borrowing func read(maxBytes: Int) -> Data {
                var buffer = Data(count: maxBytes)
                let bytesRead = buffer.withUnsafeMutableBytes { ptr in
                    Darwin.read(descriptor, ptr.baseAddress!, maxBytes)
                }
                return buffer.prefix(bytesRead)
            }

            // Function with mixed parameter ownership
            $borrowing func copyTo(
                destination: $consuming FileHandle,
                $inout_mod bytesWritten: Int
            ) throws {
                let data = self.read(maxBytes: 4096)
                let written = destination.write(data)
                bytesWritten += written
            }

            // Available since iOS 15 - borrowing with async
            $available_ios15
            $borrowing func readAsync(maxBytes: Int) $async_mod throws -> Data {
                return $await_kw Task.detached {
                    self.read(maxBytes: maxBytes)
                }.value
            }

            // Available since iOS 16 - complex parameter mix
            $available_ios16
            $consuming func transfer(
                to destination: $borrowing FileHandle,
                $inout_mod progress: TransferProgress,
                bufferSize: Int = 4096
            ) $async_mod throws {
                var totalBytes = 0
                while true {
                    let chunk = $await_kw self.readAsync(maxBytes: bufferSize)
                    if chunk.isEmpty { break }

                    let written = destination.write(chunk)
                    totalBytes += written
                    progress.update(bytes: totalBytes)
                }
                self.close()
            }

            // Mutating function for atomic operations
            mutating func updateMetadata(newSize: Int64) throws {
                guard ftruncate(descriptor, newSize) == 0 else {
                    throw FileError.resizeFailed
                }
            }

            // Multiple borrowing parameters
            static func compare(
                $borrowing lhs: FileHandle,
                $borrowing rhs: FileHandle,
                chunkSize: Int = 1024
            ) -> Bool {
                let leftData = lhs.read(maxBytes: chunkSize)
                let rightData = rhs.read(maxBytes: chunkSize)
                return leftData == rightData
            }

            // Mixed: consuming self, borrowing other, inout result
            $consuming func merge(
                with other: $borrowing FileHandle,
                $inout_mod result: MergeResult
            ) throws {
                result.filesProcessed += 1

                // Read all from self
                var allData = Data()
                while true {
                    let chunk = self.read(maxBytes: 4096)
                    if chunk.isEmpty { break }
                    allData.append(chunk)
                }

                // Read from other without consuming it
                let otherData = other.read(maxBytes: Int.max)
                allData.append(otherData)

                result.totalBytes = allData.count
                self.close()
            }

            // Private helper with borrowing
            private $borrowing func write(_ data: Data) -> Int {
                data.withUnsafeBytes { ptr in
                    Darwin.write(descriptor, ptr.baseAddress!, data.count)
                }
            }
        }

        // Supporting types
        struct TransferProgress {
            var bytesTransferred: Int = 0

            mutating func update(bytes: Int) {
                bytesTransferred = bytes
            }
        }

        struct MergeResult {
            var filesProcessed: Int = 0
            var totalBytes: Int = 0
        }

        enum FileError: Error {
            case openFailed(String)
            case resizeFailed
            case transferFailed
        }

        // Usage example
        func demonstrateOwnership() throws {
            // Create two file handles (non-copyable)
            let source = try FileHandle(path: "/tmp/source.txt")
            let dest = try FileHandle(path: "/tmp/dest.txt")

            // Borrowing - can still use source after this
            let data = source.read(maxBytes: 1024)
            print("Read \\(data.count) bytes")

            // Mixed ownership
            var bytesWritten = 0
            try source.copyTo(destination: dest, &bytesWritten)
            print("Wrote \\(bytesWritten) bytes")

            // Consuming - source is moved and closed
            source.close()
            // source is no longer accessible here
        }
    };

    println!("{}", toks.to_file_string()?);

    Ok(())
}
