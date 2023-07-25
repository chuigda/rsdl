use rsdl::codegen::rustgen::RustGeneratorFactory;

fn main() {
    rsdl::driver::application_start(
        include_str!("../stdlib.rsdl"),
        None,
        &[
        &RustGeneratorFactory()
        ]
    )
}
