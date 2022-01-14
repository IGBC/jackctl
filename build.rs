use gio::compile_resources;

fn main() {
    compile_resources(
        "resources",
        "resources/resources.xml",
        "resources.gresource",
    );
}
