use anyhow::Result;
use indoc::eprintdoc;
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::path::Path;
use thiserror::Error;

/// A collection of custom errors relating to the utility components for this package.
#[derive(Error, Debug, PartialEq)]
pub enum UtilityError {
    /// Indicates an invalid verbosity for summary output
    #[error("{0} is not a valid level of verbosity")]
    InvalidVerbosity(String),
}

// Adopted from Michael B. Hall - Rasusa (https://github.com/mbhall88/rasusa)

pub trait CompressionExt {
    fn from_path<S: AsRef<OsStr> + ?Sized>(p: &S) -> Self;
}

/// Attempts to infer the compression type from the file extension.
/// If the extension is not known, then Uncompressed is returned.
impl CompressionExt for niffler::compression::Format {
    fn from_path<S: AsRef<OsStr> + ?Sized>(p: &S) -> Self {
        let path = Path::new(p);
        match path.extension().map(|s| s.to_str()) {
            Some(Some("gz")) => Self::Gzip,
            Some(Some("bz") | Some("bz2")) => Self::Bzip,
            Some(Some("lzma")) => Self::Lzma,
            _ => Self::No,
        }
    }
}

/// ReadSet object
///
/// Read set objects are mutable to allow
/// sorting of read length and quality vectors
///
/// * `read_lengths` - a vector of read lengths
/// * `read_qualities` - a vector of read qualities
///
#[derive(Debug)]
pub struct ReadSet {
    read_lengths: Vec<u32>,
    read_qualities: Vec<f32>,
}

impl ReadSet {
    /// Create a new ReadSet instance
    ///
    /// Given the verctors of read lengths and
    /// qualities return a mutable ReadSet
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut read_set = ReadSet::new(
    ///     vec![10, 100, 1000], vec![10.0, 11.0, 12.0]
    /// )
    /// ```
    pub fn new(read_lengths: Vec<u32>, read_qualities: Vec<f32>) -> Self {
        ReadSet {
            read_lengths,
            read_qualities,
        }
    }
    /// Print a summary of the read set to stderr
    ///
    /// * `verbosity` - detail of summary message
    ///     * 0: standard output without headers
    ///     * 1: standard output with pretty headers
    ///     * 2: add length and quality thresholds
    ///     * 3: add top ranked read statistics
    ///
    /// * `top` - number of top ranking read lengths
    ///     and qualities to show in output
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut read_set = ReadSet::new(
    ///     !vec[10, 100, 1000], !vec[10.0, 11.0, 12.0]
    /// )
    /// read_set.summary(0, 3);
    /// ```
    pub fn summary(
        &mut self,
        verbosity: &u64,
        top: usize,
        header: bool,
    ) -> Result<(), UtilityError> {
        let length_range = self.range_length();

        match verbosity {
            &0 => {
                let head = match header {
                    true => "reads bases n50 longest shortest mean_length median_length mean_quality median_quality\n",
                    false => ""
                };

                eprintdoc! {
                    "{head}{reads} {bases} {n50} {longest} {shortest} {mean} {median} {meanq:.1} {medianq:.1}\n",
                    head = head,
                    reads = self.reads(),
                    bases = self.bases(),
                    n50 = self.n50(),
                    longest = length_range[1],
                    shortest = length_range[0],
                    mean = self.mean_length(),
                    median = self.median_length(),
                    meanq = self.mean_quality(),
                    medianq = self.median_quality(),
                }
                Ok(())
            }
            &1 | &2 | &3 => {
                eprintdoc! {"\n
                    Nanoq Read Summary
                    ====================
                    
                    Number of reads:      {reads}
                    Number of bases:      {bases}
                    N50 read length:      {n50}
                    Longest read:         {longest} 
                    Shortest read:        {shortest}
                    Mean read length:     {mean}
                    Median read length:   {median} 
                    Mean read quality:    {meanq:.2} 
                    Median read quality:  {medianq:.2}
                    ",
                    reads = self.reads(),
                    bases = self.bases(),
                    n50 = self.n50(),
                    longest = length_range[1],
                    shortest = length_range[0],
                    mean = self.mean_length(),
                    median = self.median_length(),
                    meanq = self.mean_quality(),
                    medianq = self.median_quality(),
                }
                if verbosity > &1 {
                    self.print_thresholds();
                }
                if verbosity > &2 {
                    self.print_ranking(top);
                }

                Ok(())
            }
            _ => Err(UtilityError::InvalidVerbosity(verbosity.to_string())),
        }
    }
    /// Get the number of reads
    ///
    /// # Example
    ///
    /// ```compile
    /// let actual = read_set.reads();
    /// let expected = 3;
    /// assert_eq!(actual, expected);
    /// ```
    pub fn reads(&self) -> u64 {
        self.read_lengths.len() as u64
    }
    /// Get the total number of bases
    ///
    /// # Example
    ///
    /// ```compile
    /// let actual = read_set.bases();
    /// let expected = 1110;
    /// assert_eq!(actual, expected);
    /// ```
    pub fn bases(&self) -> u64 {
        self.read_lengths
            .iter()
            .fold(0u64, |sum, i| sum + (*i as u64))
    }
    /// Get the range of read lengths
    ///
    /// # Example
    ///
    /// ```compile
    /// let actual = read_set.range_length();
    /// let expected = (10, 1000);
    /// assert_eq!(actual, expected);
    /// ```
    pub fn range_length(&self) -> [u32; 2] {
        let n_reads = self.reads();
        match n_reads.cmp(&1) {
            Ordering::Greater => [
                *self.read_lengths.iter().min().unwrap(),
                *self.read_lengths.iter().max().unwrap(),
            ],
            Ordering::Equal => [self.read_lengths[0], self.read_lengths[0]],
            Ordering::Less => [0, 0],
        }
    }
    /// Get the mean of read lengths
    ///
    /// # Example
    ///
    /// ```compile
    /// let actual = read_set.mean_length();
    /// let expected = 370;
    /// assert_eq!(actual, expected);
    /// ```
    pub fn mean_length(&self) -> u64 {
        let n_reads = self.reads();
        if n_reads > 0 {
            self.bases() / n_reads
        } else {
            0
        }
    }
    /// Get the median of read lengths
    ///
    /// # Example
    ///
    /// ```compile
    /// let actual = read_set.median_length();
    /// let expected = 100;
    /// assert_eq!(actual, expected);
    /// ```
    pub fn median_length(&mut self) -> u32 {
        let n_reads = self.reads();
        if n_reads == 0 {
            0
        } else {
            self.read_lengths.sort_unstable();
            let mid = n_reads / 2;
            if n_reads % 2 == 0 {
                (self.read_lengths[mid as usize - 1] + self.read_lengths[mid as usize]) / 2
            } else {
                self.read_lengths[mid as usize]
            }
        }
    }
    /// Get the N50 of read lengths
    ///
    /// # Example
    ///
    /// ```compile
    /// let actual = read_set.n50();
    /// let expected = 1000;
    /// assert_eq!(actual, expected);
    /// ```
    pub fn n50(&mut self) -> u64 {
        self.read_lengths.sort_unstable();
        self.read_lengths.reverse();
        let _stop = self.bases() / 2;
        let mut n50: u64 = 0;
        let mut _cum_sum: u64 = 0;
        for x in self.read_lengths.iter().map(|&i| i as u64) {
            _cum_sum += x;
            if _cum_sum >= _stop {
                n50 += x;
                break;
            }
        }
        n50
    }
    /// Get the mean of read qualities
    ///
    /// # Example
    ///
    /// ```compile
    /// let actual = read_set.mean_quality();
    /// let expected = 11.0;
    /// assert_eq!(actual, expected);
    /// ```
    pub fn mean_quality(&self) -> f32 {
        if !self.read_qualities.is_empty() {
            let qsum: f32 = self.read_qualities.iter().sum();
            qsum / self.read_qualities.len() as f32
        } else {
            f32::NAN
        }
    }
    /// Get the median of read qualities
    ///
    /// # Example
    ///
    /// ```compile
    /// let actual = read_set.median_quality();
    /// let expected = 11.0;
    /// assert_eq!(actual, expected);
    /// ```
    pub fn median_quality(&mut self) -> f32 {
        self.read_qualities
            .sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.read_qualities.len() / 2;
        if !self.read_qualities.is_empty() {
            if self.read_qualities.len() % 2 == 0 {
                (self.read_qualities[mid - 1] + self.read_qualities[mid]) / 2_f32
            } else {
                self.read_qualities[mid]
            }
        } else {
            f32::NAN
        }
    }
    /// Print read length and quality thresholds to stderr
    ///
    /// Used internally by the `summary` method. Creates
    /// an instance of the `ThresholdCounter` struct.
    fn print_thresholds(&self) {
        let mut thresholds = ThresholdCounter::new();
        let length_thresholds = thresholds.length(&self.read_lengths);
        let quality_thresholds = thresholds.quality(&self.read_qualities);
        let n_reads = self.reads();

        eprintdoc! {"\n
            Read length thresholds (bp)
            
            > 200       {l200:<12}      {lp200:04.1}%
            > 500       {l500:<12}      {lp500:04.1}%
            > 1000      {l1000:<12}      {lp1000:04.1}%
            > 2000      {l2000:<12}      {lp2000:04.1}%
            > 5000      {l5000:<12}      {lp5000:04.1}%
            > 10000     {l10000:<12}      {lp10000:04.1}%
            > 30000     {l30000:<12}      {lp30000:04.1}%
            > 50000     {l50000:<12}      {lp50000:04.1}%
            > 100000    {l100000:<12}      {lp100000:04.1}%
            > 1000000   {l1000000:<12}      {lp1000000:04.1}%
            ",
            l200=length_thresholds[0],
            l500=length_thresholds[1],
            l1000=length_thresholds[2],
            l2000=length_thresholds[3],
            l5000=length_thresholds[4],
            l10000=length_thresholds[5],
            l30000=length_thresholds[6],
            l50000=length_thresholds[7],
            l100000=length_thresholds[8],
            l1000000=length_thresholds[9],
            lp200=get_length_percent(length_thresholds[0], n_reads),
            lp500=get_length_percent(length_thresholds[1], n_reads),
            lp1000=get_length_percent(length_thresholds[2], n_reads),
            lp2000=get_length_percent(length_thresholds[3], n_reads),
            lp5000=get_length_percent(length_thresholds[4], n_reads),
            lp10000=get_length_percent(length_thresholds[5], n_reads),
            lp30000=get_length_percent(length_thresholds[6], n_reads),
            lp50000=get_length_percent(length_thresholds[7], n_reads),
            lp100000=get_length_percent(length_thresholds[8], n_reads),
            lp1000000=get_length_percent(length_thresholds[9], n_reads),
        }

        if !self.read_qualities.is_empty() {
            eprintdoc! {"\n
                Read quality thresholds (Q)
                
                > 5   {q5:<12}  {qp5:04.1}%
                > 7   {q7:<12}  {qp7:04.1}%
                > 10  {q10:<12}  {qp10:04.1}%
                > 12  {q12:<12}  {qp12:04.1}%
                > 15  {q15:<12}  {qp15:04.1}%
                > 20  {q20:<12}  {qp20:04.1}%
                > 25  {q25:<12}  {qp25:04.1}%
                > 30  {q30:<12}  {qp30:04.1}%
                \n\n",
                q5=quality_thresholds[0],
                q7=quality_thresholds[1],
                q10=quality_thresholds[2],
                q12=quality_thresholds[3],
                q15=quality_thresholds[4],
                q20=quality_thresholds[5],
                q25=quality_thresholds[6],
                q30=quality_thresholds[7],
                qp5=get_quality_percent(quality_thresholds[0], n_reads),
                qp7=get_quality_percent(quality_thresholds[1], n_reads),
                qp10=get_quality_percent(quality_thresholds[2], n_reads),
                qp12=get_quality_percent(quality_thresholds[3], n_reads),
                qp15=get_quality_percent(quality_thresholds[4], n_reads),
                qp20=get_quality_percent(quality_thresholds[5], n_reads),
                qp25=get_quality_percent(quality_thresholds[6], n_reads),
                qp30=get_quality_percent(quality_thresholds[7], n_reads),
            }
        } else {
            eprintln!("\n");
        }
    }
    /// Print top ranking read lengths and qualities to stderr
    ///
    /// Used internally by the summary method.
    fn print_ranking(&mut self, top: usize) {
        let max = match (self.reads() as usize) < top {
            true => self.reads() as usize,
            false => top,
        };

        self.read_lengths.sort_unstable();
        self.read_lengths.reverse();
        eprintln!("Top ranking read lengths (bp)\n");
        for i in 0..max {
            eprintln!("{}. {:<12}", i + 1, self.read_lengths[i]);
        }
        eprintln!("\n");

        if !self.read_qualities.is_empty() {
            self.read_qualities
                .sort_by(|a, b| b.partial_cmp(a).unwrap());
            eprintln!("Top ranking read qualities (Q)\n");
            for i in 0..max {
                eprintln!("{}. {:04.1}", i + 1, self.read_qualities[i]);
            }
            eprintln!("\n");
        }
    }
}

/// Count reads at defined length and quality thresholds
///
/// Used internally by the `print_thresholds` method.
struct ThresholdCounter {
    // read quality
    q5: u64,
    q7: u64,
    q10: u64,
    q12: u64,
    q15: u64,
    q20: u64,
    q25: u64,
    q30: u64,
    // read length
    l200: u64,
    l500: u64,
    l1000: u64,
    l2000: u64,
    l5000: u64,
    l10000: u64,
    l30000: u64,
    l50000: u64,
    l100000: u64,
    l1000000: u64,
}

impl ThresholdCounter {
    /// Create a new threshold counter
    ///
    /// Creates an instance of `ThresholdCounter`
    /// with internal threshold counts set to zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut counter = ThresholdCounter::new();
    /// ```
    fn new() -> Self {
        ThresholdCounter {
            q5: 0,
            q7: 0,
            q10: 0,
            q12: 0,
            q15: 0,
            q20: 0,
            q25: 0,
            q30: 0,
            l200: 0,
            l500: 0,
            l1000: 0,
            l2000: 0,
            l5000: 0,
            l10000: 0,
            l30000: 0,
            l50000: 0,
            l100000: 0,
            l1000000: 0,
        }
    }
    /// Get read quality threshold counts
    ///
    /// Returns a tuple of counts for eight
    /// average read quality thresholds (>=)
    ///
    /// * `read_qualities`: a vector of read qualities
    ///     obtained from the `NeedleCast` methods
    ///     `filter` or `filter_length`
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut counter = ThresholdCounter::new();
    /// let expected = [2, 1, 0, 0, 0, 0, 0, 0];
    /// let actual = counter.quality(&vec![5.0, 7.0, 10.0]);
    /// assert_eq!(actual, expected);
    /// ```
    fn quality(&mut self, read_qualities: &[f32]) -> [u64; 8] {
        for q in read_qualities.iter() {
            if q > &5.0 {
                self.q5 += 1
            }
            if q > &7.0 {
                self.q7 += 1
            }
            if q > &10.0 {
                self.q10 += 1
            }
            if q > &12.0 {
                self.q12 += 1
            }
            if q > &15.0 {
                self.q15 += 1
            }
            if q > &20.0 {
                self.q20 += 1
            }
            if q > &25.0 {
                self.q25 += 1
            }
            if q > &30.0 {
                self.q30 += 1
            }
        }
        [
            self.q5, self.q7, self.q10, self.q12, self.q15, self.q20, self.q25, self.q30,
        ]
    }
    /// Get read length threshold counts
    ///
    /// Returns a tuple of counts for ten
    /// read length thresholds (>=)
    ///
    /// * `read_lengths`: a vector of read lengths
    ///     obtained from the `NeedleCast` methods
    ///     `filter` or `filter_length`
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut counter = ThresholdCounter::new();
    /// let expected = (2, 1, 0, 0, 0, 0, 0, 0, 0, 0);
    /// let actual = counter.length(&vec![200, 500, 1000]);
    /// assert_eq!(actual, expected);
    /// ```
    fn length(&mut self, read_lengths: &[u32]) -> [u64; 10] {
        for l in read_lengths.iter() {
            if l > &200 {
                self.l200 += 1
            }
            if l > &500 {
                self.l500 += 1
            }
            if l > &1000 {
                self.l1000 += 1
            }
            if l > &2000 {
                self.l2000 += 1
            }
            if l > &5000 {
                self.l5000 += 1
            }
            if l > &10000 {
                self.l10000 += 1
            }
            if l > &30000 {
                self.l30000 += 1
            }
            if l > &50000 {
                self.l50000 += 1
            }
            if l > &100000 {
                self.l100000 += 1
            }
            if l > &1000000 {
                self.l1000000 += 1
            }
        }
        [
            self.l200,
            self.l500,
            self.l1000,
            self.l2000,
            self.l5000,
            self.l10000,
            self.l30000,
            self.l50000,
            self.l100000,
            self.l1000000,
        ]
    }
}

// utility function to get length threshold percent
fn get_length_percent(number: u64, n_reads: u64) -> f64 {
    (number as f64 / n_reads as f64) * 100.0
}

// utility function to get quality threshold percent
fn get_quality_percent(number: u64, n_reads: u64) -> f64 {
    (number as f64 / n_reads as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_format_from_path() {
        assert_eq!(niffler::Format::from_path("foo.gz"), niffler::Format::Gzip);
        assert_eq!(
            niffler::Format::from_path(Path::new("foo.gz")),
            niffler::Format::Gzip
        );
        assert_eq!(niffler::Format::from_path("baz"), niffler::Format::No);
        assert_eq!(niffler::Format::from_path("baz.fq"), niffler::Format::No);
        assert_eq!(
            niffler::Format::from_path("baz.fq.bz2"),
            niffler::Format::Bzip
        );
        assert_eq!(
            niffler::Format::from_path("baz.fq.bz"),
            niffler::Format::Bzip
        );
        assert_eq!(
            niffler::Format::from_path("baz.fq.lzma"),
            niffler::Format::Lzma
        );
    }

    #[test]
    fn threshold_counter_methods_ok() {
        let mut counter = ThresholdCounter::new();

        let exp_qual = [8, 7, 6, 5, 4, 3, 2, 1];
        let actual_qual = counter.quality(&[5.0, 7.0, 10.0, 12.0, 15.0, 20.0, 25.0, 30.0, 30.1]);

        assert_eq!(actual_qual, exp_qual);

        let exp_len = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        let actual_len = counter.length(&[
            200, 500, 1000, 2000, 5000, 10000, 30000, 50000, 100000, 1000000, 1000001,
        ]);

        assert_eq!(actual_len, exp_len);
    }

    #[test]
    fn percent_functions_ok() {
        use float_eq::float_eq;

        let plength = get_length_percent(3, 4);
        let pqual = get_quality_percent(3, 4);

        float_eq!(plength, 75.0, abs <= f64::EPSILON);
        float_eq!(pqual, 75.0, abs <= f64::EPSILON);
    }

    #[test]
    fn read_set_methods_ok() {
        use float_eq::float_eq;

        let mut read_set_even = ReadSet::new(vec![10, 1000], vec![10.0, 12.0]);

        assert_eq!(read_set_even.median_length(), 505);
        float_eq!(read_set_even.median_quality(), 11.0, abs <= f32::EPSILON);

        let mut read_set_odd = ReadSet::new(vec![10, 100, 1000], vec![10.0, 11.0, 12.0]);

        assert_eq!(read_set_odd.reads(), 3);
        assert_eq!(read_set_odd.bases(), 1110);
        assert_eq!(read_set_odd.range_length(), [10, 1000]);
        assert_eq!(read_set_odd.mean_length(), 370);
        assert_eq!(read_set_odd.median_length(), 100);
        assert_eq!(read_set_odd.n50(), 1000);
        float_eq!(read_set_odd.mean_quality(), 11.0, abs <= f32::EPSILON);
        float_eq!(read_set_odd.median_quality(), 11.0, abs <= f32::EPSILON);

        read_set_odd.print_thresholds();
        read_set_odd.print_ranking(3);
        read_set_odd.print_ranking(5);

        read_set_odd.summary(&0, 5, false).unwrap();
        read_set_odd.summary(&1, 5, false).unwrap();
        read_set_odd.summary(&2, 5, false).unwrap();
        read_set_odd.summary(&3, 5, false).unwrap();

        let error = read_set_odd.summary(&4, 5, false).unwrap_err();
        assert_eq!(error, UtilityError::InvalidVerbosity("4".to_string()));
    }

    #[test]
    fn read_set_methods_no_qual_ok() {
        let mut read_set_noqual = ReadSet::new(vec![10, 1000], vec![]);

        assert!(read_set_noqual.mean_quality().is_nan());
        assert!(read_set_noqual.median_quality().is_nan());

        read_set_noqual.print_thresholds();
        read_set_noqual.print_ranking(3);
        read_set_noqual.summary(&3, 3, false).unwrap();
    }

    #[test]
    fn read_set_methods_empty_ok() {
        let mut read_set_none = ReadSet::new(vec![], vec![]);
        assert_eq!(read_set_none.mean_length(), 0);
        assert_eq!(read_set_none.median_length(), 0);
        assert!(read_set_none.mean_quality().is_nan());
        assert!(read_set_none.median_quality().is_nan());
        assert_eq!(read_set_none.range_length(), [0, 0]);

        read_set_none.print_thresholds();
        read_set_none.print_ranking(3);
        read_set_none.summary(&3, 3, false).unwrap();
    }

    #[test]
    fn read_set_methods_one_ok() {
        use float_eq::float_eq;

        let mut read_set_none = ReadSet::new(vec![10], vec![8.0]);
        assert_eq!(read_set_none.mean_length(), 10);
        assert_eq!(read_set_none.median_length(), 10);
        float_eq!(read_set_none.mean_quality(), 8.0, abs <= f32::EPSILON);
        float_eq!(read_set_none.median_quality(), 8.0, abs <= f32::EPSILON);
        assert_eq!(read_set_none.range_length(), [10, 10]);

        read_set_none.print_thresholds();
        read_set_none.print_ranking(3);
        read_set_none.summary(&3, 3, false).unwrap();
    }
    // These tests are not testing for the correct stderr output,
    // does not seem possible with libtest at the moment:
    //  * https://github.com/rust-lang/rust/issues/42474
    //  * https://github.com/rust-lang/rust/issues/40298
    #[test]
    fn summary_output_ok() {
        use float_eq::float_eq;

        let mut read_set_none = ReadSet::new(vec![10], vec![8.0]);
        assert_eq!(read_set_none.mean_length(), 10);
        assert_eq!(read_set_none.median_length(), 10);
        float_eq!(read_set_none.mean_quality(), 8.0, abs <= f32::EPSILON);
        float_eq!(read_set_none.median_quality(), 8.0, abs <= f32::EPSILON);
        assert_eq!(read_set_none.range_length(), [10, 10]);

        read_set_none.print_thresholds();
        read_set_none.print_ranking(3);
        read_set_none.summary(&3, 3, false).unwrap();
    }

    #[test]
    fn summary_header_stderr_ok() {
        let mut read_set_none = ReadSet::new(vec![10], vec![8.0]);
        read_set_none.summary(&0, 3, true).unwrap();
    }
}
