![Banner 29](assets/malbox-banner-blue.png)

# Malbox - Malware in a Box
Malbox is an advanced sandboxing solution designed for static and dynamic malware analysis. 
The project aims to provide a comprehensive, self-hosted, fast, and user-friendly platform for studying and analyzing various types of malware. 
Malbox is fully open-source, modular, and community-oriented, making it an excellent resource for cybersecurity researchers, analysts, and developers interested in understanding the behavior of malicious software.

## Features
- Modern Analysis: Quickly analyze malware samples with efficient and modern techniques.
- Static and Dynamic Analysis: Supports both static (file-based) and dynamic (runtime) analysis of malware.
- Modular Architecture: Easily extend and customize the platform with additional modules and plugins.
- User-Friendly Interface: Interact with the platform using a straightforward, intuitive interface.
- Community-Oriented: Collaborate with other researchers and contribute to the project for continuous improvement.
- Extensive Documentation: Access comprehensive resources to help you get started and make the most of Malbox.

## Roadmap

**v0.1.0**
- [x] Basic back-end structure
- [x] Plugin system with shared memory IPC (via [iceoryx2](https://docs.rs/iceoryx2/latest/iceoryx2/))
  - [ ] Modularize analysis workflow using plugins
- [ ] Proper plugin scheduler / task management
- [ ] Develop components for machinery provisioning
  - [ ] Terraform/Ansible
  - [ ] VMWare ESXi
  - [ ] KVM/libvirt
  - [ ] Hyper-V
- [ ] Develop plugins for storage management
  - [ ] Amazon S3
  - [ ] On premise
- [ ] Develop plugins for static/dynamic analysis
- [ ] Configuration options (machines, submissions, file persistence, etc..) 
- [ ] Create diverse Rest API endpoints (monitoring, analysis, etc..)

**v0.2.0**
- [ ] Implement more complex plugins and cover as much as possible for detection (agent-less analysis)
- TBD!
---


![image](assets/malbox-panel-showcase.png)
