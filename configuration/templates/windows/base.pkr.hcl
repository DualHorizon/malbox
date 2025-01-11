source "vsphere-iso" "windows_analyzer" {
  vcenter_server      = var.vcenter_server
  username           = var.vsphere_username
  password           = var.vsphere_password
  insecure_connection = true

  vm_name     = "windows-analyzer-template"
  guest_os_type = "windows9_64Guest"

  CPUs            = 2
  RAM             = 4096
  disk_size       = 61440

  iso_paths = [
    "[datastore1] ISO/windows.iso"
  ]

  network_adapters {
    network = var.network_name
    network_card = "vmxnet3"
  }

  storage {
    disk_size = 61440
    disk_thin_provisioned = true
  }

  boot_command = [
    "<enter><wait><enter><wait>"
  ]
}

build {
  sources = ["source.vsphere-iso.windows_analyzer"]

  provisioner "powershell" {
    scripts = ["scripts/windows_setup.ps1"]
  }

  provisioner "ansible" {
    playbook_file = "../ansible/playbooks/windows.yml"
  }
}
