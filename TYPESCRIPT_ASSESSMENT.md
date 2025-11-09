# TypeScript Support Assessment for genco

## Executive Summary

The JavaScript builder in genco (`src/lang/js.rs`) **does not currently support TypeScript**. While the existing JavaScript implementation provides a solid foundation for code generation (imports, string quoting, template literals), it lacks TypeScript-specific features such as type annotations, interfaces, generics, and type-related syntax.

## Current JavaScript Builder Capabilities

### Implemented Features (`src/lang/js.rs:1-561`)

The JavaScript language implementation provides:

1. **Import System** - Comprehensive ES6 module import support
   - Named imports: `import {Foo} from "module"`
   - Default imports: `import Foo from "module"`
   - Wildcard imports: `import * as Foo from "module"`
   - Import aliasing: `import {Foo as Bar} from "module"`
   - Module path resolution (relative and global)
   - Automatic import grouping and deduplication

2. **String Quoting**
   - Double-quoted strings for static content: `"text"`
   - Template literals with interpolation: `` `Hello ${name}` ``
   - Proper JavaScript escape sequences (unicode, control characters)
   - Support for string interpolation via `${}` syntax

3. **Code Formatting**
   - Whitespace-aware formatting
   - Configurable module path resolution
   - Automatic import organization

4. **Configuration**
   - `Config` struct with module path settings
   - Relative path resolution for imports
   - File-level formatting with `format_file()`

### Example Usage

From `/home/user/genco/examples/js.rs:1-42`:

```rust
let react = &js::import("react", "React").into_default();
let display = &js::import("./Display", "Display").into_default();

let tokens = quote! {
    export default class App extends $react.Component {
        state = {
            total: null,
            next: null,
        };

        handleClick = buttonName => {
            this.setState($calculate(this.state, buttonName));
        };
    }
};
```

Outputs:
```javascript
import React from "react";
import Display from "./Display";

export default class App extends React.Component {
    // ... rest of code
}
```

## TypeScript Compatibility Gap Analysis

### Missing TypeScript Features

TypeScript extends JavaScript with static type checking. The following features are **NOT** supported by the current JavaScript builder:

#### 1. Type Annotations
```typescript
// Variable types
let name: string = "Alice";
const age: number = 30;
let isActive: boolean = true;

// Function signatures
function greet(name: string): string {
    return `Hello, ${name}`;
}

// Parameter types with defaults
function add(a: number, b: number = 0): number {
    return a + b;
}
```

#### 2. Interfaces and Type Definitions
```typescript
interface User {
    id: number;
    name: string;
    email?: string;  // Optional property
    readonly createdAt: Date;  // Readonly property
}

type UserID = string | number;
type Callback<T> = (value: T) => void;
```

#### 3. Generic Types
```typescript
function identity<T>(arg: T): T {
    return arg;
}

class Container<T> {
    private value: T;

    constructor(value: T) {
        this.value = value;
    }

    getValue(): T {
        return this.value;
    }
}
```

#### 4. Type Assertions and Guards
```typescript
// Type assertions
const value = someValue as string;
const value2 = <string>someValue;

// Type guards
function isString(value: unknown): value is string {
    return typeof value === 'string';
}
```

#### 5. Enums
```typescript
enum Direction {
    Up,
    Down,
    Left,
    Right
}

enum Color {
    Red = "#ff0000",
    Green = "#00ff00"
}
```

#### 6. Advanced Type Features
```typescript
// Union types
type Result = Success | Error;

// Intersection types
type Employee = Person & Worker;

// Mapped types
type Readonly<T> = {
    readonly [P in keyof T]: T[P];
};

// Conditional types
type NonNullable<T> = T extends null | undefined ? never : T;
```

#### 7. Decorators
```typescript
@Component({
    selector: 'app-root',
    template: '<div>Hello</div>'
})
class AppComponent {
    @Input() name: string;

    @Output() change = new EventEmitter();
}
```

#### 8. Namespace/Module Declarations
```typescript
namespace Utils {
    export function log(msg: string): void {
        console.log(msg);
    }
}

declare module 'external-lib' {
    export function doSomething(): void;
}
```

#### 9. Import/Export Type Extensions
```typescript
// Type-only imports
import type { User } from './types';
import { type Config, API } from './api';

// Type-only exports
export type { User };
export { type Config };
```

#### 10. TSX Support
```tsx
interface Props {
    name: string;
    age: number;
}

const Component: React.FC<Props> = ({ name, age }) => {
    return <div>{name} is {age} years old</div>;
};
```

## Similarities with JavaScript

TypeScript is a **superset** of JavaScript, meaning:

1. **All JavaScript code is valid TypeScript** (with `allowJs` enabled)
2. **Import/export syntax is identical** for runtime code
3. **String quoting and escaping rules are the same**
4. **Template literals work identically**
5. **Comments use the same syntax** (`//` and `/* */`)

This means the current JavaScript builder can generate syntactically valid TypeScript code **as long as no type annotations are needed**.

## Implementation Proposal

### Option 1: Extend JavaScript Builder (Recommended)

Create a TypeScript language implementation that extends or wraps the JavaScript implementation.

**Pros:**
- Reuses existing import system
- Minimal code duplication
- Easy to maintain compatibility

**Cons:**
- May require refactoring to share code properly
- Type annotation API needs careful design

### Option 2: Create Standalone TypeScript Builder

Implement TypeScript as a completely separate language module.

**Pros:**
- Clean separation of concerns
- No risk of breaking JavaScript functionality
- More flexibility for TypeScript-specific features

**Cons:**
- Code duplication for imports and string handling
- More maintenance burden

### Recommended Approach: Option 1 with Shared Foundation

## Implementation Steps

### Phase 1: Core TypeScript Module (Minimal Viable Product)

**File:** `src/lang/ts.rs`

1. **Create basic TypeScript language structure**
   - Define `TypeScript` struct implementing `Lang` trait
   - Define `Tokens` type alias: `pub type Tokens = crate::Tokens<TypeScript>`
   - Implement `Config` struct (can reuse JavaScript's config initially)
   - Implement `Format` struct

2. **Reuse JavaScript string handling**
   - Copy or delegate to JavaScript's `write_quoted()` implementation
   - Copy or delegate template literal handling (`open_quote`, `close_quote`)
   - Copy string eval methods (`start_string_eval`, `end_string_eval`)

3. **Adapt JavaScript import system**
   - Copy `Import` struct with same functionality
   - Support named, default, and wildcard imports
   - Maintain module path resolution

4. **Add type-only import support**
   - Extend `Import` to support `type` modifier
   - Implement: `import type { User } from './types'`
   - Implement: `import { type Config, API } from './api'`

**Estimated effort:** 2-4 hours

### Phase 2: Type Annotation Helpers

5. **Create type annotation builders**
   ```rust
   pub fn type_annotation(name: impl Into<ItemStr>) -> TypeAnnotation
   pub fn optional_type(inner: TypeAnnotation) -> TypeAnnotation
   pub fn union_type(types: Vec<TypeAnnotation>) -> TypeAnnotation
   pub fn generic_type(base: impl Into<ItemStr>, params: Vec<TypeAnnotation>) -> TypeAnnotation
   ```

6. **Function signature support**
   ```rust
   pub struct FunctionParam {
       name: ItemStr,
       type_annotation: Option<TypeAnnotation>,
       optional: bool,
       default_value: Option<Tokens>,
   }

   pub fn function_signature(
       name: impl Into<ItemStr>,
       params: Vec<FunctionParam>,
       return_type: Option<TypeAnnotation>
   ) -> FunctionSignature
   ```

**Estimated effort:** 4-6 hours

### Phase 3: Interface and Type Definitions

7. **Interface builder**
   ```rust
   pub struct InterfaceProperty {
       name: ItemStr,
       type_annotation: TypeAnnotation,
       optional: bool,
       readonly: bool,
   }

   pub fn interface(
       name: impl Into<ItemStr>,
       properties: Vec<InterfaceProperty>
   ) -> Interface
   ```

8. **Type alias builder**
   ```rust
   pub fn type_alias(
       name: impl Into<ItemStr>,
       definition: TypeAnnotation
   ) -> TypeAlias
   ```

**Estimated effort:** 3-5 hours

### Phase 4: Advanced Features

9. **Generic type support**
   - Generic function declarations
   - Generic class declarations
   - Generic constraints (`T extends SomeType`)

10. **Enum support**
    ```rust
    pub fn enum_type(
        name: impl Into<ItemStr>,
        variants: Vec<(ItemStr, Option<TokenStream>)>
    ) -> Enum
    ```

11. **Decorator support**
    ```rust
    pub fn decorator(
        name: impl Into<ItemStr>,
        args: Option<Tokens>
    ) -> Decorator
    ```

**Estimated effort:** 6-8 hours

### Phase 5: Testing and Examples

12. **Create comprehensive tests** in `tests/`
    - Test type annotations
    - Test interface generation
    - Test generic types
    - Test import/export with types
    - Test string interpolation (should work like JS)

13. **Create example** in `examples/ts.rs`
    - Demonstrate React component with TypeScript
    - Show interfaces, type aliases
    - Show generic functions
    - Show module imports with types

14. **Update documentation**
    - Add TypeScript to README.md
    - Document TypeScript-specific API
    - Add rustdoc examples to `src/lang/ts.rs`

**Estimated effort:** 4-6 hours

### Phase 6: Integration

15. **Export TypeScript in module system**
    - Add `pub mod ts;` to `src/lang/mod.rs`
    - Add `pub use self::ts::TypeScript;` to exports
    - Add to prelude in `src/prelude.rs`

16. **Add to CI/build system**
    - Ensure tests run for TypeScript
    - Add to documentation generation

**Estimated effort:** 1-2 hours

## Total Estimated Effort

- **Minimal viable implementation (Phases 1-2):** 6-10 hours
- **Complete implementation (Phases 1-6):** 20-30 hours

## File Structure

```
src/lang/
├── ts.rs              # Main TypeScript implementation (or ts/ directory)
├── ts/
│   ├── mod.rs         # Core TypeScript Lang implementation
│   ├── import.rs      # Import/export with type support
│   ├── types.rs       # Type annotation builders
│   ├── interface.rs   # Interface and type alias builders
│   ├── generics.rs    # Generic type support
│   ├── enum.rs        # Enum builder
│   └── decorator.rs   # Decorator support

examples/
└── ts.rs              # TypeScript usage example

tests/
├── test_ts_types.rs   # Type annotation tests
├── test_ts_imports.rs # Import/export tests
└── test_ts_string.rs  # String handling tests (should match JS)
```

## API Design Sketch

```rust
// Basic usage example
use genco::prelude::*;

// Define imports with type-only support
let user_type = ts::import("./types", "User").into_type_only();
let api = ts::import("./api", "API");

// Create interface
let user_interface = ts::interface("User", vec![
    ts::property("id", ts::type_ref("number")),
    ts::property("name", ts::type_ref("string")),
    ts::optional_property("email", ts::type_ref("string")),
]);

// Create function with type annotations
let tokens = quote! {
    $(user_interface)

    function getUser(id: number): Promise<$user_type> {
        return $api.fetchUser(id);
    }
};

// Output:
// import type { User } from "./types";
// import { API } from "./api";
//
// interface User {
//     id: number;
//     name: string;
//     email?: string;
// }
//
// function getUser(id: number): Promise<User> {
//     return API.fetchUser(id);
// }
```

## Alternative: Type Annotation via Raw Strings

For a simpler initial implementation, type annotations could be added as raw strings:

```rust
let tokens = quote! {
    function greet(name: string): string {
        return $(quoted(format!("Hello, {}", "name")));
    }
};
```

This approach:
- **Pros:** Simple, no API changes needed, works immediately
- **Cons:** No type safety, no validation, harder to compose programmatically

## Compatibility Considerations

1. **JavaScript Compatibility:** TypeScript code should be able to degrade gracefully to JavaScript by stripping type annotations
2. **Version Targeting:** Consider supporting different TypeScript compiler targets (ES5, ES6, ESNext)
3. **JSX vs TSX:** TSX support could reuse JSX logic from JavaScript with type overlays

## References

- **TypeScript Handbook:** https://www.typescriptlang.org/docs/handbook/intro.html
- **TypeScript AST:** https://astexplorer.net/ (select TypeScript parser)
- **Current JavaScript implementation:** `src/lang/js.rs:1-561`
- **Dart reference implementation:** `src/lang/dart/mod.rs:1-292` (similar string interpolation)
- **Lang trait definition:** `src/lang/mod.rs:48-132`

## Conclusion

TypeScript support is **not present** in the current JavaScript builder but is **highly feasible** to implement. The JavaScript implementation provides an excellent foundation, and TypeScript-specific features can be added incrementally. The recommended approach is to create a new `ts` module that reuses JavaScript's string handling and imports while adding type annotation capabilities.

**Recommendation:** Start with Phase 1 (basic module structure with reused JS functionality) to validate the approach, then incrementally add type annotation features based on user needs.
