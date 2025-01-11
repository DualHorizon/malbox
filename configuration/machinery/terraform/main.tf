module "windows_analysis_vm" {
  source = "./modules/analysis_vm"
  
  vm_template = "windows-analyzer-template"
  vm_name     = "win-analysis-${random_id.suffix.hex}"
  network_id  = var.isolated_network_id
  
  analysis_config = {
    file_type = var.file_type
    timeout   = var.analysis_timeout
    memory    = 4096
    vcpu      = 2
  }
}

# Dynamic VM configuration based on file type
locals {
  vm_config = {
    "exe" = {
      template = "windows-analyzer-template"
      memory   = 4096
      cpu      = 2
    }
    "elf" = {
      template = "linux-analyzer-template"
      memory   = 2048
      cpu      = 1
    }
  }
}

# Provision VM based on file type
resource "vsphere_virtual_machine" "analysis_vm" {
  name             = "analysis-${var.file_type}-${random_id.suffix.hex}"
  resource_pool_id = data.vsphere_resource_pool.pool.id
  datastore_id     = data.vsphere_datastore.datastore.id

  num_cpus = local.vm_config[var.file_type].cpu
  memory   = local.vm_config[var.file_type].memory
  guest_id = "windows9_64Guest"

  network_interface {
    network_id = var.isolated_network_id
  }

  disk {
    label = "disk0"
    size  = 61440
  }

  clone {
    template_uuid = data.vsphere_virtual_machine.template[local.vm_config[var.file_type].template].id
  }
}
