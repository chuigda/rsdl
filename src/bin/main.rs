use rsdl::codegen::rustgen::RustGeneratorFactory;

fn main() {
    rsdl::driver::application_start(&[
        &RustGeneratorFactory()
    ])
}