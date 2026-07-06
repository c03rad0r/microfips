fn main() {
    microfips_build::emit_all_keys();
    // Track WiFi credential env vars so cargo rebuilds when they change
    println!("cargo:rerun-if-env-changed=WIFI_SSID");
    println!("cargo:rerun-if-env-changed=WIFI_PASSWORD");
}
