use std::env;
const USAGE: &str = "Uso: programa <DIRECTORIO> <NUM_HILOS> <OUTPUT_FILENAME>";
const BAD_NUM_TREADS: &str = "El número de worker threads debe ser un entero positivo";
pub struct CommandLineParser{}

impl CommandLineParser{
    pub fn arguments_parsing() -> (String, usize, String) {
        let mut args = env::args().skip(1);
    
        let directory_path = args
            .next()
            .expect(USAGE);
    
        let num_threads: usize = args
            .next()
            .expect(USAGE)
            .parse()
            .expect(BAD_NUM_TREADS);
    
        let output_filename = args
            .next()
            .expect(USAGE);
    
        (directory_path, num_threads, output_filename)
    }
}