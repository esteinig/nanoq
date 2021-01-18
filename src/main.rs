use std::io::{BufWriter, BufReader, Write, Read};
use std::io::{stdin, stdout, Error};
use needletail::{parse_fastx_reader};
use clap::{Arg, ArgMatches, App};
use std::cmp::Ordering;
use bio::io::fastq;
use std::process;
use libm::log10;
use std::fs::File;

fn command_line_interface<'a>() -> ArgMatches<'a> {

    App::new("nanoq")
        .version("0.2.0")
        .about("\nFast quality control and summary statistics for nanopore reads\n")
        .arg(Arg::with_name("FASTX").short("f").long("fastx").takes_value(true).help("Fastx input file [-]"))
        .arg(Arg::with_name("OUTPUT").short("o").long("output").takes_value(true).help("Fastx output file [-]"))
        .arg(Arg::with_name("MINLEN").short("l").long("min_length").takes_value(true).help("Minimum sequence length [0]"))
        .arg(Arg::with_name("MAXLEN").short("m").long("max_length").takes_value(true).help("Maximum sequence length [0]"))
        .arg(Arg::with_name("QUALITY").short("q").long("min_quality").takes_value(true).help("Minimum sequence quality [0]"))
        .arg(Arg::with_name("PERCENT").short("p").long("keep_percent").takes_value(true).help("Keep best percent quality bases on reads (0 - 100) [0]"))
        .arg(Arg::with_name("BASES").short("b").long("keep_bases").takes_value(true).help("Keep reads with best quality number of bases [0]"))
        .arg(Arg::with_name("DETAIL").short("d").long("detail").takes_value(false).help("Pretty print dtailed stats [false]"))
        .arg(Arg::with_name("CRAB").short("c").long("crab").takes_value(false).help("Use the rust-bio parser (fastq) [false]"))
    .get_matches()

}

fn main() -> Result<(), Error> {

    let cli = command_line_interface();

    let fastx: String = cli.value_of("FASTX").unwrap_or("-").parse().unwrap();
    let output: String = cli.value_of("OUTPUT").unwrap_or("-").parse().unwrap();
    let min_length: u64 = cli.value_of("MINLEN").unwrap_or("0").parse().unwrap();
    let max_length: u64 = cli.value_of("MAXLEN").unwrap_or("0").parse().unwrap();
    let min_quality: f64 = cli.value_of("QUALITY").unwrap_or("0").parse().unwrap();
    let keep_percent: f64 = cli.value_of("PERCENT").unwrap_or("0").parse().unwrap();
    let keep_bases: usize = cli.value_of("BASES").unwrap_or("0").parse().unwrap();
    let crab: bool = cli.is_present("CRAB");
    let detail: bool = cli.is_present("DETAIL");
    
    if keep_percent > 0.0 || keep_bases > 0 {

        // Advanced mode

        if min_length > 0 || min_quality > 0.0 || max_length > 0 {
            eprintln!("Cannot specify length or quality filters with two-pass filters");
            process::exit(1);
        }

        two_pass_filter(fastx, keep_percent, keep_bases);

    } else {
        
        // Standard mode
         
        let (reads, base_pairs, mut read_lengths, mut read_qualities) = if crab {
            crabcast(fastx, output, min_length, max_length, min_quality)
        } else {
            if min_length > 0 || min_quality > 0.0 || max_length > 0 {
                needlecast_filter(fastx, output, min_length, max_length, min_quality)
            } else {
                needlecast_stats(fastx)
            }
        }.expect("Carcinised error encountered - what the crab?");

        if reads == 0 {
            eprintln!("No reads");
            process::exit(1);
        }

        eprint_stats(reads, base_pairs, read_lengths, read_qualities).expect("failed to collect stats");

    }


    Ok(())
    
}

// Main functions

fn crabcast(fastx: String, output: String, min_length: u64, max_length: u64, min_quality: f64) -> Result<(u64, u64, Vec<u64>, Vec<f64>), Error>  {

    // Rust-Bio parser, Fastq only

    let input_handle: Box<dyn Read> = if fastx == "-".to_string(){ 
        Box::new(BufReader::new(stdin()))
    } else {
        Box::new(File::open(&fastx)?)
    };

    let output_handle: Box<dyn Write> = if output == "-".to_string(){
        Box::new(BufWriter::new(stdout()))
     } else {
        Box::new(File::create(&output)?)
     };

    let reader = fastq::Reader::new(input_handle);
    let mut writer = fastq::Writer::new(output_handle);

    let max_length = if max_length <= 0 { u64::MAX } else { max_length };

    let mut base_pairs: u64 = 0;
    let mut reads: u64 = 0;
    let mut read_lengths: Vec<u64> = Vec::new();
    let mut read_qualities: Vec<f64> = Vec::new();

    for result in reader.records() {
        
        let record = result.expect("invalid sequence record");

        // Nanopore quality score computation

        let quality_values: Vec<u8> = record.qual().to_vec();
        let mean_error = get_mean_error(&quality_values);
        let mean_quality: f64 = -10f64*log10(mean_error as f64);

        let seqlen = record.seq().len() as u64;
                
        if seqlen >= min_length && mean_quality >= min_quality && seqlen <= max_length {
            
            read_lengths.push(seqlen);
            read_qualities.push(mean_quality);            
            base_pairs += seqlen;
            reads += 1;

            if min_length > 0 || min_quality > 0.0 || max_length > 0 {
                writer.write_record(&record).expect("Error: could not write record");
            }
        }           

    }  

    Ok((reads, base_pairs, read_lengths, read_qualities))

}

fn needlecast_filter(fastx: String, output: String, min_length: u64, max_length: u64, min_quality: f64) -> Result<(u64, u64, Vec<u64>, Vec<f64>), Error> {

    // Needletail parser, with output and filters
    
    let mut reader = if fastx == "-".to_string() {
        parse_fastx_reader(stdin()).expect("invalid stdin")
    } else {
        parse_fastx_reader(File::open(&fastx)?).expect("invalid file")
    };

    let mut output_handle: Box<dyn Write> = if output == "-".to_string(){
        Box::new(BufWriter::new(stdout()))
     } else {
        Box::new(File::create(&output)?)
     };

    let max_length = if max_length <= 0 { u64::MAX } else { max_length };

    let mut reads: u64 = 0;
    let mut base_pairs: u64 = 0;
    let mut read_lengths: Vec<u64> = Vec::new();
    let mut read_qualities: Vec<f64> = Vec::new();

    while let Some(record) = reader.next() {
        
        let seqrec = record.expect("invalid sequence record");
        let seqlen = seqrec.seq().len() as u64;
        
        // Quality scores present:
        if let Some(qual) = seqrec.qual() {
            let mean_error = get_mean_error(&qual);
            let mean_quality: f64 = -10f64*log10(mean_error as f64);
            // Fastq filter
            if seqlen >= min_length && mean_quality >= min_quality && seqlen <= max_length {
                reads += 1;
                base_pairs += seqlen;
                read_lengths.push(seqlen);
                read_qualities.push(mean_quality);
                seqrec.write(&mut output_handle, None).expect("invalid record write");
            }
        } else {
            // Fasta filter
            if seqlen >= min_length {
                reads += 1;
                base_pairs += seqlen;
                read_lengths.push(seqlen);
                seqrec.write(&mut output_handle, None).expect("invalid record write");
            }
        }

    }

    return Ok((reads, base_pairs, read_lengths, read_qualities))

}

fn needlecast_stats(fastx: String) -> Result<(u64, u64, Vec<u64>, Vec<f64>), Error> {

    // Needletail parser just for stats, no filters or output, slight speed-up
    
    let mut reader = if fastx == "-".to_string() {
        parse_fastx_reader(stdin()).expect("invalid stdin")
    } else {
        parse_fastx_reader(File::open(&fastx)?).expect("invalid file")
    };
    
    let mut reads: u64 = 0;
    let mut base_pairs: u64 = 0;
    let mut read_lengths: Vec<u64> = Vec::new();
    let mut read_qualities: Vec<f64> = Vec::new();

    while let Some(record) = reader.next() {
        
        let seqrec = record.expect("invalid sequence record");
        let seqlen = seqrec.seq().len() as u64;
        
        // Quality scores:
        if let Some(qual) = seqrec.qual() {
            let mean_error = get_mean_error(&qual);
            let mean_quality: f64 = -10f64*log10(mean_error as f64);
            read_qualities.push(mean_quality);
        } 

        reads += 1;
        base_pairs += seqlen;
        read_lengths.push(seqlen);

    }

    return Ok((reads, base_pairs, read_lengths, read_qualities))

}

fn two_pass_filter(fastx: String, keep_percent: f64, keep_bases: usize){

    // Advanced filters that require a single pass for stats, 
    // a second pass to output filtered reads; needs file input

    if !is_fastq(&fastx).expect("invalid file input") {
        eprintln!("Two pass filter requires fastq format with quality scores");
        process::exit(1);
    }

    // First pass, get read stats:
    let (reads, base_pairs, mut read_lengths, mut read_qualities) = needlecast_stats(fastx).expect("failed stats pass");

    let mut _qualities: Vec<(usize, f64)> = Vec::new();
    for (i, q) in read_qualities.iter().enumerate() {
        _qualities.push((i, *q));
    }

    // read_qualities.sort_by(|a, b| compare_f64_ascending(a[1], b[1]));

    // println!("{:?}", &read_qualities[1..5]);

}

// Base functions

fn eprint_stats(reads: u64, base_pairs: u64, mut read_lengths: Vec<u64>, mut read_qualities: Vec<f64>) -> Result<(), Error> {

    let mean_read_length = get_mean_read_length(&read_lengths);
    let mean_read_quality = get_mean_read_quality(&read_qualities);
    let median_read_length = get_median_read_length(&mut read_lengths);
    let median_read_quality = get_median_read_quality(&mut read_qualities);
    let read_length_n50 = get_read_length_n50(&base_pairs, &mut read_lengths);
    let (min_read_length, max_read_length) = get_read_length_range(&read_lengths);

    eprintln!(
        "{:} {:} {:} {:} {:} {:} {:} {:.2} {:.2}",
        reads, 
        base_pairs, 
        read_length_n50,
        max_read_length, 
        min_read_length, 
        mean_read_length, 
        median_read_length, 
        mean_read_quality, 
        median_read_quality
    );

    Ok(())

}

fn is_fastq(fastx: &String) -> Result<bool, Error> {
    
    let mut reader = if fastx == &"-".to_string() {
        parse_fastx_reader(stdin()).expect("invalid stdin")
    } else {
        parse_fastx_reader(File::open(&fastx)?).expect("invalid file")
    };

    let first_read = reader.next();

    println!("{:?}", first_read);

    Ok(true)
}

fn compare_f64_ascending(a: &f64, b: &f64) -> Ordering {

    // Will get killed with NAN (R.I.P)
    // but we should never see NAN

    if a < b {
        return Ordering::Less;
    } else if a > b {
        return Ordering::Greater;
    }
    Ordering::Equal
}

fn compare_f64_descending(a: &f64, b: &f64) -> Ordering {

    // Will get killed with NAN (R.I.P)
    // but we should never see NAN

    if a < b {
        return Ordering::Greater;
    } else if a > b {
        return Ordering::Less;
    }
    Ordering::Equal
}

fn compare_u64_descending(a: &u64, b: &u64) -> Ordering {

    // Will get killed with NAN (R.I.P)
    // but we should never see NAN

    if a < b {
        return Ordering::Greater;
    } else if a > b {
        return Ordering::Less;
    }
    Ordering::Equal
}

fn get_mean_error(quality_bytes: &[u8]) -> f32 {

    /* Compute the mean error probability from a quality score vector

    Quality encoding: Sanger Phred+33 --> ASCII: 33 - 126 --> Q: 0 - 93

    f32 vs f64 makes a huge difference - CHECK IF THIS WILL LIMIT THE READ LENGTH

    Computation of the base quality scores is described at:

    https://community.nanoporetech.com/technical_documents/data-analysis/

    https://gigabaseorgigabyte.wordpress.com/2017/06/26/averaging-basecall-quality-scores-the-right-way/

    */
    
    let mut sum: f32 = 0.0;
    for q in quality_bytes.iter(){
        sum += 10f32.powf((q-33u8) as f32 / -10f32)  // Q score and error probability
    }
    
    sum / quality_bytes.len() as f32  // Mean error probability

}

// Read length range

fn get_read_length_range(numbers: &Vec<u64>) -> (&u64, &u64) {

    let min_read_length = numbers.iter().min().expect("Could not determine minimum read length");
    let max_read_length = numbers.iter().max().expect("Could not determine maximum read length");
    
    return (min_read_length, max_read_length)

}

// Mean and medians for different numeric types

fn get_median_read_length(numbers: &mut Vec<u64>) -> u64 {
    
    // Compute the median of a vector of unsigned integers

    numbers.sort();

    let mid = numbers.len() / 2;
    if numbers.len() % 2 == 0 {
        get_mean_read_length(&vec![numbers[mid - 1], numbers[mid]]) as u64
    } else {
        numbers[mid]
    }

}

fn get_mean_read_length(numbers: &Vec<u64>) -> u64 {

    // Compute the mean of a vector of unsigned integers

    let sum: u64 = numbers.iter().sum();

    sum as u64 / numbers.len() as u64

}


fn get_median_read_quality(numbers: &mut Vec<f64>) -> f64 {

    // Compute the median of a vector of double-precision floats

    numbers.sort_by(compare_f64_ascending);

    let mid = numbers.len() / 2;
    if numbers.len() % 2 == 0 {
        get_mean_read_quality(&vec![numbers[mid - 1], numbers[mid]]) as f64
    } else {
        numbers[mid]
    }

}

fn get_mean_read_quality(numbers: &Vec<f64>) -> f64 {

    // Compute the mean of a vector of double-precision floats

    let sum: f64 = numbers.iter().sum();

    sum as f64 / numbers.len() as f64

}

fn get_read_length_n50(base_pairs: &u64, read_lengths: &mut Vec<u64>) -> u64 {
    
    // Compute the read length N50 of a vector of unsigned integers
    
    read_lengths.sort_by(compare_u64_descending);

    let _stop = base_pairs / 2;

    let mut n50: u64 = 0;
    let mut _cum_sum: u64 = 0;
    for x in read_lengths.iter() {
        _cum_sum += x;
        if _cum_sum >= _stop {
            n50 += x;
            break
        }
    }

    return n50

}