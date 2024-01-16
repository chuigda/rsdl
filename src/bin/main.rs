use rsdl::codegen::pl5gen::PL5GeneratorFactory;
use rsdl::codegen::rustgen::RustGeneratorFactory;
use rsdl::codegen::tsgen::TSInterfaceGeneratorFactory;
use rsdl::driver::REFERENTIAL_STDLIB;

fn main() {
    rsdl::driver::application_start(
        REFERENTIAL_STDLIB,
        None,
        &[
            &RustGeneratorFactory(),
            &TSInterfaceGeneratorFactory(),
            &PL5GeneratorFactory()
        ]
    )
}
