# Wickra Proof — C / C++ examples

The Wickra Proof C ABI is a single shared/static library plus a generated header
([`bindings/c/include/wickra_proof.h`](../../bindings/c/include/wickra_proof.h)). Any C-capable
language links against the same artifact; these examples show the plain-C path
and, through [`wickra_proof.hpp`](../../bindings/c/include/wickra_proof.hpp), the C++ one.

## Build the library

From the workspace root:

```sh
cargo build -p wickra-proof-c --release
```

This produces, in `target/release/`:

| Platform | Shared library | Link target |
|----------|----------------|-------------|
| Linux    | `libwickra_proof.so`     | `-lwickra_proof` |
| macOS    | `libwickra_proof.dylib`  | `-lwickra_proof` |
| Windows (MSVC) | `wickra_proof.dll` | `wickra_proof.dll.lib` (import lib) |

A static library (`libwickra_proof.a` / `wickra_proof.lib`) is emitted alongside.

## Build and run the examples

### With CMake (portable, used by CI)

```sh
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

### Directly with a compiler

```sh
# Linux / macOS
cc examples/c/prove.c -I bindings/c/include -L target/release -lwickra_proof -lm -o prove
LD_LIBRARY_PATH=target/release ./prove        # macOS: DYLD_LIBRARY_PATH

# Windows (MinGW gcc, linking the DLL directly)
gcc examples/c/prove.c -I bindings/c/include target/release/wickra_proof.dll -lm -o prove.exe
```

## The examples

| Example | What it does |
|---------|--------------|
| `prove.c` | A minimal C example: prove a (spec, data) pair through the wickra-proof C ABI, |
| `prove.cpp` | A minimal C++ example: prove a (spec, data) pair through the wickra-proof C ABI, print the report hash, then verify the proof and assert it holds. |

## Usage shape

Every call follows the same handle discipline: construct from a spec JSON, drive
with command JSON, read the response, free the handle exactly once. `wickra_proof.h` is
the whole contract; the C++ header, where one ships, wraps the handle in a
move-only RAII type. See [`bindings/c/README.md`](../../bindings/c/README.md).
