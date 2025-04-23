use crate::dtos::review_register::ReviewRegister;
use csv::Reader;
use rayon::prelude::*;
use std::env;
use std::fs::{File, read_dir};
use std::io;
use std::io::BufWriter;
use std::path::PathBuf;

const USAGE: &str = "Uso: programa <DIRECTORIO> <NUM_HILOS> <OUTPUT_FILENAME>";
const BAD_NUM_TREADS: &str = "El número de worker threads debe ser un entero positivo";

pub struct IO {}

impl IO {
    pub fn parse_command_line_args() -> (String, usize, String) {
        let mut args = env::args().skip(1);

        let directory_path = args.next().expect(USAGE);

        let num_threads: usize = args.next().expect(USAGE).parse().expect(BAD_NUM_TREADS);

        let output_filename = args.next().expect(USAGE);

        (directory_path, num_threads, output_filename)
    }

    fn csv_file_paths(directory_path: String) -> Vec<PathBuf> {
        read_dir(directory_path)
            .unwrap()
            .map(|d| d.unwrap().path())
            .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "csv"))
            .collect::<Vec<PathBuf>>()
    }

    //metodoque lee los archivos csv de un directorio y devuelve un iterador paralelo de ReviewRegister
    pub fn par_iter_into_rewiews(
        directory_path: String,
    ) -> impl ParallelIterator<Item = ReviewRegister> {
        IO::csv_file_paths(directory_path)
            .into_par_iter()
            .flat_map(|path| {
                let file = File::open(path).unwrap();
                let reviews_per_file = Reader::from_reader(file)
                    .into_deserialize::<ReviewRegister>()
                    .par_bridge()
                    .filter_map(Result::ok);
                reviews_per_file
            })
    }
    pub fn save_json<T: serde::Serialize>(path: String, output: T) -> io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &output).unwrap();
        Ok(())
    }
}
