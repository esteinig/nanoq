

use std::{ffi::OsStr, io::{BufWriter, Write}, path::{Path, PathBuf}};

use niffler::get_writer;

use termcolor::{
    Color, 
    ColorChoice, 
    ColorSpec, 
    StandardStream, 
    WriteColor
};

use crate::error::Error;


pub fn init_logger(log_file: Option<std::path::PathBuf>) {
    // Create a buffer for writing log messages with colors
    let mut builder = env_logger::Builder::new();

    // Define the log format
    builder.format(|buf, record| {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let mut stderr = StandardStream::stderr(ColorChoice::Always);

        let mut style = ColorSpec::new();
        match record.level() {
            log::Level::Trace => style.set_fg(Some(Color::White)).set_bold(true),
            log::Level::Debug => style.set_fg(Some(Color::Rgb(255, 195, 0))).set_bold(true),
            log::Level::Info => style.set_fg(Some(Color::Green)).set_bold(true),
            log::Level::Warn => style.set_fg(Some(Color::Rgb(255, 102, 0))).set_bold(true),
            log::Level::Error => style.set_fg(Some(Color::Red)).set_bold(true),
        };

        let mut default_style = ColorSpec::new();
        default_style.set_fg(Some(Color::White));

        let timestamp = buf.timestamp();

        if record.level() == log::Level::Error || record.level() == log::Level::Warn {
            
            stderr.set_color(&default_style).unwrap();
            write!(&mut stderr, "{} [", timestamp).unwrap();

            stderr.set_color(&style).unwrap();
            write!(&mut stderr,"{}", record.level()).unwrap();

            stderr.set_color(&default_style).unwrap();
            writeln!(&mut stderr, "] - {}", record.args()).unwrap();

            stderr.reset().unwrap();
        } else {
            
            stdout.set_color(&default_style).unwrap();
            write!(&mut stdout, "{} [", timestamp).unwrap();

            stdout.set_color(&style).unwrap();
            write!(&mut stdout, "{}", record.level()).unwrap();

            stdout.set_color(&default_style).unwrap();
            writeln!(&mut stdout, "] - {}", record.args()).unwrap();

            stdout.reset().unwrap();
        }

        Ok(())
    });

    // Set the default filter level to Info
    builder.filter(None, log::LevelFilter::Info);

    if let Some(log_path) = log_file {
        let file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_path)
            .unwrap();

        builder.target(env_logger::Target::Pipe(Box::new(file)));
    }

    builder.init();
}


/// Extension trait for inferring compression format from file extension.
pub trait CompressionExt {
    fn from_path<S: AsRef<OsStr> + ?Sized>(p: &S) -> Self;
}

impl CompressionExt for niffler::compression::Format {
    /// Attempts to infer the compression type from the file extension.
    /// If the extension is not known, `Uncompressed` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use scrubby::CompressionExt;
    /// let format = niffler::compression::Format::from_path("file.gz");
    /// ```
    fn from_path<S: AsRef<OsStr> + ?Sized>(p: &S) -> Self {
        let path = Path::new(p);
        match path.extension().map(|s| s.to_str()) {
            Some(Some("gz")) => Self::Gzip,
            Some(Some("bz") | Some("bz2")) => Self::Bzip,
            Some(Some("lzma") | Some("xz")) => Self::Lzma,
            _ => Self::No,
        }
    }
}

pub fn get_fastx_writer(
    output: &Path,
    compression_level: niffler::compression::Level,
    output_format: Option<niffler::compression::Format>,
) -> Result<Box<dyn std::io::Write>, Error> {

    let file = std::fs::File::create(output)
        .map_err(|_| Error::CreateFilePath(output.to_path_buf()))?;

    let file_handle = BufWriter::new(file);
    
    let format = match output_format {
        None => niffler::Format::from_path(output),
        Some(format) => format,
    };
    let writer = get_writer(
        Box::new(file_handle), 
        format, 
        compression_level
    )?;

    Ok(writer)
}

// Type traits for PathBuf and Vec<PathBuf>


pub trait IntoVecPathBuf {
    fn into(self) -> Vec<PathBuf>;
}
impl IntoVecPathBuf for &str {
    fn into(self) -> Vec<PathBuf> {
        vec![PathBuf::from(self)]
    }
}
impl IntoVecPathBuf for String {
    fn into(self) -> Vec<PathBuf> {
        vec![PathBuf::from(self)]
    }
}
impl IntoVecPathBuf for &Path {
    fn into(self) -> Vec<PathBuf> {
        vec![self.to_path_buf()]
    }
}
impl IntoVecPathBuf for Vec<&str> {
    fn into(self) -> Vec<PathBuf> {
        self.into_iter().map(PathBuf::from).collect()
    }
}
impl IntoVecPathBuf for Vec<String> {
    fn into(self) -> Vec<PathBuf> {
        self.into_iter().map(PathBuf::from).collect()
    }
}
impl IntoVecPathBuf for Vec<&Path> {
    fn into(self) -> Vec<PathBuf> {
        self.into_iter().map(|x| x.to_path_buf()).collect()
    }
}
impl IntoVecPathBuf for Vec<PathBuf> {
    fn into(self) -> Vec<PathBuf> {
        self
    }
}


pub trait IntoPathBuf {
    fn into(self) -> PathBuf;
}
impl IntoPathBuf for &str {
    fn into(self) -> PathBuf {
        PathBuf::from(self)
    }
}
impl IntoPathBuf for String {
    fn into(self) -> PathBuf {
        PathBuf::from(self)
    }
}
impl IntoPathBuf for &Path {
    fn into(self) -> PathBuf {
        self.to_path_buf()
    }
}