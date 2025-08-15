# Universal NFT Program - Source Code Structure

This directory contains the Solana program source code for the Universal NFT project, organized in a modular structure for better maintainability and scalability.

## Directory Structure

```
src/
├── lib.rs                 # Main program entry point and instruction dispatcher
├── constants/             # Program constants and configuration
│   ├── mod.rs            # Constants module exports
│   ├── seeds.rs          # PDA seeds for account derivation
│   └── limits.rs         # Program limits and constraints
├── error/                 # Custom error definitions
│   ├── mod.rs            # Error module exports
│   ├── universal_nft.rs  # Main program errors
│   └── validation.rs     # Input validation errors
├── state/                 # On-chain data structures
│   ├── mod.rs            # State module exports
│   ├── program_state.rs  # Global program state
│   ├── nft_origin.rs     # NFT origin tracking
│   ├── replay_protection.rs # Cross-chain message replay protection
│   ├── collection.rs     # Collection management
│   └── cross_chain_message.rs # Cross-chain communication
├── instructions/          # Instruction handlers
│   ├── mod.rs            # Instructions module exports
│   ├── initialize_program_state.rs # Program initialization
│   ├── create_nft_origin.rs # NFT origin creation
│   ├── create_replay_protection.rs # Replay protection setup
│   ├── mint_nft.rs       # NFT minting
│   ├── burn_nft.rs       # NFT burning
│   ├── setup_collection.rs # Collection setup
│   └── validate_message.rs # Cross-chain message validation
└── utils/                 # Utility functions
    ├── mod.rs            # Utils module exports
    ├── pda.rs            # PDA derivation helpers
    ├── validation.rs     # Input validation utilities
    └── crypto.rs         # Cryptographic utilities
```

## Module Overview

### Constants (`constants/`)

- **seeds.rs**: PDA seeds for deterministic account addresses
- **limits.rs**: Program constraints and maximum values

### Errors (`error/`)

- **universal_nft.rs**: Core program error types
- **validation.rs**: Input validation error types

### State (`state/`)

- **program_state.rs**: Global program configuration and counters
- **nft_origin.rs**: NFT origin tracking across chains
- **replay_protection.rs**: Cross-chain message replay prevention
- **collection.rs**: Collection metadata and verification
- **cross_chain_message.rs**: Cross-chain communication data

### Instructions (`instructions/`)

- **initialize_program_state.rs**: Program setup and initialization
- **create_nft_origin.rs**: NFT origin account creation
- **create_replay_protection.rs**: Replay protection setup
- **mint_nft.rs**: NFT minting logic
- **burn_nft.rs**: NFT burning logic
- **setup_collection.rs**: Collection configuration
- **validate_message.rs**: Cross-chain message validation

### Utils (`utils/`)

- **pda.rs**: Program Derived Address derivation helpers
- **validation.rs**: Input validation and sanitization
- **crypto.rs**: Cryptographic operations and message hashing

## Design Principles

1. **Modularity**: Each major feature is separated into its own module
2. **Separation of Concerns**: State, logic, and utilities are clearly separated
3. **Extensibility**: New features can be easily added as new modules
4. **Maintainability**: Clear structure makes code easier to understand and modify
5. **Reusability**: Utility functions can be shared across different instructions

## Adding New Features

To add a new feature:

1. **State**: Add new state structures in `state/` module
2. **Instructions**: Create instruction handlers in `instructions/` module
3. **Errors**: Define custom errors in `error/` module
4. **Constants**: Add configuration constants in `constants/` module
5. **Utils**: Create helper functions in `utils/` module
6. **Integration**: Wire up new instructions in `lib.rs`

## Current Status

- ✅ Program state initialization
- ✅ Basic structure and organization
- 🔄 NFT origin PDA (Task 3 - in progress)
- 🔄 Replay protection PDA (Task 4 - planned)
- 🔄 NFT minting (Task 5 - planned)
- 🔄 Cross-chain integration (Task 9 - planned)
