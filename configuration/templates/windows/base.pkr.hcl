# Common variables
variable "vm_name" {
  type = string
}

variable "os_version" {
  type = string
}

variable "cpu_num" {
  type = number
}

variable "memory" {
  type = number
}

# VMware variables
variable "vcenter_server" {
  type = string
  default = ""
}

variable "vsphere_username" {
  type = string
  default = ""
}

variable "vsphere_password" {
  type = string
  default = ""
  sensitive = true
}

variable "vmware_network" {
  type = string
  default = ""
}

# VirtualBox variables
variable "vbox_network" {
  type = string
  default = ""
}

variable "vbox_guest_os_type" {
  type = string
  default = ""
}

# KVM variables
variable "libvirt_uri" {
  type = string
  default = ""
}

variable "libvirt_network" {
  type = string
  default = ""
}

# VMware Builder
source "vsphere-iso" "windows_analyzer" {
  vcenter_server = var.vcenter_server
  username = var.vsphere_username
  password = var.vsphere_password
  insecure_connection = true
  shutdown_command = "shutdown /s /t 10 /f /d p:4:1 /c \"Packer Shutdown\""

  vm_name = "${var.vm_name}-vmware"
  guest_os_type = "windows9_64Guest"
  CPUs = var.cpu_num
  RAM = var.memory

  network_adapters {
    network = var.vmware_network
    network_card = "vmxnet3"
  }

  iso_paths = ["[datastore1] ISO/windows_${var.os_version}.iso"]

  floppy_files = [
    "scripts/autounattend.xml",
    "scripts/enable-winrm.ps1"
  ]
}

# VirtualBox Builder
source "virtualbox-iso" "windows_analyzer" {
  vm_name = "${var.vm_name}-virtualbox"
  guest_os_type = var.vbox_guest_os_type
  cpus = var.cpu_num
  memory = var.memory
  ssh_username = "root"
  shutdown_command = "shutdown /s /t 10 /f /d p:4:1 /c \"Packer Shutdown\""


  disk_size = 61440

  iso_url = "./en_windows_10_enterprise_ltsc_2019_x64_dvd_5795bb03.iso"
  iso_checksum = "sha256:b570ddfdc4672f4629a95316563df923bd834aec657de5d4ca7c7ef9b58df2b1"

  guest_additions_mode = "attach"

  vboxmanage = [
    ["modifyvm", "{{.Name}}", "--nic1", "hostonly", "--hostonlyadapter1", var.vbox_network],
    ["modifyvm", "{{.Name}}", "--vram", "128"]
  ]
}

# KVM/QEMU Builder
source "qemu" "windows_analyzer" {
  vm_name = "${var.vm_name}-kvm"
  accelerator = "kvm"

  cpus = var.cpu_num
  memory = var.memory
  disk_size = "61440M"
  shutdown_command = "shutdown /s /t 10 /f /d p:4:1 /c \"Packer Shutdown\""


  iso_url = "./en_windows_10_enterprise_ltsc_2019_x64_dvd_5795bb03.iso"
  iso_checksum = "sha256:b570ddfdc4672f4629a95316563df923bd834aec657de5d4ca7c7ef9b58df2b1"

  ssh_username = "root"
  net_device = "virtio-net"
  disk_interface = "virtio"
  qemuargs = [
    ["-netdev", "user,id=user.0,hostfwd=tcp::3389-:3389"],
    ["-device", "virtio-net,netdev=user.0"]
  ]
}

build {
  sources = [
    "source.vsphere-iso.windows_analyzer",
    "source.virtualbox-iso.windows_analyzer",
    "source.qemu.windows_analyzer"
  ]

  provisioner "powershell" {
    scripts = [
      "./scripts/disable-windows-defender.ps1",
      "./scripts/disable-updates.ps1",
      "./scripts/install-analysis-tools.ps1"
    ]
  }

  provisioner "ansible" {
    playbook_file = "playbooks/windows.yml"
    extra_arguments = [
      "-e", "ansible_winrm_server_cert_validation=ignore",
      "-e", "analysis_tools_path=C:\\Tools"
    ]
  }
}
