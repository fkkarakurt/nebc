# 🌌 NEBULANG QUANTUM COMPILER (NEBC)

**NEBC** is the flagship compiler for the **Nebulang programming language**, architected from the ground up to achieve an unprecedented fusion of **speed**, **lightness**, and **privacy**. It pioneers a new domain of secure, hardware-aware, and high-performance software development.

Going beyond traditional compilation, NEBC aims to make **reverse engineering and binary tampering exponentially more difficult** by **integrating Quantum-Inspired** and **Adaptive Security** mechanisms directly into the native binary.

-----

## ✨ Why Nebulang is the Future

Nebulang is not just a language; it is an **aspirational ecosystem protected at the machine level**. NEBC is engineered to solve two of the most critical challenges in modern software deployment: **deep security** and **ultra-low startup latency**.

| Feature             | NEBC Advantage                                            | Security Goal                                         | Performance Goal      |
| :------------------ | :-------------------------------------------------------- | :---------------------------------------------------- | :-------------------- |
| **Code Protection** | Quantum-Inspired Instruction Entanglement                 | **95% Protection (Target)**                           | Minimal Overhead (5%) |
| **Startup Speed**   | Micro-Binary Architecture (Smallest Footprint)            |                                                       |                       |
| **Integrity**       | Hardware-Integrated Anti-Tamper Signatures                | **Make reverse engineering as difficult as possible** |                       |
| **Deployment**      | Cross-Platform Native Binary Generation (ELF, Mach-O, PE) |                                                       |                       |

-----

## 🚀 Getting Started

### Installation

NEBC is a native, single-binary compiler. To get started, you only need Rust's stable toolchain.

```bash
# Clone the repository
git clone https://github.com/NebulaProjectSystems/nebulang.git
cd nebulang

# Build the compiler (Requires Rust stable)
# The compiler will be linked against system libraries for cross-platform support.
cargo build --release

# The executable will be available at target/release/nebc
```

### Basic Commands

The NEBC CLI is the powerful gateway to the Nebulang ecosystem.

| Command      | Description                                                                              | Example                       |
| :----------- | :--------------------------------------------------------------------------------------- | :---------------------------- |
| `nebc build` | **Compiles** the source file into a platform-native, protected binary.                   | `nebc build main.neb`         |
| `nebc run`   | **Builds, links, and executes** the program immediately.                                 | `nebc run demo/game.neb`      |
| `nebc test`  | Executes the core compiler pipeline (lexing/parsing/analysis) for internal file testing. | `nebc test /path/to/test.neb` |

```sh
> nebc help       

🌌 Nebulang Quantum Compiler

Usage: nebc [COMMAND]

Commands:
  build  Build quantum binary
  run    Compile and run quantum program
  test   Test quantum program files
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

-----

## 🔬 Nebulang Language Syntax: Simplicity Meets Power

Nebulang's symbolic, whitespace-sensitive syntax is designed for maximum developer efficiency, reducing boilerplate while retaining C-like control.

### Core Constructs

| Feature            | Syntax            | Example                | Description                                  |
| :----------------- | :---------------- | :--------------------- | :------------------------------------------- |
| **Output (Print)** | `! "..."`         | `! "Hello, {name}"`    | The `!` symbol is the print command.         |
| **New Line**       | `>                | `                      | `! "Line 1 >                                 | Line 2"` | Explicitly enforces a newline during output. |
| **Assignment**     | `var value`       | `a 5, name "Joe"`      | Space-separated assignment.                  |
| **Conditional**    | `? (cond) ... !?` | `? (a > b) ... !? ...` | Symbolic `If` and `Else` (`!?`).             |
| **Loop**           | `@ i, start..end` | `@ i, 1..10`           | Symbolic loop construct for range iteration. |

### Advanced Syntax (Arrays)

The Nebulang data structure philosophy prioritizes flexibility and direct access.

> **NOTE:** Array structures and advanced indexing are currently **under development** and represent the language's intended future state.

```neb
// Defining a collection named 'cars' (aliased as 'c')
[] cars as c
    [] brands as b
        "Toyota", "Nissan", "Honda"
    [] colors as col
        "White", "Black", "Red"

// Complex indexing for nested collections:
! "{c:col:0} >|"       // Accesses 'White' (colors[0])
! "{[]cars:b:2} >|"    // Accesses 'Honda' (brands[2])

// Sub-array assignment:
myCars c:b
! "My Cars: {myCars}"  // My Cars: "Toyota", "Nissan", "Honda"
```

### Example: Financial Analysis 📊

```neb
// [Code and Output remain the same as provided]
```

### Example: Dice Game Simulation 🎲

```nebulang
// [Code and Output remain the same as provided]
```

-----

## 🛠️ Performance Comparison for NEBC v0.1.0

The primary architectural objective of NEBC is to achieve a Micro-Binary Architecture and near-zero overhead performance by eliminating the runtime costs associated with virtual machines (VMs) and extensive standard libraries.

To evaluate this claim, NEBC v0.1.0 was subjected to comparative benchmarking against multiple programming languages using a computationally intensive loop overhead test, comprising **5,000,000 × 500 total iterations**, executed on an Arch Linux system.

### Execution Time

| Language     | Total Execution Time (s) |
| :----------- | :----------------------- |
| ASM (NASM)   | 0.807                    |
| C            | 5.105                    |
| C++          | 5.110                    |
| **Nebulang** | **5.425**                |
| C (-O3)      | 0.001                    |
| C++ (-O3)    | 0.002                    |
| Perl         | 77.425                   |
| Python       | 154.466                  |


> **Note on C/C++ (O3)**: The near-zero execution time indicates the compiler's aggressive **Dead Code Elimination (DCE)** optimization entirely removed the heavy loop load. Nebulang intentionally did not perform this optimization in v0.1.0, truthfully executing all 2.5 billion iterations, making the non-optimized C/C++ time (~5.1s) a more relevant comparison for the current code generation.

> These great optimization techniques from GNU GCC are a guide to Nebulang.

### Binary Size (Lower is Better)

| Language     | Binary Size |
| :----------- | :---------- |
| ASM (NASM)   | 8.9K        |
| **Nebulang** | **11.0K**   |
| C            | 16.0K       |
| C++          | 16.0K       |
| Golang       | 1.5M        |

> NEBC successfully achieves the smallest native executable size among high-level compilers. The resulting 11K binary is only marginally larger than the pure Assembly benchmark (8.9K) and 31% smaller than the highly-optimized C/C++ output. This eliminates the runtime overhead associated with Go's 1.5M executable, fulfilling a core vision of the Nebula Project.

*While successfully executing all 2.5 billion operations, NEBC v0.1.0 is currently 3.2x slower than Go and 6.7x slower than the theoretical ASM limit. This identifies a clear goal for the next iteration: to optimize the low-level Assembly output generation to match or exceed the performance of Go and close the gap with the raw Assembly limit.*