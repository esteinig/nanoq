use std::io::{BufWriter, BufReader, Write, Read};
use std::io::{stdin, stdout, Error};
use needletail::{parse_fastx_reader};
use clap::{Arg, ArgMatches, App};
use std::cmp::Ordering;
use bio::io::fastq;
use std::process;
use std::fs::File;
use needletail::parser::Format;
use needletail::parser::FastxReader;
use std::collections::HashMap;
use num_format::{Locale, ToFormattedString};

fn command_line_interface<'a>() -> ArgMatches<'a> {

    App::new("nanoq")
        .version("0.2.0")
        .about("\nFast quality control and summary statistics for nanopore reads\n")
        .arg(Arg::with_name("FASTX").short("f").long("fastx").takes_value(true).help("Fastx input file [-]"))
        .arg(Arg::with_name("OUTPUT").short("o").long("output").takes_value(true).help("Fastx output file [-]"))
        .arg(Arg::with_name("MINLEN").short("l").long("min_length").takes_value(true).help("Minimum sequence length [0]"))
        .arg(Arg::with_name("MAXLEN").short("m").long("max_length").takes_value(true).help("Maximum sequence length [0]"))
        .arg(Arg::with_name("QUALITY").short("q").long("min_quality").takes_value(true).help("Minimum average seq quality [0]"))
        .arg(Arg::with_name("PERCENT").short("p").long("keep_percent").takes_value(true).help("Keep best percent quality bases on reads (0 - 100) [0]"))
        .arg(Arg::with_name("BASES").short("b").long("keep_bases").takes_value(true).help("Keep reads with best quality number of bases [0]"))
        .arg(Arg::with_name("CRAB").short("c").long("crab").takes_value(false).help("Use the rust-bio parser (fastq) [false]"))
        .arg(Arg::with_name("DETAIL").short("d").long("detail").multiple(true).takes_value(false).help("Print detailed read summary [false]"))
        .arg(Arg::with_name("TOP").short("t").long("top").takes_value(true).help("Print <top> length + quality reads [5]"))
    .get_matches()

}

fn main() -> Result<(), Error> {

    let cli = command_line_interface();

    let fastx: String = cli.value_of("FASTX").unwrap_or("-").parse().unwrap();
    let output: String = cli.value_of("OUTPUT").unwrap_or("-").parse().unwrap();
    let min_length: u64 = cli.value_of("MINLEN").unwrap_or("0").parse().unwrap();
    let max_length: u64 = cli.value_of("MAXLEN").unwrap_or("0").parse().unwrap();
    let min_quality: f32 = cli.value_of("QUALITY").unwrap_or("0").parse().unwrap();
    let keep_percent: f64 = cli.value_of("PERCENT").unwrap_or("0").parse().unwrap();
    let keep_bases: usize = cli.value_of("BASES").unwrap_or("0").parse().unwrap();
    let top: u64 = cli.value_of("TOP").unwrap_or("5").parse().unwrap();
    let crab: bool = cli.is_present("CRAB");
    
    let detail: u64 = match cli.occurrences_of("d") {
        0 => 0,    // no details
        1 => 1,    // top ranks
        2 => 2,    // top ranks + thresholds
        3 | _ => 2 // anything more not effective
    };

    if keep_percent > 0.0 || keep_bases > 0 {

        // Advanced mode (Filtlong analog)

        if fastx == "-".to_string() || fastx == "/dev/stdin".to_string() {
            eprintln!("Cannot read from STDIN with advanced filters, must read from file");
            process::exit(1);
        }

        if min_length > 0 || min_quality > 0.0 || max_length > 0 {
            eprintln!("Cannot specify length or quality filters with advanced filters, keep filters only");
            process::exit(1);
        }

        two_pass_filter(fastx, output, keep_percent, keep_bases).expect("failed advanced filter");

    } else {
        
        // Standard mode
        
        let (reads, base_pairs, read_lengths, read_qualities) = if crab {
            crabcast_filter(fastx, output, min_length, max_length, min_quality)
        } else {
            if min_length > 0 || min_quality > 0.0 || max_length > 0 {
                needlecast_filter(fastx, output, min_length, max_length, min_quality)
            } else {
                needlecast_stats(&fastx)
            }
        }.expect("Carcinised error encountered - what the crab?");
        
        // This check prevents the stats computation from panicking on empty vec, see tests 
        if reads == 0 {
            eprintln!("No reads");
            process::exit(0); // exit gracefully
        }

        eprint_stats(reads, base_pairs, read_lengths, read_qualities, detail, top).expect("failed to collect stats");

    }


    Ok(())
    
}

// Main functions

fn crabcast_filter(fastx: String, output: String, min_length: u64, max_length: u64, min_quality: f32) -> Result<(u64, u64, Vec<u64>, Vec<f32>), Error>  {

    // Rust-Bio parser, Fastq only

    let input_handle = get_input_handle(fastx).expect("failed to initiate fastx input handle");
    let output_handle = get_output_handle(output).expect("failed to initiate fastx output handle");

    let reader = fastq::Reader::new(input_handle);
    let mut writer = fastq::Writer::new(output_handle);

    let _max_length = if max_length <= 0 { u64::MAX } else { max_length };

    let mut base_pairs: u64 = 0;
    let mut reads: u64 = 0;
    let mut read_lengths: Vec<u64> = Vec::new();
    let mut read_qualities: Vec<f32> = Vec::new();

    for result in reader.records() {
        
        let record = result.expect("invalid sequence record");

        // Nanopore quality score computation

        let quality_values = record.qual().to_vec();
        let mean_error = get_mean_error(&quality_values);
        let mean_quality: f32 = -10f32*mean_error.log(10.0);

        let seqlen = record.seq().len() as u64;
                
        if seqlen >= min_length && mean_quality >= min_quality && seqlen <= _max_length {
            
            read_lengths.push(seqlen);
            read_qualities.push(mean_quality);            
            base_pairs += seqlen;
            reads += 1;

            if min_length > 0 || min_quality > 0.0 || max_length > 0 {
                writer.write_record(&record).expect("invalid record write");
            }
        }           

    }  

    Ok((reads, base_pairs, read_lengths, read_qualities))

}

fn needlecast_filter(fastx: String, output: String, min_length: u64, max_length: u64, min_quality: f32) -> Result<(u64, u64, Vec<u64>, Vec<f32>), Error> {

    // Needletail parser, with output and filters
    
    let mut reader = get_needletail_reader(&fastx).expect("failed to initiate needletail reader");
    let mut output_handle = get_output_handle(output).expect("failed to initiate fastx output handle");

    let max_length = if max_length <= 0 { u64::MAX } else { max_length };

    let mut reads: u64 = 0;
    let mut base_pairs: u64 = 0;
    let mut read_lengths: Vec<u64> = Vec::new();
    let mut read_qualities: Vec<f32> = Vec::new();

    while let Some(record) = reader.next() {
        
        let seqrec = record.expect("invalid sequence record");
        let seqlen = seqrec.seq().len() as u64;
        
        // Quality scores present:
        if let Some(qual) = seqrec.qual() {
            let mean_error = get_mean_error(&qual);
            let mean_quality: f32 = -10f32*mean_error.log(10.0);
            // Fastq filter
            if seqlen >= min_length && mean_quality >= min_quality && seqlen <= max_length {
                reads += 1;
                base_pairs += seqlen;
                read_lengths.push(seqlen);
                read_qualities.push(mean_quality);
                seqrec.write(&mut output_handle, None).expect("invalid record write op");
            }
        } else {
            // Fasta filter
            if seqlen >= min_length {
                reads += 1;
                base_pairs += seqlen;
                read_lengths.push(seqlen);
                seqrec.write(&mut output_handle, None).expect("invalid record write op");
            }
        }

    }

    return Ok((reads, base_pairs, read_lengths, read_qualities))

}

fn needlecast_stats(fastx: &String) -> Result<(u64, u64, Vec<u64>, Vec<f32>), Error> {

    // Needletail parser just for stats, no filters or output, slight speed-up
    
    let mut reader = get_needletail_reader(fastx).expect("failed to initiate needletail reader");
    
    let mut reads: u64 = 0;
    let mut base_pairs: u64 = 0;
    let mut read_lengths: Vec<u64> = Vec::new();
    let mut read_qualities: Vec<f32> = Vec::new();

    while let Some(record) = reader.next() {
        
        let seqrec = record.expect("invalid sequence record");
        let seqlen = seqrec.seq().len() as u64;
        
        // Quality scores:
        if let Some(qual) = seqrec.qual() {
            let mean_error = get_mean_error(&qual);
            let mean_quality: f32 = -10f32*mean_error.log(10.0);
            read_qualities.push(mean_quality);
        } 
        
        reads += 1;
        base_pairs += seqlen;
        read_lengths.push(seqlen);

    }

    Ok((reads, base_pairs, read_lengths, read_qualities))

}

fn two_pass_filter(fastx: String, output: String, keep_percent: f64, keep_bases: usize) -> Result<(), Error> {

    // Advanced filters that require a single pass for stats, a second pass to output filtered reads

    if !is_fastq(&fastx).expect("invalid file input") {
        panic!("invalid fastq input");
    }

    if !(keep_percent >= 0. && keep_percent <= 100.) {
        panic!("keep percent range");
    }

    let keep_percent: f64 = if keep_percent == 0. {
        1.0
    } else {
        keep_percent / 100.
    };

    // First pass, get read stats:
    let (_, _, read_lengths, read_qualities) = needlecast_stats(&fastx).expect("failed stats pass");

    let indexed_qualities_retain = retain_indexed_quality_reads(read_qualities, read_lengths, keep_percent, keep_bases).expect("failed index collect");

    if indexed_qualities_retain.len() == 0 {
        eprintln!("No reads");
        process::exit(0);  // exit gracefully
    }

    // Second pass, filter reads to output by indices
    let mut _indices: HashMap<usize, f32> = indexed_qualities_retain.iter().cloned().collect();
    needlecast_index_filter(&fastx, output, _indices).expect("failed output pass"); // TODO: check if vec contains is faster
    
    Ok(())
}


fn needlecast_index_filter(fastx: &String, output: String, indices: HashMap<usize, f32>) -> Result<(), Error> {

    // Needletail parser just for output by read index:
    
    let mut reader = get_needletail_reader(fastx).expect("failed to initiate needletail reader");
    let mut output_handle = get_output_handle(output).expect("failed to initiate fastx output handle");
    
     let mut read: usize = 0;
     while let Some(record) = reader.next() {
        if indices.contains_key(&read) {  // test if this is faster than checking if index in vec
            let seqrec = record.expect("invalid sequence record");
            seqrec.write(&mut output_handle, None).expect("invalid record write op");
        }
        read += 1;
    }
        
    Ok(())

}

// Base functions

fn retain_indexed_quality_reads(read_qualities: Vec<f32>, read_lengths: Vec<u64>, keep_percent: f64, keep_bases: usize) -> Result<Vec<(usize, f32)>, Error> {

    // Index quality values by read amd fo;ter by kee-Percent or keep_bases

    let mut indexed_qualities: Vec<(usize, f32)> = Vec::new();
    for (i, q) in read_qualities.iter().enumerate() {
        indexed_qualities.push((i, *q));
    }

    // Sort (read index, qual) descending
    indexed_qualities.sort_by(compare_indexed_tuples_descending_f32);

    // Apply keep_percent (0 -> keep all)
    let _limit: usize = (indexed_qualities.len() as f64 * keep_percent) as usize;
    let mut _indexed_qualities_retain = &indexed_qualities[0.._limit];

    // Apply keep_bases 
    let mut indexed_qualities_retain: Vec<(usize, f32)> = Vec::new();
    if keep_bases > 0 {
        let mut bp_sum: usize = 0;
        for qtup in _indexed_qualities_retain.iter() {
            bp_sum += read_lengths[qtup.0 as usize] as usize;
            if bp_sum >= keep_bases {
                break;
            } else {
                indexed_qualities_retain.push(*qtup);
            }
        }
    } else {
        for qtup in _indexed_qualities_retain.iter() {
            indexed_qualities_retain.push(*qtup);
        }
    };

    return Ok(indexed_qualities_retain)

}

fn eprint_stats(reads: u64, base_pairs: u64, mut read_lengths: Vec<u64>, mut read_qualities: Vec<f32>, detail: u64, top: u64) -> Result<(u64, u64, u64, u64, u64, f64, f64), Error> {

    let mean_read_length = get_mean_read_length(&read_lengths);
    let mean_read_quality = get_mean_read_quality(&read_qualities);
    let median_read_length = get_median_read_length(&mut read_lengths);
    let median_read_quality = get_median_read_quality(&mut read_qualities);
    let read_length_n50 = get_read_length_n50(&base_pairs, &mut read_lengths);
    let (min_read_length, max_read_length) = get_read_length_range(&read_lengths);

    if detail > 0 {

        if reads < top {
            panic!("Must have at least {:} reads for extended summary output", top)
        }

        eprintln!(
            "
Nanoq v0.2.0: Read Summary
==============================

Number of reads:     {:}
Number of bases:     {:}
N50:                 {:}
Longest read:        {:} 
Shortest read:       {:}
Mean read length:    {:}
Median read length:  {:} 
Mean read quality:   {:.1} 
Median read quality: {:.1}
            ",
            reads.to_formatted_string(&Locale::en), 
            base_pairs.to_formatted_string(&Locale::en), 
            read_length_n50.to_formatted_string(&Locale::en),
            max_read_length.to_formatted_string(&Locale::en), 
            min_read_length.to_formatted_string(&Locale::en), 
            mean_read_length.to_formatted_string(&Locale::en), 
            median_read_length.to_formatted_string(&Locale::en), 
            mean_read_quality, 
            median_read_quality
        );


        if detail > 1 {
            let mut indexed_lengths: Vec<(usize, u64)> = Vec::new();
            for (i, q) in read_lengths.iter().enumerate() {
                indexed_lengths.push((i, *q));
            }
    
            &indexed_lengths.sort_by(compare_indexed_tuples_descending_u64);
    
            let mut indexed_qualities: Vec<(usize, f32)> = Vec::new();
            for (i, q) in read_qualities.iter().enumerate() {
                indexed_qualities.push((i, *q));
            }
    
            &indexed_qualities.sort_by(compare_indexed_tuples_descending_f32);
            
             // Read lengths
            eprintln!("Top ranking read lengths\n");
            for i in 0..top {
                let (_, length) = indexed_lengths[i as usize];
                eprintln!("{}. {:} bp", i+1, length);
            }
            eprintln!("");
    
            // Read quality
            eprintln!("Top ranking mean read qualities\n");
            for i in 0..top {
                let (_, qual) = indexed_qualities[i as usize];
                eprintln!("{}. Q{:}", i+1, qual);
            }


            // Read length thresholds
        }
        


    } else {
        
        eprintln!(
            "{:} {:} {:} {:} {:} {:} {:} {:.1} {:.1}",
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
    }

    Ok((read_length_n50, max_read_length, min_read_length, mean_read_length, median_read_length, mean_read_quality, median_read_quality))

}

fn is_fastq(fastx: &String) -> Result<bool, Error> {
    
    let mut reader = get_needletail_reader(fastx).expect("failed to initiate needletail reader");

    let first_read = reader.next().unwrap().unwrap();
    let read_format = first_read.format();

    if read_format == Format::Fastq {
        Ok(true)
    } else {
        Ok(false)
    } 
}

fn get_needletail_reader(fastx: &String) -> Result<Box<dyn FastxReader>, Error> {
    if fastx == &"-".to_string() {
        Ok(parse_fastx_reader(stdin()).expect("invalid stdin"))
    } else {
        Ok(parse_fastx_reader(File::open(&fastx)?).expect("invalid file"))
    }

}

fn get_output_handle(output: String) -> Result<Box<dyn Write>, Error> {
    if output == "-".to_string(){
        Ok(Box::new(BufWriter::new(stdout())))
    } else {
        Ok(Box::new(File::create(&output)?))
    }
}

fn get_input_handle(fastx: String) -> Result<Box<dyn Read>, Error> {
    if fastx == "-".to_string(){ 
        Ok(Box::new(BufReader::new(stdin())))
    } else {
        Ok(Box::new(File::open(&fastx)?))
    }
}

fn compare_indexed_tuples_descending_f32(a: &(usize, f32), b: &(usize, f32)) -> Ordering {

    // Will get killed with NAN (R.I.P)
    // but we should never see NAN

    if a.1 < b.1 {
        return Ordering::Greater;
    } else if a.1 > b.1 {
        return Ordering::Less; 
    }
    Ordering::Equal
   
}

fn compare_indexed_tuples_descending_u64(a: &(usize, u64), b: &(usize, u64)) -> Ordering {

    // Will get killed with NAN (R.I.P)
    // but we should never see NAN

    if a.1 < b.1 {
        return Ordering::Greater;
    } else if a.1 > b.1 {
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

fn compare_f32_ascending(a: &f32, b: &f32) -> Ordering {

    // Will get killed with NAN (R.I.P)
    // but we should never see NAN

    if a < b {
        return Ordering::Less;
    } else if a > b {
        return Ordering::Greater;
    }
    Ordering::Equal
    
}

#[allow(dead_code)]
fn compare_f32_descending(a: &f32, b: &f32) -> Ordering {

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

    f32 vs f64 makes a huge difference - DOCS: CHECK IF THIS WILL LIMIT READ LENGTH

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

fn get_read_length_range(numbers: &Vec<u64>) -> (u64, u64) {

    let min_read_length = numbers.iter().min().expect("Could not determine minimum read length");
    let max_read_length = numbers.iter().max().expect("Could not determine maximum read length");
    
    return (*min_read_length, *max_read_length)

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

fn get_median_read_quality(numbers: &mut Vec<f32>) -> f64 {

    // Compute the median of a vector of single-precision floats

    numbers.sort_by(compare_f32_ascending);

    let mid = numbers.len() / 2;
    if numbers.len() % 2 == 0 {
        get_mean_read_quality(&vec![numbers[mid - 1], numbers[mid]])
    } else {
        numbers[mid] as f64
    }

}

fn get_mean_read_quality(numbers: &Vec<f32>) -> f64 {

    // Compute the mean of a vector of single-precision floats

    let sum: f32 = numbers.iter().sum();

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

#[cfg(test)]
mod tests {
    use super::*;

    // Test files

    fn get_root() -> String {
        let mut root: String = env!("CARGO_MANIFEST_DIR").to_string();
        root.push_str("/data/");
        return root;
    }

    fn get_test_fq() -> String {
        let mut root: String = get_root();
        root.push_str(&String::from("test.fq"));
        return root;
    }

    fn get_test_fq_gz() -> String {
        let mut root: String = get_root();
        root.push_str(&String::from("test.fq.gz"));
        return root;
    }

    fn get_test_fa() -> String {
        let mut root: String = get_root();
        root.push_str(&String::from("test.fa"));
        return root;
    }

    fn get_test_fa_gz() -> String {
        let mut root: String = get_root();
        root.push_str(&String::from("test.fa.gz"));
        return root;
    }


    // Crabcast based filters and stats

    #[test]
    fn test_crabcast_filter_all_pass() {
        let test_file = get_test_fq();
        let (reads, base_pairs, read_lengths, read_qualities) = crabcast_filter(test_file, String::from("/dev/null"), 1, 0, 7.0).unwrap();
        assert_eq!(reads, 1);
        assert_eq!(base_pairs, 12);
        assert_eq!(read_lengths, vec![12]);
        assert_eq!(read_qualities, vec![40.0]);
    }

    #[test]
    fn test_crabcast_filter_min_length_none_pass() {
        let test_file = get_test_fq();
        let (reads, base_pairs, read_lengths, read_qualities) = crabcast_filter(test_file, String::from("/dev/null"), 15, 0, 7.0).unwrap();
        assert_eq!(reads, 0);
        assert_eq!(base_pairs, 0);
        assert_eq!(read_lengths, vec![]);
        assert_eq!(read_qualities, vec![]);
    }

    #[test]
    fn test_crabcast_filter_max_length_none_pass() {
        let test_file = get_test_fq();
        let (reads, base_pairs, read_lengths, read_qualities) = crabcast_filter(test_file, String::from("/dev/null"), 10, 10, 7.0).unwrap();
        assert_eq!(reads, 0);
        assert_eq!(base_pairs, 0);
        assert_eq!(read_lengths, vec![]);
        assert_eq!(read_qualities, vec![]);
    }

    #[test]
    fn test_crabcast_filter_min_quality_none_pass() {
        let test_file = get_test_fq();
        let (reads, base_pairs, read_lengths, read_qualities) = crabcast_filter(test_file, String::from("/dev/null"), 10, 0, 60.0).unwrap();
        assert_eq!(reads, 0);
        assert_eq!(base_pairs, 0);
        assert_eq!(read_lengths, vec![]);
        assert_eq!(read_qualities, vec![]);
    }

    // Needlecast based filters and stats

    #[test]
    fn test_needlecast_filter_all_pass() {
        let test_file = get_test_fq();
        let (reads, base_pairs, read_lengths, read_qualities) = needlecast_filter(test_file, String::from("/dev/null"), 1, 0, 7.0).unwrap();
        assert_eq!(reads, 1);
        assert_eq!(base_pairs, 12);
        assert_eq!(read_lengths, vec![12]);
        assert_eq!(read_qualities, vec![40.0]);
    }

    #[test]
    fn test_needlecast_filter_min_length_none_pass() {
        let test_file = get_test_fq();
        let (reads, base_pairs, read_lengths, read_qualities) = needlecast_filter(test_file, String::from("/dev/null"), 15, 0, 7.0).unwrap();
        assert_eq!(reads, 0);
        assert_eq!(base_pairs, 0);
        assert_eq!(read_lengths, vec![]);
        assert_eq!(read_qualities, vec![]);
    }

    #[test]
    fn test_needlecast_filter_max_length_none_pass() {
        let test_file = get_test_fq();
        let (reads, base_pairs, read_lengths, read_qualities) = needlecast_filter(test_file, String::from("/dev/null"), 10, 10, 7.0).unwrap();
        assert_eq!(reads, 0);
        assert_eq!(base_pairs, 0);
        assert_eq!(read_lengths, vec![]);
        assert_eq!(read_qualities, vec![]);
    }

    #[test]
    fn test_needlecast_filter_min_quality_none_pass() {
        let test_file = get_test_fq();
        let (reads, base_pairs, read_lengths, read_qualities) = needlecast_filter(test_file, String::from("/dev/null"), 10, 0, 60.0).unwrap();
        assert_eq!(reads, 0);
        assert_eq!(base_pairs, 0);
        assert_eq!(read_lengths, vec![]);
        assert_eq!(read_qualities, vec![]);
    }

    #[test]
    fn test_needlecast_stats_fq() {
        let test_file = get_test_fq();
        let (reads, base_pairs, read_lengths, read_qualities) = needlecast_stats(&test_file).unwrap();
        assert_eq!(reads, 1);
        assert_eq!(base_pairs, 12);
        assert_eq!(read_lengths, vec![12]);
        assert_eq!(read_qualities, vec![40.0]);
    }

    #[test]
    fn test_needlecast_stats_fa() {
        let test_file = get_test_fa();
        let (reads, base_pairs, read_lengths, read_qualities) = needlecast_stats(&test_file).unwrap();
        assert_eq!(reads, 1);
        assert_eq!(base_pairs, 12);
        assert_eq!(read_lengths, vec![12]);
        assert_eq!(read_qualities, vec![]);
    }

    #[test]
    fn test_two_pass_filter_main_fq() {
        let test_file = get_test_fq();
        let completed = two_pass_filter(test_file, String::from("/dev/null"), 100.0, 0);
        assert!(completed.is_ok());
    }

    #[test]
    #[should_panic]  // fasta not supported, need qual scores
    fn test_two_pass_filter_main_fa() {
        let test_file = get_test_fa();
        let _ = two_pass_filter(test_file, String::from("/dev/null"), 100.0, 0);
    }

    #[test]
    fn test_needlecast_index_filter_fq() {
        let test_file = get_test_fq();
        let _indices: Vec<(usize, f32)> = vec![(0, 40.0)];
        let indices: HashMap<usize, f32> = _indices.iter().cloned().collect();
        let completed = needlecast_index_filter(&test_file, String::from("/dev/null"), indices);
        assert!(completed.is_ok());
    }

    #[test]
    fn test_needlecast_index_filter_fa() {
        let test_file = get_test_fa();
        let _indices: Vec<(usize, f32)> = vec![(0, 40.0)];
        let indices: HashMap<usize, f32> = _indices.iter().cloned().collect();
        let completed = needlecast_index_filter(&test_file, String::from("/dev/null"), indices);
        assert!(completed.is_ok());
    }

    // Retain indices from quality filtering
    
    #[test]
    fn test_retain_indexed_quality_reads_keep_percent_retain_none() {
        let read_qualities: Vec<f32> = vec![10.0, 20.0, 20.0, 30.0];
        let read_lengths: Vec<u64> = vec![10, 20, 20, 30];
        let keep_percent: f64 = 0.1;
        let keep_bases: usize = 0;
        let indices = retain_indexed_quality_reads(read_qualities, read_lengths, keep_percent, keep_bases).unwrap();
        assert_eq!(indices.len(), 0); 
        assert_eq!(indices, vec![]);
    }

    #[test]
    fn test_retain_indexed_quality_reads_keep_percent_retain_some() {
        let read_qualities: Vec<f32> = vec![10.0, 20.0, 20.0, 30.0];
        let read_lengths: Vec<u64> = vec![10, 20, 20, 30];
        let keep_percent: f64 = 0.50;
        let keep_bases: usize = 0;
        let indices = retain_indexed_quality_reads(read_qualities, read_lengths, keep_percent, keep_bases).unwrap();
        assert_eq!(indices.len(), 2); // order of quality sort with equal elements not guaranteed
    }

    #[test]
    fn test_retain_indexed_quality_reads_keep_bases_retain_none() {
        let read_qualities: Vec<f32> = vec![10.0, 20.0, 20.0, 30.0];
        let read_lengths: Vec<u64> = vec![10, 20, 20, 30];
        let keep_percent: f64 = 0.1;
        let keep_bases: usize = 0;
        let indices = retain_indexed_quality_reads(read_qualities, read_lengths, keep_percent, keep_bases).unwrap();
        assert_eq!(indices.len(), 0);
        assert_eq!(indices, vec![]);
    }

    #[test]
    fn test_retain_indexed_quality_reads_keep_bases_retain_some() {
        let read_qualities: Vec<f32> = vec![10.0, 20.0, 20.0, 30.0];
        let read_lengths: Vec<u64> = vec![10, 20, 20, 30];
        let keep_percent: f64 = 1.0;
        let keep_bases: usize = 50;
        let indices = retain_indexed_quality_reads(read_qualities, read_lengths, keep_percent, keep_bases).unwrap();
        assert_eq!(indices.len(), 1); // order of quality sort with equal elements not guaranteed
    }

    #[test]
    fn test_retain_indexed_quality_reads_both() {
        let read_qualities: Vec<f32> = vec![10.0, 20.0, 20.0, 30.0];
        let read_lengths: Vec<u64> = vec![10, 20, 20, 30];
        let keep_percent: f64 = 0.75;
        let keep_bases: usize = 60;
        let indices = retain_indexed_quality_reads(read_qualities, read_lengths, keep_percent, keep_bases).unwrap();
        assert_eq!(indices.len(), 2); // order of quality sort with equal elements not guaranteed
    }


    // Eprint stats function

    #[test]
    fn test_eprint_stats_detail_success() {
        let reads: u64 = 5;
        let base_pairs: u64 = 80;
        let read_lengths: Vec<u64> = vec![20, 10, 30, 20, 10];
        let read_qualities: Vec<f32> = vec![10.0, 20.0, 20.0, 30.0];
        let (
            read_length_n50, 
            max_read_length, 
            min_read_length, 
            mean_read_length, 
            median_read_length, 
            mean_read_quality, 
            median_read_quality
        ) = eprint_stats(reads, base_pairs, read_lengths, read_qualities, 1, 5).unwrap(); // detailed

        assert_eq!(read_length_n50, 20);
        assert_eq!(max_read_length, 30);
        assert_eq!(min_read_length, 10);
        assert_eq!(mean_read_length, 18);
        assert_eq!(median_read_length, 20);
        assert_eq!(mean_read_quality, 18.0);
        assert_eq!(median_read_quality, 20.0);
    }

    #[test]
    #[should_panic]
    fn test_eprint_stats_detail_fail() {
        let reads: u64 = 5;
        let base_pairs: u64 = 80;
        let read_lengths: Vec<u64> = vec![20, 10, 30, 20, 10];
        let read_qualities: Vec<f32> = vec![10.0, 20.0, 20.0, 30.0];
        eprint_stats(reads, base_pairs, read_lengths, read_qualities, 1, 10).unwrap(); // panics at implausible top param
    }


    // Fastq

    #[test]
    fn test_is_fastq_fq_file() {
        let test_file = get_test_fq();
        let is_fastq = is_fastq(&test_file).unwrap();
        assert!(is_fastq);
    }

    #[test]
    fn test_is_fastq_fq_gz_file() {
        let test_file = get_test_fq_gz();
        let is_fastq = is_fastq(&test_file).unwrap();
        assert!(is_fastq);
    }

    #[test]
    fn test_is_fastq_fa_file() {
        let test_file = get_test_fa();
        let is_fastq = is_fastq(&test_file).unwrap();
        assert!(!is_fastq);
    }

    #[test]
    fn test_is_fastq_fa_gz_file() {
        let test_file = get_test_fa_gz();
        let is_fastq = is_fastq(&test_file).unwrap();
        assert!(!is_fastq);
    }

    // Needletail IO

    #[test]
    fn test_needletail_input_fq_file() {
        let test_file = get_test_fq();
        let mut reader = get_needletail_reader(&test_file).expect("invalid input handle");

        while let Some(record) = reader.next() {
            let record = record.expect("invalid sequence record");
            assert_eq!(&record.id(), b"id");
            assert_eq!(&record.raw_seq(), b"ACCGTAGGCTGA");
            assert_eq!(&record.qual().unwrap(), b"IIIIIIJJJJJJ");
        }
    }

    #[test]
    fn test_needletail_input_fq_gz_file() {
        let test_file = get_test_fq_gz();
        let mut reader = get_needletail_reader(&test_file).expect("invalid input handle");

        while let Some(record) = reader.next() {
            let record = record.expect("invalid sequence record");
            assert_eq!(&record.id(), b"id");
            assert_eq!(&record.raw_seq(), b"ACCGTAGGCTGA");
            assert_eq!(&record.qual().unwrap(), b"IIIIIIJJJJJJ");
        }
    }

    #[test]
    fn test_needletail_input_fa_file() {
        let test_file = get_test_fa();
        let mut reader = get_needletail_reader(&test_file).expect("invalid input handle");

        while let Some(record) = reader.next() {
            let record = record.expect("invalid sequence record");
            assert_eq!(&record.id(), b"id");
            assert_eq!(&record.raw_seq(), b"ACCGTAGGCTGA");
        }
    }

    #[test]
    fn test_needletail_input_fa_gz_file() {
        let test_file = get_test_fa_gz();
        let mut reader = get_needletail_reader(&test_file).expect("invalid input handle");

        while let Some(record) = reader.next() {
            let record = record.expect("invalid sequence record");
            assert_eq!(&record.id(), b"id");
            assert_eq!(&record.raw_seq(), b"ACCGTAGGCTGA");
        }
    }

    // Rust-Bio IO

    #[test]
    fn test_crabcast_filter_input_fq_file() {
        let test_file = get_test_fq();
        let input_handle = get_input_handle(test_file).expect("invalid input handle");
        let reader = fastq::Reader::new(input_handle);

        for record in reader.records() {
            let record = record.expect("invalid sequence record");
            assert_eq!(record.check(), Ok(()));
            assert_eq!(record.id(), "id");
            assert_eq!(record.desc(), None);
            assert_eq!(record.seq(), b"ACCGTAGGCTGA");
            assert_eq!(record.qual(), b"IIIIIIJJJJJJ");
        }
    }

    #[test]
    #[should_panic]
    fn test_crabcast_filter_input_fq_gz_file() {
        let test_file = get_test_fq_gz();
        let input_handle = get_input_handle(test_file).expect("invalid input handle");
        let reader = fastq::Reader::new(input_handle);
        let _ = reader.records().next().unwrap().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_crabcast_filter_input_fa_file() {
        let test_file = get_test_fa();
        let input_handle = get_input_handle(test_file).expect("invalid input handle");
        let reader = fastq::Reader::new(input_handle);
        let _ = reader.records().next().unwrap().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_crabcast_filter_input_fa_gz_file() {
        let test_file = get_test_fa_gz();
        let input_handle = get_input_handle(test_file).expect("invalid input handle");
        let reader = fastq::Reader::new(input_handle);
        let _ = reader.records().next().unwrap().unwrap();
    }

    // Ordering

    #[test]
    fn test_compare_indexed_tuples_descending_u64() {
        let mut test_data: Vec<(usize, u64)> = vec![(0, 30), (1, 10), (2, 50)];
        test_data.sort_by(compare_indexed_tuples_descending_u64);
        assert_eq!(test_data, vec![(2, 50), (0, 30), (1, 10)]);
        assert_eq!(test_data.sort_by_key(|tup| tup.1).reverse(), vec![(2, 50), (0, 30), (1, 10)]);
    }
    
    #[test]
    fn test_compare_indexed_tuples_descending_f32() {
        let mut test_data: Vec<(usize, f32)> = vec![(0, 30.0), (1, 10.0), (2, 50.0)];
        test_data.sort_by(compare_indexed_tuples_descending_f32);
        assert_eq!(test_data, vec![(2, 50.0), (0, 30.0), (1, 10.0)]);
    }

    #[test]
    fn test_compare_u64_descending() {
        let mut test_data: Vec<u64> = vec![1, 5, 2];
        test_data.sort_by(compare_u64_descending);
        assert_eq!(test_data, vec![5, 2, 1]);
    }

    #[test]
    fn test_compare_f32_ascending() {
        let mut test_data: Vec<f32> = vec![1.0, 5.0, 2.0];
        test_data.sort_by(compare_f32_ascending);
        assert_eq!(test_data, vec![1.0, 2.0, 5.0]);
    }

    // Mean read error

    #[test]
    fn test_mean_error_function() {
        let mean_error = get_mean_error(b"???");
        assert_eq!(mean_error, 0.001 as f32);
    }

    // Mean read q-score

    #[test]
    fn test_mean_error_qscore() {
        let mean_error = get_mean_error(b"IIIIIIJJJJJJ");
        let mean_quality: f32 = -10f32*mean_error.log(10.0);
        assert_eq!(mean_quality, 40 as f32);
    }

    // N50

    #[test]
    fn test_read_length_n50_empty() {
        let mut test_data: Vec<u64> = Vec::new();
        let n50 = get_read_length_n50(&70, &mut test_data);
        assert_eq!(n50, 0 as u64);
    }

    #[test]
    fn test_read_length_n50() {
        let n50 = get_read_length_n50(&70, &mut vec![10, 10, 20, 30]);
        assert_eq!(n50, 20 as u64);
    }

    // Read quality

    #[test]
    fn test_mean_read_quality_empty() {
        let test_data: Vec<f32> = Vec::new();
        let mean_quality = get_mean_read_quality(&test_data);
        assert!(mean_quality.is_nan()); // f32 returns NaN on ZeroDivision
    }    

    #[test]
    fn test_mean_read_quality() {
        let mean_quality = get_mean_read_quality(&mut vec![10.0, 10.0, 20.0, 30.0]);
        assert_eq!(mean_quality, 17.5);
    }

    
    #[test]
    #[should_panic]
    fn test_median_read_quality_empty() {
        let mut test_data: Vec<f32> = Vec::new();
        let _ = get_median_read_quality(&mut test_data);
    }    

    #[test]
    fn test_median_read_quality_even() {
        let median_quality = get_median_read_quality(&mut vec![10.0, 10.0, 20.0, 30.0]);
        assert_eq!(median_quality, 15.0);
    }

    #[test]
    fn test_median_read_quality_odd() {
        let median_quality = get_median_read_quality(&mut vec![10.0, 10.0, 20.0, 30.0]);
        assert_eq!(median_quality, 20.0);
    }

    // Read lengths

    #[test]
    #[should_panic]
    fn test_mean_read_length_empty() {
        let mut test_data: Vec<u64> = Vec::new();
        let _ = get_mean_read_length(&mut test_data);
    }

    #[test]
    fn test_mean_read_length() {
        let mean_length = get_mean_read_length(&vec![10, 10, 20, 30]);
        assert_eq!(mean_length, 17 as u64);
    }

    #[test]
    #[should_panic]
    fn test_median_read_length_empty() {
        let mut test_data: Vec<u64> = Vec::new();
        let _ = get_median_read_length(&mut test_data);
    }

    #[test]
    fn test_median_read_length_even() {
        let median_length = get_median_read_length(&mut vec![10, 10, 20, 30]);
        assert_eq!(median_length, 15 as u64);
    }

    #[test]
    fn test_median_read_length_odd() {
        let median_length = get_median_read_length(&mut vec![10, 10, 20, 30, 40]);
        assert_eq!(median_length, 20 as u64);
    }

    // Range

    #[test]
    #[should_panic]
    fn test_read_length_range_empty() {
        let test_data: Vec<u64> = Vec::new();
        let _ = get_read_length_range(&test_data);
    }

    #[test]
    fn test_read_length_range() {
        let test_data = vec![10, 10, 20, 30];
        let (min_read_length, max_read_length) = get_read_length_range(&test_data);
        assert_eq!(min_read_length, 10);
        assert_eq!(max_read_length, 30);
    }

}