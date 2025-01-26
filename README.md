![Banner](assets/malbox-banner-blue.png)

<div align="center">

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/github/license/DualHorizon/malbox?style=for-the-badge)](LICENSE)
[![Release](https://img.shields.io/github/v/release/DualHorizon/malbox?style=for-the-badge)](https://github.com/DualHorizon/malbox/releases)
[![Build](https://img.shields.io/github/actions/workflow/status//malbox/rust.yml?style=for-the-badge)](https://github.com/DualHorizon/malbox/actions)
[![Coverage](https://codecov.io/gh/DualHorizon/malbox/branch/main/graph/badge.svg?token=123)](https://codecov.io/gh/DualHorizon/malbox)
[![Discord](https://img.shields.io/discord/YOUR_DISCORD_ID?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/your-invite)
[![Plugins](https://img.shields.io/badge/plugins-50%2B-blue?style=for-the-badge)](https://marketplace.malbox.io)

[Documentation](docs) • [Installation](docs/installation.md) • [API Reference](docs/api) • [Plugin Marketplace](https://marketplace.mal.box) • [Discord](https://discord.gg/XWBdpQ5bMp)

</div>

---

## Overview

Malbox is an enterprise-grade malware analysis platform built in Rust. Its plugin-driven architecture enables security teams and malware analysis enthousiasts to extend and customize analysis capabilities while maintaining high performance and stability.

![Dashboard](assets/malbox-panel-showcase.png)

### Why Malbox?

- **Plugin Architecture**: Extend functionality through plugins, which can be written in Rust, Javascript and Python.
- **High Performance**: Malbox is using [iceoryx2](https://docs.rs/iceoryx2/latest/iceoryx2/), a shared memory IPC (Inter-Process-Communication) library, allowing zero-copy and lock-free inter-process communication.
- **Completely Free and Self-Hostable**: Complete control over your infrastructure
- **Large Ecosystem**: Thanks to Malbox's built-in marketplace, you can easily install and go through official and verified plugins, not rebuild or restart required, hot-reloading all the way!
- **Cloud or On-Premise**: Malbox supports cloud providers and on-premise for machinery and storage.
- **Easy Deployment**: User-friendly and no-overhead setup of the platform, ready to use within a few minutes.

## Plugin Ecosystem

At the core of Malbox is its extensible plugin system, powered by high-performance IPC using iceoryx2. Plugins maintain process isolation while enabling seamless integration of new capabilities.
Each plugin has metadata, and can be qualified for specific categories, plugins can be grouped together in different analysis profiles, which are also available through the marketplace.

```mermaid
graph TD
    A[Core System] --> B[Plugin Manager]
    B --> C[Analysis Plugins]
    B --> D[Storage Plugins]
    B --> E[Report Plugins]
    B --> F[Infrastructure Plugins]
    
    subgraph "Plugin Types"
        C
        D
        E
        F
    end
```

### Plugin Types

- **Static Analysis**
  - PE/ELF/MachO analysis
  - YARA pattern matching
  - String and entropy analysis
  - Digital signature verification
  - Office document analysis
  - PDF analysis
  
- **Dynamic Analysis**
  - Process monitoring
  - Network traffic analysis
  - Memory analysis
  - Registry monitoring
  - Behavioral tracking
  - Anti-VM detection mitigation

- **Unpacking**

> [!WARNING]  
> Plugin categories aren't defined yet, this is just a rough idea of what they could be. Stay tuned for updates!

### Plugin Marketplace

Access 50+ verified and official plugins from our [Marketplace](https://marketplace.malbox.io) or at your self-hosted Malbox instance:

![Plugin Marketplace](https://github.com/user-attachments/assets/56ea97a7-4e84-4cba-a02a-17932a27c8a6)


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

![Analysis Result](https://github.com/user-attachments/assets/a0d7d3d5-c4b0-4079-8264-8fc617205ae8)


### Enterprise Features

- Multi-user support with RBAC
- Team management
- API access and monitoring
- Custom reporting
- Integration capabilities

## Technology Stack

<p align="center">
  <img src="https://github-readme-tech-stack.vercel.app/api/cards?title=Malbox+Core+Technologies&align=center&titleAlign=center&lineCount=2&line1=rust%2CRust+Core%2C6b1919%3Btokio%2CAsync+Runtime%2C7c5af5%3Bpostgresql%2CDatabase%2C346891%3B&line2=iceoryx2%2CIPC%2C844FBA%3Baxum%2CAPI%2CF25F2E%3Bastro%2CFrontend%2C61DAFB%3B" alt="Tech Stack" />
</p>

| Component | Technology | Details |
|-----------|------------|----------|
| Core | ![Rust](https://img.shields.io/badge/rust-1.81.0-orange.svg) | Safe, high-performance execution |
| IPC | ![iceoryx2](https://img.shields.io/badge/iceoryx2-latest-blue.svg) | Zero-copy plugin communication |
| Database | ![PostgreSQL](https://img.shields.io/badge/postgresql-13+-blue.svg) | Reliable state management |
| API | ![Axum](https://img.shields.io/badge/axum-latest-green.svg) | Modern web framework |
| Frontend | ![Astro](https://img.shields.io/badge/astro-latest-purple.svg) | Fast, static frontend |

## Performance

### Analysis Metrics

| Operation | Performance | Notes |
|-----------|-------------|--------|
| Static Analysis | 2-5 seconds | PE files up to 10MB |
| Dynamic Analysis | 45-60 seconds | Full system monitoring |
| Concurrent Analyses | 50+ | With recommended hardware |
| Memory Usage | 512MB base | +256MB per analysis |
| Storage Required | 20GB+ | Scales with retention policy |

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
