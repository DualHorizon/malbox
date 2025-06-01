![malbox banner](assets/banner-1.png)



<div align="center">

[![Rust](https://img.shields.io/badge/Built%20with%20Rust-grey?style=for-the-badge&logo=rust&color=%23282828)](https://www.rust-lang.org/)
[![License](https://img.shields.io/github/license/DualHorizon/malbox?style=for-the-badge&color=%23282828)](LICENSE)
[![Discord](https://img.shields.io/badge/Discord-grey?style=for-the-badge&logo=discord&color=%23282828)](https://discord.gg/7BVnQHRy7h)
[![Plugins](https://img.shields.io/badge/plugins-WIP-blue?style=for-the-badge&color=%23282828)](#)

[Documentation](https://dualhorizon.github.io/malbox-docs/) • [Installation](https://dualhorizon.github.io/malbox-docs/getting-started/quickstart/) • [API Reference](https://dualhorizon.github.io/malbox-docs/reference/api/) • [Plugin Marketplace](#) • [Discord](https://discord.gg/7BVnQHRy7h)

</div>

---


> [!IMPORTANT]  
> Malbox is still in a very early stage of development, currently, the platform as is, isn't ready to be utilized.
> There are still a lot of rough edges, the code is for the most part not refactored/optimized, and all features described
> further-on may not be implemented yet (or only partially).

> The estimated release version to achieve something functional and stable is `v0.4.0`. 


# Overview

Malbox is an open-source malware analysis platform designed to provide security researchers, malware analysts, and cybersecurity teams with a powerful, extensible environment for analyzing files and understanding their behavior. 

## Why Malbox?

- **Plugin Architecture**: Extend functionality easily through plugins, which can be written in Rust, Javascript and Python. Malbox's plugin system lets you extend functionality as easily as installing a package, and discover them through the Malbox marketplace.
- **High Performance**: Malbox does not compromise on performance despite its modular plugin system. It primarily uses [iceoryx2](https://docs.rs/iceoryx2/latest/iceoryx2/), a shared-memory IPC (Inter-Process-Communication) library that enables zero-copy and lock-free communication. In addition, plugin creators and users can declare and configure plugin specifics, often resulting in more optimized runtimes and adaptable use cases. 
- **Completely Free and Self-Hostable**: Retain full control over your infrastructure—Malbox will remain open-source and free forever.
- **User-friendly Ecosystem**: Malbox’s built-in marketplace makes it easy to install official and community verified plugins. Installation does not require rebuilding or restarting the Malbox service. Plugins and profiles follow strict standards to ensure a healthy, thriving ecosystem.
- **Cloud or On-Premise Storage and Deployment**: Malbox supports both cloud-based and on-premise solutions for your infrastructure and storage needs.
- **Easy Setup**: Enjoy a user-friendly, minimal-overhead setup that is ready to use within minutes. Malbox emphasizes declarative configuration to reduce complexity and simplify the setup and configuration process.

# Plugin Ecosystem

At the core of Malbox is its extensible plugin system, designed for analysis flexibility while maintaining process isolation. Plugins operate with a well-defined lifecycle and communication framework that enables seamless integration of new capabilities, sharing data between plugins without any duplication, and much more.

Plugins are discoverable via metadata, which defines their capabilities, requirements, and compatibility with other plugins. This allows for creating comprehensive analysis profiles that combine multiple plugins for in-depth examination of artifacts.

## Plugin Marketplace

Access community verified or official plugins through our [Marketplace](#) - also available in your self-hosted Malbox instance:

![Plugin Marketplace](https://github.com/user-attachments/assets/f0c2c099-1093-4d9c-a4d9-30adac8da4c9)

#### Official Plugins
[![PE Analysis](https://img.shields.io/badge/PE%20Analysis-1.2.0-blue?style=flat-square&logo=windows)](#)
[![Network Monitor](https://img.shields.io/badge/Network%20Monitor-2.0.1-blue?style=flat-square&logo=wireshark)](#)
[![YARA Engine](https://img.shields.io/badge/YARA%20Engine-3.1.0-blue?style=flat-square&logo=search)](#)
[![Memory Analysis](https://img.shields.io/badge/Memory%20Analysis-1.0.2-blue?style=flat-square&logo=memory)](#)

#### Featured Community Plugins
[![Threat Intel](https://img.shields.io/badge/Threat%20Intel-2.1.0-green?style=flat-square)](#)
[![ML Classifier](https://img.shields.io/badge/ML%20Classifier-1.5.0-green?style=flat-square)](#)
[![Malcat Scripting](https://img.shields.io/badge/Malcat%20Scripting-2.2.1-green?style=flat-square)](#)

> [!IMPORTANT]  
> All plugins undergo security review and verification before being listed in the marketplace. [Submit your plugin](#)

## Features

### Analysis Capabilities

Analysis capabilities depend on the plugins installed, hence, the capabilities will continue to grow as plugins are released, both from the community and maintainers.
For good measure, you can find a couple of functionalities that will be available through official plugins.

- **File Type Support**
  - Windows Executables (PE32, PE32+)
  - Linux Executables (ELF)
  - Office Documents
  - PDF Files
  - Script Files (JS, VBS, PS1)
  - Archive Files

- **Analysis Features**
  - Automated unpacking
  - String extraction
  - Entropy analysis
  - Network analysis
  - Memory inspection
  - Behavioral analysis
  - Custom scripting support

![Analysis Result Popup](https://github.com/user-attachments/assets/1d25d9fc-291c-4cea-80bc-6c10e5ccff27)

### [![TBD](https://img.shields.io/badge/TBD-red?style=flat-square)](#) Enterprise Features

- Multi-user support with RBAC
- Team management
- API access and monitoring
- Custom reporting
- Integration capabilities

## Support & Community

- [Documentation](#)
- [GitHub Issues](https://github.com/DualHorizon/malbox/issues)
- [Discord Community](https://discord.gg/XWBdpQ5bMp)

## Contributing

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for development setup and guidelines.
Also, feel free to submit issues, Malbox's development is still in an early stage and contains a lot of rough edges!

## License

Licensed under GNU General Public License (GPL) - © 2024 Malbox Contributors

---

<div align="center">

**[⬆ Back to Top](#top)** • Made with ❤️ by the Malbox maintainers and its contributors

<a href="https://star-history.com/#DualHorizon/malbox">
  <img src="https://api.star-history.com/svg?repos=DualHorizon/malbox&type=Date" alt="Star History Chart" />
</a>

</div>
