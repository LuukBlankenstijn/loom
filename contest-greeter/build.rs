fn main() {
    pkg_config::Config::new()
        .atleast_version("1.0")
        .probe("liblightdm-gobject-1")
        .expect("Failed to find liblightdm-gobject-1 via pkg-config");
}
