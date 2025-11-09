# TypeScript Feature TODO List

This document tracks TypeScript features that are not yet implemented in genco.

## Priority 1: Core Type System Enhancements

### Union and Intersection Types
- [ ] Implement union type builder: `ts::union_type(vec![type1, type2])`
- [ ] Implement intersection type builder: `ts::intersection_type(vec![type1, type2])`
- [ ] Support literal types: `ts::literal("success")`, `ts::literal(42)`
- [ ] Combine union with literals for common patterns

### Function Type Signatures
- [ ] Implement function parameter type: `ts::param("name", type_ref)`
- [ ] Implement function signature builder: `ts::function_signature(name, params, return_type)`
- [ ] Support arrow function types: `ts::arrow_function(params, return_type)`
- [ ] Support optional parameters in function signatures
- [ ] Support rest parameters: `...args: Type[]`

### Tuple Types
- [ ] Implement tuple type builder: `ts::tuple_type(vec![type1, type2])`
- [ ] Support optional tuple elements
- [ ] Support rest elements in tuples
- [ ] Support labeled tuple elements: `[name: string, age: number]`

## Priority 2: Generic System ✅ COMPLETE

### Generic Type Parameters
- [x] Add generics to interfaces: `Interface::with_generic_params(vec![GenericParam])`
- [x] Add generics to type aliases: `TypeAlias::with_generic_params(vec![GenericParam])`
- [x] Support generic constraints: `GenericParam::with_constraint(TypeRef)`
- [x] Support default generic parameters: `GenericParam::with_default(TypeRef)`
- [ ] Add generics to classes (if class builder is added)
- [ ] Support multiple constraints: `T extends A & B` (can use intersection types)

## Priority 3: Advanced Type Features

### Index Signatures ✅ COMPLETE
- [x] Add index signatures to interfaces: `Interface::with_index_signature(IndexSignature)`
- [x] Support string index signatures: `ts::index_signature_string(key, value_type)`
- [x] Support number index signatures: `ts::index_signature_number(key, value_type)`
- [x] Support symbol index signatures: `ts::index_signature_symbol(key, value_type)`

### Mapped Types
- [ ] Implement mapped type builder: `ts::mapped_type(key_type, value_type)`
- [ ] Support `keyof` operator
- [ ] Support `in` operator for mapped types
- [ ] Support mapped type modifiers: `+readonly`, `-?`, etc.

### Conditional Types
- [ ] Implement conditional type: `ts::conditional_type(check, extends, true_type, false_type)`
- [ ] Support `infer` keyword in conditional types
- [ ] Support distributive conditional types

### Template Literal Types
- [ ] Implement template literal type builder
- [ ] Support string literal type interpolation
- [ ] Support intrinsic string manipulation types

## Priority 4: Interface and Type Enhancements

### Interface Extensions ✅ COMPLETE
- [x] Add extends clause to interfaces: `Interface::with_extends(vec![parent_interface])`
- [x] Support single interface inheritance: `interface Foo extends Bar {}`
- [x] Support multiple interface inheritance: `interface Foo extends Bar, Baz {}`
- [ ] Support interface merging

### Method and Call Signatures ✅ CALL/CONSTRUCT COMPLETE
- [ ] Add method signatures to interfaces (separate from properties)
- [x] Add call signatures: `(x: number): string`
- [x] Add construct signatures: `new (x: number): Foo`
- [ ] Support method overloads in interfaces

### Type Assertions and Guards
- [ ] Implement type assertion helper: `ts::type_assertion(value, type)`
- [ ] Support `as` syntax
- [ ] Support angle bracket syntax: `<Type>value`
- [ ] Implement type guard helper: `ts::type_guard(param, type)`
- [ ] Support `is` predicate types

## Priority 5: Access Control and Modifiers

### Class-related Features (if class support is added)
- [ ] Implement access modifiers: `public`, `private`, `protected`
- [ ] Implement `readonly` modifier for class properties
- [ ] Implement `static` modifier
- [ ] Implement `abstract` classes and members
- [ ] Support parameter properties: `constructor(public name: string)`

### Non-null Assertions
- [ ] Implement non-null assertion operator: `value!`
- [ ] Support definite assignment assertion: `!`

## Priority 6: Advanced Module Features

### Export Enhancements
- [ ] Implement type-only exports: `ts::export_type(items)`
- [ ] Support `export type { Foo }`
- [ ] Support export aliasing: `export { Foo as Bar }`
- [ ] Support re-exports with types: `export type * from "module"`

### Import Enhancements
- [ ] Implement `import =` syntax for CommonJS compatibility
- [ ] Implement `export =` syntax for CommonJS compatibility
- [ ] Support dynamic imports (consider if needed for code generation)

### Module Declarations
- [ ] Implement namespace declarations: `ts::namespace(name, contents)`
- [ ] Implement module declarations: `ts::declare_module(name, contents)`
- [ ] Support ambient declarations
- [ ] Support `.d.ts` file generation patterns

## Priority 7: Utility and Helper Types

### Built-in Utility Types
- [ ] Implement `Partial<T>` helper
- [ ] Implement `Required<T>` helper
- [ ] Implement `Readonly<T>` helper
- [ ] Implement `Pick<T, K>` helper
- [ ] Implement `Omit<T, K>` helper
- [ ] Implement `Record<K, T>` helper
- [ ] Implement `Exclude<T, U>` helper
- [ ] Implement `Extract<T, U>` helper
- [ ] Implement `NonNullable<T>` helper
- [ ] Implement `ReturnType<T>` helper
- [ ] Implement `Parameters<T>` helper
- [ ] Implement `ConstructorParameters<T>` helper

### String Manipulation Types
- [ ] Implement `Uppercase<T>` helper
- [ ] Implement `Lowercase<T>` helper
- [ ] Implement `Capitalize<T>` helper
- [ ] Implement `Uncapitalize<T>` helper

## Priority 8: Advanced Enum Features

### Const Enums
- [ ] Support `const enum` declarations
- [ ] Support const enum inline behavior

### Enum Extensions
- [ ] Support computed enum members
- [ ] Support heterogeneous enums (mixed string/number)
- [ ] Support ambient enum declarations

## Implementation Notes

### High Priority Items (Priority 1-2)
These features provide the most value for day-to-day TypeScript code generation:
- Union types (very common: `string | number`)
- Function signatures (essential for API generation)
- Generic interfaces (common pattern: `interface Response<T>`)
- Generic constraints (type safety)

### Medium Priority Items (Priority 3-4)
These features enable more advanced TypeScript patterns:
- Mapped types (useful for transformations)
- Conditional types (advanced type manipulation)
- Interface extensions (common OOP pattern)
- Type guards (runtime type checking)

### Lower Priority Items (Priority 5-8)
These features are less critical or more specialized:
- Access modifiers (mainly for class-based code)
- Utility types (can be written manually as needed)
- Module declarations (mainly for .d.ts files)
- String manipulation types (advanced use cases)

## Implementation Strategy

1. **Start with Priority 1**: Focus on union types and function signatures first
2. **Add tests for each feature**: Follow the pattern in `tests/test_ts.rs`
3. **Update documentation**: Add examples to `TYPESCRIPT_FEATURES.md` as features are implemented
4. **Maintain backward compatibility**: Ensure existing code continues to work
5. **Follow existing patterns**: Use the same builder pattern as interfaces and enums

## Contribution Guidelines

When implementing features from this list:
1. Create comprehensive tests in `tests/test_ts.rs`
2. Add documentation with examples in `src/lang/ts.rs`
3. Update `TYPESCRIPT_FEATURES.md` to move features from "Not Implemented" to "Implemented"
4. Add usage examples to `examples/ts.rs` if applicable
5. Ensure all existing tests continue to pass

## Estimated Effort

- **Priority 1**: ~8-12 hours
- **Priority 2**: ~6-8 hours
- **Priority 3**: ~10-15 hours
- **Priority 4**: ~6-10 hours
- **Priority 5**: ~8-12 hours
- **Priority 6**: ~4-6 hours
- **Priority 7**: ~6-10 hours
- **Priority 8**: ~2-4 hours

**Total estimated effort**: 50-77 hours for complete implementation
