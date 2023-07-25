use rsdl::codegen::rustgen::RustGeneratorFactory;
use rsdl::driver::REFERENTIAL_STDLIB;

fn main() {
    rsdl::driver::application_start(
        REFERENTIAL_STDLIB,
        None,
        &[&RustGeneratorFactory()]
    )
}
