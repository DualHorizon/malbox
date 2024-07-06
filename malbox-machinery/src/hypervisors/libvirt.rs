use virt::connect::Connect;

pub async fn libvirt_connect() {
    if let Ok(mut conn) = Connect::open("qemu:///system") {
        tracing::info!("ok!")
    }
}
