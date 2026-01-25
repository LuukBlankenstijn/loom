use std::sync::Arc;

use crate::service::make_admin_service;

mod app;
mod service;
mod ui;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create temporary runtime");

    let service = rt.block_on(async {
        make_admin_service("http://localhost:8081".into())
            .await
            .expect("cannot create admin service")
    });

    if let Err(e) = app::run_app(Arc::new(service)) {
        println!("error running app: {:?}", e)
    }
}
