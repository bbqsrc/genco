# Swift 6.x Compatibility Assessment

**Date:** 2025-11-09
**Project:** genco v0.19.0
**Assessed by:** Claude (Anthropic)

## Executive Summary

The current Swift language implementation in genco (`src/lang/swift.rs`) provides basic Swift code generation capabilities but **lacks support for Swift 6.x-specific features**. This assessment examines three key Swift 6.x features: decorators/macros, ownership keywords, and negative types.

## Current Implementation Status

### Supported Features ✅

The current implementation (`src/lang/swift.rs:1-219`) includes:

1. **Basic Imports** (`src/lang/swift.rs:90-99`)
   - Standard import statements: `import ModuleName`
   - Example: `import UIKit`, `import Foundation`

2. **Implementation-Only Imports** (`src/lang/swift.rs:101-124`)
   - Private imports using `@_implementationOnly import ModuleName`
   - Hides imported modules from public API
   - Prevents transitive dependencies

3. **String Quoting** (`src/lang/swift.rs:33-53`)
   - UTF-8 string handling
   - Escape sequences: `\0`, `\\`, `\t`, `\n`, `\r`, `\'`, `\"`
   - Unicode escape sequences: `\u{...}`

## Swift 6.x Feature Analysis

### 1. Decorator/Macro Support ❌ NOT SUPPORTED

**Status:** Missing
**Swift Version:** 5.9+ (macros), 6.0+ (expanded support)

#### What's Missing:

Swift 6.x supports several types of macros and decorators:

- **Attached Macros:**
  - `@attached(member)` - Adds new members to a type
  - `@attached(memberAttribute)` - Adds attributes to members
  - `@attached(accessor)` - Adds accessors to properties
  - `@attached(peer)` - Adds peers alongside declarations
  - `@attached(conformance)` - Adds protocol conformances

- **Freestanding Macros:**
  - `#expression` - Expression macros
  - `#declaration` - Declaration macros

- **Property Wrappers:**
  - `@propertyWrapper` - Custom property wrappers
  - Built-in wrappers: `@State`, `@Binding`, `@ObservedObject`, etc.

- **Result Builders:**
  - `@resultBuilder` - Custom DSL builders

#### Current Implementation:

```rust
// src/lang/swift.rs - No decorator/macro support
// Only supports basic imports and string quoting
```

#### Recommended Additions:

```rust
/// Attached macro decorator for Swift macros
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct AttachedMacro {
    /// Type of attached macro (member, memberAttribute, accessor, peer, conformance)
    macro_type: ItemStr,
    /// Additional attributes
    attributes: Vec<ItemStr>,
}

/// Property wrapper decorator
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct PropertyWrapper {
    /// Wrapper name (e.g., "State", "Binding", "Published")
    name: ItemStr,
}

/// Result builder decorator
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ResultBuilder {
    /// Builder name
    name: ItemStr,
}
```

### 2. Ownership Keywords ❌ NOT SUPPORTED

**Status:** Missing
**Swift Version:** 6.0+

#### What's Missing:

Swift 6.0 introduced ownership keywords for explicit memory management:

- **`consuming`** - Takes ownership of the value (move semantics)
  ```swift
  func process(consuming value: LargeStruct) { }
  ```

- **`borrowing`** - Borrows the value without taking ownership
  ```swift
  func inspect(borrowing value: LargeStruct) { }
  ```

- **`inout`** - Mutable borrow (existed pre-6.0 but more important now)
  ```swift
  func modify(inout value: LargeStruct) { }
  ```

#### Current Implementation:

No support for ownership annotations in function parameters or return types.

#### Impact:

- Cannot generate Swift 6.x code with explicit ownership semantics
- Missing optimization opportunities for large value types
- Cannot express move-only types properly

#### Recommended Additions:

```rust
/// Ownership modifier for function parameters
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum OwnershipModifier {
    /// Consuming - takes ownership (move semantics)
    Consuming,
    /// Borrowing - immutable borrow
    Borrowing,
    /// Inout - mutable borrow
    Inout,
}

/// Function parameter with ownership annotation
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Parameter {
    /// Ownership modifier (optional)
    ownership: Option<OwnershipModifier>,
    /// Parameter name
    name: ItemStr,
    /// Parameter type
    type_name: ItemStr,
}
```

### 3. Negative Types (~Copyable) ❌ NOT SUPPORTED

**Status:** Missing
**Swift Version:** 6.0+

#### What's Missing:

Swift 6.0 introduced **non-copyable types** using the `~Copyable` constraint:

```swift
// Define a non-copyable type
struct FileHandle: ~Copyable {
    let descriptor: Int32

    deinit {
        close(descriptor)
    }
}

// Generic function with ~Copyable constraint
func process<T: ~Copyable>(_ value: consuming T) { }

// Type that can be either copyable or non-copyable
struct Container<T: ~Copyable> {
    var value: T
}
```

#### Current Implementation:

No support for:
- `~Copyable` constraint syntax
- Move-only types
- Suppressing protocol conformances
- Generic constraints with negative types

#### Impact:

- Cannot generate code for resource-managing types (file handles, locks, etc.)
- Missing modern Swift 6.x type system features
- Cannot express move semantics at the type level

#### Recommended Additions:

```rust
/// Protocol conformance with optional suppression
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ProtocolConformance {
    /// Protocol name
    protocol: ItemStr,
    /// Whether this is a negative constraint (e.g., ~Copyable)
    negated: bool,
}

/// Generic constraint that can be positive or negative
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct GenericConstraint {
    /// Type parameter name
    type_param: ItemStr,
    /// Conformances (can include ~Copyable)
    conformances: Vec<ProtocolConformance>,
}

/// Helper function to create ~Copyable constraint
pub fn non_copyable() -> ProtocolConformance {
    ProtocolConformance {
        protocol: "Copyable".into(),
        negated: true,
    }
}
```

## Additional Swift 6.x Features Not Assessed

The following Swift 6.x features were not specifically requested but may be relevant:

1. **Typed Throws** (`throws(ErrorType)`)
2. **Existential `any` keyword** (required for protocols as types)
3. **Primary Associated Types**
4. **Regex Literals**
5. **Actor Isolation** (`nonisolated`, `isolated`)
6. **Concurrency** (`async`, `await`, `@Sendable`)

## Recommendations

### Priority 1: Critical for Swift 6.x

1. **Add Ownership Keywords Support**
   - Implement `consuming`, `borrowing`, `inout` modifiers
   - Location: `src/lang/swift.rs`
   - Impact: Enables modern Swift 6.x function signatures

2. **Add ~Copyable Support**
   - Implement negative protocol constraints
   - Support move-only types
   - Location: `src/lang/swift.rs`
   - Impact: Enables resource-managing types

### Priority 2: Important for Modern Swift

3. **Add Macro/Decorator Support**
   - Implement `@attached`, `@freestanding` macros
   - Implement `@propertyWrapper`, `@resultBuilder`
   - Location: `src/lang/swift.rs`
   - Impact: Enables modern Swift DSLs and meta-programming

### Priority 3: Nice to Have

4. **Add Typed Throws**
5. **Add Actor Isolation Keywords**
6. **Add `any` keyword for existential types**

## Testing Recommendations

Create test files to validate Swift 6.x feature generation:

```rust
// tests/swift_6_features.rs

#[test]
fn test_ownership_keywords() {
    let toks = quote! {
        func process(consuming value: String) {}
        func inspect(borrowing value: String) {}
        func modify(inout value: String) {}
    };
    // Validate output
}

#[test]
fn test_non_copyable_types() {
    let toks = quote! {
        struct FileHandle: ~Copyable {
            let descriptor: Int32
        }
    };
    // Validate output
}

#[test]
fn test_property_wrappers() {
    let toks = quote! {
        @State var count: Int = 0
        @Binding var name: String
    };
    // Validate output
}
```

## Conclusion

The current Swift implementation in genco is **basic and does not support Swift 6.x features**. To generate modern Swift 6.x code, the following additions are necessary:

1. ❌ **Decorator/Macro Support** - Not implemented
2. ❌ **Ownership Keywords** - Not implemented
3. ❌ **Negative Types (~Copyable)** - Not implemented

**Compatibility Rating:** ⚠️ **Swift 5.x Compatible, Swift 6.x Incomplete**

The current implementation can generate valid Swift code but cannot take advantage of Swift 6.x's modern features for ownership, non-copyable types, and macro system.

## References

- [Swift Evolution - SE-0390: Noncopyable Types](https://github.com/apple/swift-evolution/blob/main/proposals/0390-noncopyable-structs-and-enums.md)
- [Swift Evolution - SE-0377: borrow and take parameter ownership modifiers](https://github.com/apple/swift-evolution/blob/main/proposals/0377-parameter-ownership-modifiers.md)
- [Swift Evolution - SE-0389: Attached Macros](https://github.com/apple/swift-evolution/blob/main/proposals/0389-attached-macros.md)
- [Swift 6.0 Release Notes](https://www.swift.org/blog/swift-6-released/)
