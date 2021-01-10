use clap::{Arg, ArgMatches, App, AppSettings};
use std::cmp::Ordering;
use std::process;
use libm::log10;
use std::fs::File;
use std::io::{stdin, Error};
use needletail::{parse_fastx_reader};

fn command_line_interface<'a>() -> ArgMatches<'a> {

    App::new("nanoq")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .setting(AppSettings::DisableHelpSubcommand)
    .version("0.2.0")
    .about("\nFast quality control and summary statistics for nanopore reads\n")
    .arg(Arg::with_name("FASTX").short("f").long("fastx").takes_value(true).help("Fastx path or STDIN [-]"))
    .arg(Arg::with_name("OUTPUT").short("o").long("output").takes_value(true).help("Output path or STDOUT [-]"))
    .arg(Arg::with_name("LENGTH").short("l").long("min_length").takes_value(true).help("Minimum sequence length [0]"))
    .arg(Arg::with_name("QUALITY").short("q").long("min_quality").takes_value(true).help("Minimum sequence quality [0]"))
    .arg(Arg::with_name("NEEDLE").short("n").long("needletail").takes_value(false).help("Use needletail as read parser [0]"))
    .get_matches()
}

fn main() -> Result<(), Error> {

    let cli = command_line_interface();
 
    let fastx = cli.value_of("FASTX").unwrap_or("-").parse::<String>().unwrap();

    let mut reader = if fastx == "-".to_string() {
        let stdin = stdin();
        parse_fastx_reader(stdin).expect("invalid /dev/stdin")
    } else {
        parse_fastx_reader(File::open(&fastx)?).expect("invalid file/path")
    };

    let _min_length: u64 = cli.value_of("LENGTH").unwrap_or("0").parse().unwrap();
    let _min_quality: f64 = cli.value_of("QUALITY").unwrap_or("0").parse().unwrap();
    

    let mut reads: u64 = 0;
    let mut base_pairs: u64 = 0;
    let mut read_lengths: Vec<u64> = Vec::new();
    let mut read_qualities: Vec<f64> = Vec::new();

    while let Some(record) = reader.next() {
        let seqrec = record.expect("invalid record");
        let seqlen = seqrec.seq().len() as u64;

        reads += 1;
        base_pairs += seqlen;
        read_lengths.push(seqlen);

        if let Some(qual) = seqrec.qual() {
            
            let mean_error = get_mean_error(&qual);
            let mean_quality: f64 = -10f64*log10(mean_error as f64);
            read_qualities.push(mean_quality);
        }
    }

    // Summary statistics

    if reads == 0 {
        eprintln!("No reads");
        process::exit(1);
    }

    let mean_read_length = get_mean_read_length(&read_lengths);
    let mean_read_quality = get_mean_read_quality(&read_qualities);
    let median_read_length = get_median_read_length(&mut read_lengths);
    let median_read_quality = get_median_read_quality(&mut read_qualities);
    let (min_read_length, max_read_length) = get_read_length_range(&read_lengths);
    let read_length_n50 = get_read_length_n50(&base_pairs, &read_lengths);

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

// Helper functions

fn compare_f64(a: &f64, b: &f64) -> Ordering {

    // Will get killed with NAN (R.I.P)
    // but we should also never see NAN

    if a < b {
        return Ordering::Less;
    } else if a > b {
        return Ordering::Greater;
    }
    Ordering::Equal
}

fn compare_u64(a: &u64, b: &u64) -> Ordering {

    // Will get killed with NAN (R.I.P)
    // but we should also never see NAN

    if a < b {
        return Ordering::Less;
    } else if a > b {
        return Ordering::Greater;
    }
    Ordering::Equal
}

fn get_mean_error(quality_bytes: &[u8]) -> f32 {

    /* Compute the mean error probability from a quality score vector

    Quality encoding: Sanger Phred+33 --> ASCII: 33 - 126 --> Q: 0 - 93

    f32 vs f64 makes a huge difference!

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

    numbers.sort_by(compare_f64);

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

fn get_read_length_n50(base_pairs: &u64, read_lengths: &Vec<u64>) -> u64 {
    
    // Compute the read length N50 if a vector of unsigned integers
    
    read_lengths.sort_by(compare_u64);

    println!("{:?}", read_lengths);

    let _stop = base_pairs / 2;

    let mut n50 = 0;
    let mut _cum_sum = 0;
    for x in read_lengths.rev().iter() {
        _cum_sum += x;
        if _cum_sum >= _stop {
            let n50 = x;
            break
        }
    }

    n50 as u64

}