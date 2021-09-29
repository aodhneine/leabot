fn main() {
	println!("cargo:rustc-link-lib=static=tls");
	println!(concat!(
		"cargo:rustc-link-search=",
		env!("CARGO_MANIFEST_DIR"),
		"/vendor/build/lib"
	));
}
