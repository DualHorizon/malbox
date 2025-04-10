![malbox banner](assets/banner-1.png)



<div align="center">

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/github/license/DualHorizon/malbox?style=for-the-badge)](LICENSE)
[![Coverage](https://codecov.io/gh/DualHorizon/malbox/branch/main/graph/badge.svg?token=123)](https://codecov.io/gh/DualHorizon/malbox)
[![Discord](https://img.shields.io/discord/YOUR_DISCORD_ID?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/your-invite)
[![Plugins](https://img.shields.io/badge/plugins-WIP-blue?style=for-the-badge)](https://marketplace.malbox.io)

[Documentation](docs) • [Installation](docs/installation.md) • [API Reference](docs/api) • [Plugin Marketplace](https://marketplace.mal.box) • [Discord](https://discord.gg/XWBdpQ5bMp)

</div>

---


> [!IMPORTANT]  
> Malbox is still in a very early stage of development, currently, the platform as is, isn't ready to be utilized.
> There are still a lot of rough edges, the code is for the most part not refactored/optimized, and all features described
> further-on may not be implemented yet (or only partially).

> The estimated release version to achieve something functional and stable is `v0.4.0`. 


## Overview

Malbox is a malware analysis platform/framework built in Rust. Its plugin-driven architecture enables security teams and malware analysis enthousiasts to extend and customize analysis capabilities easily. 


### Why Malbox?

- **Plugin Architecture**: Extend functionality easily through plugins, which can be written in Rust, Javascript and Python.
- **High Performance**: Malbox does not compromise on performance despite its modular plugin system. It primarily uses [iceoryx2](https://docs.rs/iceoryx2/latest/iceoryx2/), a shared-memory IPC (Inter-Process-Communication) library that enables zero-copy and lock-free communication. In addition, plugin creators and users can declare and configure plugin specifics, often resulting in more optimized runtimes and adaptable use cases.
- **Completely Free and Self-Hostable**: Retain full control over your infrastructure—Malbox will remain open-source and free forever.
- **User-friendly Ecosystem**: Malbox’s built-in marketplace makes it easy to install official and community verified plugins. Installation does not require rebuilding or restarting the Malbox service. Plugins and profiles follow strict standards to ensure a healthy, thriving ecosystem.
- **Cloud or On-Premise Storage and Deployment**: Malbox supports both cloud-based and on-premise solutions for your infrastructure and storage needs.
- **Easy Deployment**: Enjoy a user-friendly, minimal-overhead setup that is ready to use within minutes. Malbox emphasizes declarative configuration to reduce complexity and simplify the setup and configuration process.

# Plugin Ecosystem

At the core of Malbox is its extensible plugin system, designed for analysis flexibility while maintaining process isolation. Plugins operate with a well-defined lifecycle and communication framework that enables seamless integration of new capabilities, sharing data between plugins without any duplication, and much more.

## Architecture Overview

```mermaid
graph TD
    A[Core System] --> B[Plugin Manager]
    B --> C[Host Plugins]
    B --> D[Guest Plugins]
    B --> E[Hybrid Plugins]
    
    C --> F[IPC Channel]
    D --> G[gRPC Channel]
    E --> F
    E --> G
    
    F --> H[Plugin Registry]
    G --> H
    
    subgraph "Communication Bridge"
        F
        G
    end
    
    subgraph "Plugin Management"
        B
        H
    end
```

## Plugin Types by Execution Context

Malbox supports plugins in diverse execution environments:

- **Host Plugins**: Run directly on the host OS for static analysis, reporting, and task coordination
- **Guest Plugins**: Execute within VM environments for dynamic analysis
- **Hybrid Plugins**: Operate across both environments with coordinated components

## Execution Models

Plugins can operate in various modes:

- **Exclusive**: Plugin must be executed alone, no other plugins can run
- **Sequential**: Plugin must be executed sequentially, one at a time
- **Parallel**: Plugin can run in parallel with other plugins in the same group
- **Unrestricted**: Plugin has no special execution requirements

## State Management

Plugins can maintain different levels of persistence:

- **Stateless**: Fresh state for each task (default)
- **Stateful**: Maintains state between all tasks
- **StatefulByType**: Maintains state only across tasks of the same type

## Communication Infrastructure

The plugin system uses advanced IPC mechanisms:

- **Host Communication**: Zero-copy IPC using iceoryx2
- **Guest Communication**: gRPC for VM-based plugins
- **Bridge Component**: Translates between different communication protocols

## Plugin Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Created
    Created --> Starting: start()
    Starting --> Running
    Running --> Stopping: stop()
    Stopping --> Stopped
    Stopped --> Starting: start()
    Stopped --> [*]
    
    Starting --> Failed
    Running --> Failed
    Failed --> [*]
```

## Example Plugin Categories

- **Static Analysis Plugins**
  - File format analysis (PE/ELF/MachO)
  - YARA scanning
  - Signature verification
  - String extraction and analysis
  
- **Dynamic Analysis Plugins**
  - Process monitoring
  - Network traffic analysis
  - Memory inspection
  - Behavioral analysis
  
- **Infrastructure Plugins**
  - VM management
  - Network configuration
  - Artifact storage
  - Result aggregation

- **Utility Plugins**
  - Unpacking
  - Decryption
  - File type detection
  - Data visualization

Plugins are discoverable via metadata, which defines their capabilities, requirements, and compatibility with other plugins. This allows for creating comprehensive analysis profiles that combine multiple plugins for in-depth examination of artifacts.

> [!WARNING]  
> Plugin categories aren't defined yet, this is just a rough idea of what they could be. Stay tuned for updates!

### Plugin Marketplace

Access 50+ verified and official plugins from our [Marketplace](https://marketplace.mal.box) or at your self-hosted Malbox instance:

![Plugin Marketplace](https://github.com/user-attachments/assets/f0c2c099-1093-4d9c-a4d9-30adac8da4c9)

#### Official Plugins
[![PE Analysis](https://img.shields.io/badge/PE%20Analysis-1.2.0-blue?style=flat-square&logo=windows)](https://marketplace.malbox.io/plugins/pe-analysis)
[![Network Monitor](https://img.shields.io/badge/Network%20Monitor-2.0.1-blue?style=flat-square&logo=wireshark)](https://marketplace.malbox.io/plugins/network-monitor)
[![YARA Engine](https://img.shields.io/badge/YARA%20Engine-3.1.0-blue?style=flat-square&logo=search)](https://marketplace.malbox.io/plugins/yara-engine)
[![Memory Analysis](https://img.shields.io/badge/Memory%20Analysis-1.0.2-blue?style=flat-square&logo=memory)](https://marketplace.malbox.io/plugins/memory-analysis)

#### Featured Community Plugins
[![Threat Intel](https://img.shields.io/badge/Threat%20Intel-2.1.0-green?style=flat-square)](https://marketplace.malbox.io/plugins/threat-intel)
[![ML Classifier](https://img.shields.io/badge/ML%20Classifier-1.5.0-green?style=flat-square)](https://marketplace.malbox.io/plugins/ml-classifier)
[![Report Generator](https://img.shields.io/badge/Report%20Generator-2.2.1-green?style=flat-square)](https://marketplace.malbox.io/plugins/report-gen)

> [!IMPORTANT]  
> All plugins undergo security review and verification before being listed in the marketplace. [Submit your plugin](docs/plugins/publishing.md)

## Features

### Analysis Capabilities

Analysis capabilities depend on the plugins installed, hence, the capabilities will continue to grow as the project lives.
For good measure, you can find a couple of functionalities that are already available through our plugins.

- **File Type Support**
  - Windows Executables (PE32, PE32+)
  - Linux Executables (ELF)
  - macOS Executables (MachO)
  - Office Documents
  - PDF Files
  - Script Files (JS, VBS, PS1)
  - Archive Files

- **Analysis Features**
  - Automated unpacking
  - String extraction
  - Entropy analysis
  - Network simulation
  - Memory inspection
  - Behavioral analysis
  - Custom scripting support

![Analysis Result Popup](https://github.com/user-attachments/assets/1d25d9fc-291c-4cea-80bc-6c10e5ccff27)

### Enterprise Features

- Multi-user support with RBAC
- Team management
- API access and monitoring
- Custom reporting
- Integration capabilities

## Technology Stack

| Component | Technology | Details |
|-----------|------------|----------|
| Core | ![Rust](https://img.shields.io/badge/rust-1.81.0-orange.svg) | Safe, high-performance execution |
| IPC | ![iceoryx2](https://img.shields.io/badge/iceoryx2-latest-blue.svg) | Zero-copy plugin communication |
| Database | ![PostgreSQL](https://img.shields.io/badge/postgresql-13+-blue.svg) | Reliable state management |
| API | ![Axum](https://img.shields.io/badge/axum-latest-green.svg) | Modern web framework |
| Frontend | ![Astro](https://img.shields.io/badge/astro-latest-purple.svg) | Fast, static frontend |

## Architecture

![Runtime Architecture](assets/malbox-runtime.svg)

## Quick Start

### Prerequisites
- Rust 1.81.0+
- PostgreSQL 13+
- One of: KVM, VMware, or VirtualBox

```bash
# Install
git clone https://github.com/DualHorizon/malbox.git
cd malbox

# Configure
cp configuration/malbox.example.toml configuration/malbox.toml
$EDITOR configuration/malbox.toml

# Build and Run
cargo build --release
cargo run --release
```

Detailed setup instructions available in our [Installation Guide](docs/installation.md).

### Docker Support

```bash
# Pull official image
docker pull malbox/malbox:latest

# Start with docker-compose
wget https://raw.githubusercontent.com/DualHorizon/malbox/main/docker-compose.yml
docker-compose up -d
```

## Support & Community

- [Documentation](https://docs.malbox.io)
- [GitHub Issues](https://github.com/DualHorizon/malbox/issues)
- [Discord Community](https://discord.gg/your-invite)
- [Enterprise Support](https://malbox.io/enterprise)

## Contributing

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for development setup and guidelines.
Also, feel free to submit issues, Malbox's development is still in an early stage and contains a lot of rough edges!

## License

Licensed under GNU General Public License (GPL) - © 2024 Malbox Contributors

---

<div align="center">

**[⬆ Back to Top](#top)** • Made with ❤️ by the Malbox Team

<a href="https://star-history.com/#DualHorizon/malbox">
  <img src="https://api.star-history.com/svg?repos=DualHorizon/malbox&type=Date" alt="Star History Chart" />
</a>

</div>
