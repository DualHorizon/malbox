variable "name" {
    type = string
    description = "Name of the VM/template to create"
}

variable "iso_url" {
    type = string
    description = "URL or local path to the Windows ISO file"
}

variable "iso_checksum" {
    type = string
    description = "Checksum of the ISO file"
}

variable "disk_size" {
    type = number
    default = 61440
    description = "Disk size in MB"
}

variable "autounattend_path" {
    type = string
    description = "Path to the Autounattend.xml file"
}

variable "tools_path" {
    type = string
    default = "C:\\Tools"
    description = "Path where analysis tools will be installed"
}

# VMware Builder
source "vsphere-iso" "windows_analyzer" {
    vm_name = var.name
    disk_size = var.disk_size
    disk_thin_provisioned = true

    guest_os_type = "windows9_64Guest"
    iso_url = var.iso_url
    iso_checksum = var.iso_checksum

    floppy_files = [
        var.autounattend_path,
        "scripts/enable-winrm.ps1"
    ]
}

# VirtualBox Builder
source "virtualbox-iso" "windows_analyzer" {
    vm_name = var.name
    disk_size = var.disk_size
    guest_os_type = "Windows10_64"

    iso_url = var.iso_url
    iso_checksum = var.iso_checksum
    guest_additions_mode = "attach"

    floppy_files = [
        var.autounattend_path,
        "scripts/enable-winrm.ps1"
    ]
}

build {
    sources = [
        "source.vsphere-iso.windows_analyzer",
        "source.virtualbox-iso.windows_analyzer"
    ]

    provisioner "powershell" {
        scripts = [
            "scripts/disable-windows-defender.ps1",
            "scripts/disable-updates.ps1",
            "scripts/install-analysis-tools.ps1"
        ]
        environment_vars = [
            "TOOLS_PATH=${var.tools_path}"
        ]
    }

    provisioner "ansible" {
        playbook_file = "playbooks/windows.yml"
        extra_arguments = [
            "-e", "ansible_winrm_server_cert_validation=ignore",
            "-e", "tools_path=${var.tools_path}"
        ]
    }
}
