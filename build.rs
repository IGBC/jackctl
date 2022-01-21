use gio::compile_resources;

fn main() {
    compile_resources(
        "res",
        "res/resources.xml",
        "resources.gresource",
    );
    println!("cargo:rerun-if-changed=res");
}
