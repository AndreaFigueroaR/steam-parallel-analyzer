use crate::dtos::review_register::ReviewRegister;
use csv::{ByteRecord, ReaderBuilder};
use rayon::prelude::*;
use std::env;
use std::fs::{File, read_dir};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const USAGE: &str =
    "Uso: programa <DIRECTORIO> <NUM_HILOS> <OUTPUT_FILENAME> [DIRECTORIO_GRAFICOS]";
const BAD_NUM_TREADS: &str = "El número de worker threads debe ser un entero positivo";

/// Tamaño aproximado (en bytes) de cada bloque que el lector entrega al pool de Rayon.
/// Bloques de pocos MB amortizan la sincronización de `par_bridge` (un lock por bloque, no por
/// registro) y mantienen acotada la memoria en vuelo (~ `CHUNK_SIZE` × cantidad de workers).
pub const CHUNK_SIZE: usize = 4 * 1024 * 1024;

/// Argumentos de línea de comandos del binario principal.
pub struct CliArgs {
    pub directory_path: String,
    pub num_threads: usize,
    pub output_filename: String,
    pub charts_directory: Option<String>,
}

/// Bloque de bytes crudos de un CSV que contiene únicamente registros completos.
///
/// El hilo lector solo copia bytes del archivo y busca dónde cortar; el tokenizado CSV, la
/// validación UTF-8, el parseo de campos y la lógica de negocio ocurren en paralelo dentro de
/// los workers, que toman los campos prestados del bloque (zero-copy).
pub struct ReviewsBatch {
    headers: Arc<ByteRecord>,
    bytes: Vec<u8>,
}

impl ReviewsBatch {
    /// Tokeniza el bloque y entrega cada registro válido a `action`, con sus campos prestados.
    /// Se reutiliza un único `ByteRecord` para todo el bloque: no hay asignaciones por registro.
    /// Los registros mal formados (cantidad de campos distinta, tipos inválidos) se descartan.
    pub fn for_each_register(&self, mut action: impl FnMut(ReviewRegister<'_>)) {
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(self.bytes.as_slice());
        let mut record = ByteRecord::new();
        loop {
            match reader.read_byte_record(&mut record) {
                Ok(true) if record.len() == self.headers.len() => {
                    if let Ok(register) = record.deserialize::<ReviewRegister>(Some(&self.headers))
                    {
                        action(register);
                    }
                }
                Ok(true) | Err(_) => continue,
                Ok(false) => break,
            }
        }
    }
}

/// Devuelve la posición siguiente al último fin de línea que está fuera de comillas, es decir,
/// el final del último registro completo del bloque (el bloque debe empezar al inicio de un registro).
///
/// En un CSV RFC 4180 (como el que genera pandas) toda comilla literal se escribe duplicada dentro
/// de un campo entre comillas, por lo que un `\n` es fin de registro si y solo si la cantidad de
/// comillas vistas hasta él es par. Con `memchr` se salta directamente entre comillas y saltos de línea.
fn last_record_boundary(bytes: &[u8]) -> Option<usize> {
    let mut inside_quotes = false;
    let mut boundary = None;
    for position in memchr::memchr2_iter(b'"', b'\n', bytes) {
        if bytes[position] == b'"' {
            inside_quotes = !inside_quotes;
        } else if !inside_quotes {
            boundary = Some(position + 1);
        }
    }
    boundary
}

/// Igual que `last_record_boundary` pero para el primer registro (usado para separar el header).
fn first_record_boundary(bytes: &[u8]) -> Option<usize> {
    let mut inside_quotes = false;
    memchr::memchr2_iter(b'"', b'\n', bytes).find_map(|position| {
        if bytes[position] == b'"' {
            inside_quotes = !inside_quotes;
            None
        } else if inside_quotes {
            None
        } else {
            Some(position + 1)
        }
    })
}

/// Iterador secuencial que lee un CSV en bloques de ~`chunk_size` bytes cortados en límites de registro.
struct ChunkedRecords {
    file: File,
    headers: Arc<ByteRecord>,
    /// Bytes leídos que todavía no forman un registro completo; se anteponen al siguiente bloque.
    pending: Vec<u8>,
    chunk_size: usize,
    finished: bool,
}

impl ChunkedRecords {
    fn open(path: &Path, chunk_size: usize) -> io::Result<Self> {
        let mut chunks = ChunkedRecords {
            file: File::open(path)?,
            headers: Arc::new(ByteRecord::new()),
            pending: Vec::new(),
            chunk_size: chunk_size.max(1),
            finished: false,
        };
        if let Some(first) = chunks.read_chunk()? {
            let header_end = first_record_boundary(&first).unwrap_or(first.len());
            let mut header_reader = ReaderBuilder::new()
                .has_headers(false)
                .from_reader(&first[..header_end]);
            let mut headers = ByteRecord::new();
            header_reader
                .read_byte_record(&mut headers)
                .map_err(io::Error::other)?;
            chunks.headers = Arc::new(headers);
            let mut rest = first[header_end..].to_vec();
            rest.append(&mut chunks.pending);
            chunks.pending = rest;
        }
        Ok(chunks)
    }

    /// Lee del archivo hasta tener al menos un registro completo y devuelve los bytes de todos los
    /// registros completos disponibles. Al llegar al final del archivo devuelve lo que quede.
    fn read_chunk(&mut self) -> io::Result<Option<Vec<u8>>> {
        loop {
            if self.finished {
                return Ok((!self.pending.is_empty()).then(|| std::mem::take(&mut self.pending)));
            }
            let mut buffer = std::mem::take(&mut self.pending);
            buffer.reserve(self.chunk_size);
            let read = (&mut self.file)
                .take(self.chunk_size as u64)
                .read_to_end(&mut buffer)?;
            if read == 0 {
                self.finished = true;
                self.pending = buffer;
                continue;
            }
            match last_record_boundary(&buffer) {
                Some(end) => {
                    self.pending = buffer.split_off(end);
                    return Ok(Some(buffer));
                }
                // El bloque no contiene ningún registro completo (registro más largo que el bloque).
                None => self.pending = buffer,
            }
        }
    }
}

impl Iterator for ChunkedRecords {
    type Item = ReviewsBatch;

    fn next(&mut self) -> Option<ReviewsBatch> {
        match self.read_chunk() {
            Ok(Some(bytes)) => Some(ReviewsBatch {
                headers: Arc::clone(&self.headers),
                bytes,
            }),
            Ok(None) => None,
            Err(err) => {
                eprintln!("Error de lectura, se descarta el resto del archivo: {err}");
                self.finished = true;
                self.pending.clear();
                None
            }
        }
    }
}

pub struct IO {}

impl IO {
    /// Método que obtiene de la linea de comandos los argumentos: directorio de donde se leeran los datos, numero de workers, el nombre bajo el cual se guardará el resultado del procesamiento y, opcionalmente, el directorio donde se generarán los gráficos.
    pub fn parse_command_line_args() -> Result<CliArgs, String> {
        let mut args = env::args().skip(1);

        let directory_path = args.next().ok_or(USAGE)?;
        let num_threads = args
            .next()
            .ok_or(USAGE)?
            .parse::<usize>()
            .ok()
            .filter(|&n| n > 0)
            .ok_or(BAD_NUM_TREADS)?;
        let output_filename = args.next().ok_or(USAGE)?;
        let charts_directory = args.next();

        Ok(CliArgs {
            directory_path,
            num_threads,
            output_filename,
            charts_directory,
        })
    }

    fn csv_file_paths(directory_path: &str) -> io::Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        let entries = read_dir(directory_path).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("No se pudo leer el directorio '{directory_path}': {err}"),
            )
        })?;
        for entry in entries {
            let path = entry?.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "csv") {
                paths.push(path);
            }
        }
        Ok(paths)
    }

    /// Método que abre los archivos csv de un directorio y devuelve un iterador paralelo de lotes de registros.
    /// - Los archivos se reparten entre workers (`into_par_iter`).
    /// - Dentro de cada archivo, un lector secuencial produce bloques de ~`chunk_size` bytes con registros
    ///   completos, que `par_bridge` reparte entre workers para tokenizarlos y procesarlos en paralelo.
    pub fn par_iter_into_batches(
        directory_path: &str,
        chunk_size: usize,
    ) -> io::Result<impl ParallelIterator<Item = ReviewsBatch>> {
        let readers = IO::csv_file_paths(directory_path)?
            .iter()
            .map(|path| ChunkedRecords::open(path, chunk_size))
            .collect::<io::Result<Vec<_>>>()?;
        Ok(readers
            .into_par_iter()
            .flat_map(|reader| reader.par_bridge()))
    }

    pub fn save_json<T: serde::Serialize>(path: &str, output: &T) -> io::Result<()> {
        let mut writer = BufWriter::new(File::create(path)?);
        serde_json::to_writer_pretty(&mut writer, output)?;
        writer.flush()
    }
}
