use indoc::eprintdoc;
use thiserror::Error;
use anyhow::Result;

/// A collection of custom errors relating to the utility components for this package.
#[derive(Error, Debug, PartialEq)]
pub enum UtilityError {

    /// Indicates an invalid verbosity for summary output
    #[error("{0} is not a valid level of verbosity")]
    InvalidVerbosity(String),

}

/// Create a read set object
///
/// Read set objects are mutable to allow
/// sorting of read length and quality vectors
#[derive(Debug)]
pub struct ReadSet {
    pub read_lengths: Vec<u32>,
    pub read_qualities: Vec<f32>,
}

impl ReadSet {
    /// Print a summary of the read set to stderr
    ///
    /// * `detail` - verbosity of summary message
    ///     * 0: standard output without headers
    ///     * 1: standard output with pretty headers
    ///     * 2: add length and quality thresholds
    ///     * 3: add top ranked read statistics
    ///
    /// * `top` - show top ranking lengths and qualities
    ///
    /// # Example
    /// 
    /// ```rust
    /// let mut read_set = ReadSet {
    ///     read_lengths: !vec[10, 100, 1000],
    ///     read_qualities: !vec[10.0, 11.0, 12.0],
    /// }
    /// read_set.summary(0, 3)
    /// read_set.summary(1, 3)
    /// read_set.summary(2, 3)
    /// read_set.summary(3, 3)
    /// ```
    pub fn summary(&mut self, verbosity: &u8, top: usize) -> Result<(), UtilityError> {

        let (min_length, max_length) = self.range_length();

        match verbosity {
            0 => {
                eprintdoc! {
                    "{reads} {bases} {n50} {longest} {shortest} {mean} {median} {meanq:.1} {medianq:.1}\n",
                    reads = self.reads(),
                    bases = self.bases(),
                    n50 = self.n50(),
                    longest = max_length,
                    shortest = min_length,
                    mean = self.mean_length(),
                    median = self.median_length(),
                    meanq = self.mean_quality(),
                    medianq = self.median_quality(),
                }
                Ok(())
            },
            1 | 2 | 3 => {
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
                    longest = max_length,
                    shortest = min_length,
                    mean = self.mean_length(),
                    median = self.median_length(),
                    meanq = self.mean_quality(),
                    medianq = self.median_quality(),
                }
                if *verbosity > 1  {
                    self.print_thresholds();
                } 
                if *verbosity > 2 {
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
    pub fn reads(&self) -> u32 {
        self.read_lengths.len() as u32
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
        self.read_lengths.iter().fold(0u64, |sum, i| sum + (*i as u64))
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
    pub fn range_length(&self) -> (u32, u32) {
        (*self.read_lengths.iter().min().unwrap(), *self.read_lengths.iter().max().unwrap())
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
        self.bases() / self.reads() as u64
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
        self.read_lengths.sort();
        let mid = self.reads() / 2;
        if self.reads() % 2 == 0 {
            (self.read_lengths[mid as usize - 1] + self.read_lengths[mid as usize]) / 2 as u32
        } else {
            self.read_lengths[mid as usize]
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
        self.read_lengths.sort();
        self.read_lengths.reverse();
        let _stop = self.bases() / 2;
        let mut n50: u64 = 0;
        let mut _cum_sum: u64 = 0;
        for x in self.read_lengths.iter().map(|&i| i as u64){
            _cum_sum += x;
            if _cum_sum >= _stop {
                n50 += x;
                break
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
        let qsum: f32 = self.read_qualities.iter().sum();
        qsum / self.read_qualities.len() as f32
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
        self.read_qualities.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.read_qualities.len() / 2;
        if self.read_qualities.len() > 0 {
            if self.read_qualities.len() % 2 == 0 {
                (self.read_qualities[mid - 1] + self.read_qualities[mid]) / 2 as f32
            } else {
                self.read_qualities[mid]
            }
        } else {
            f32::NAN
        }
        
    }

    fn print_thresholds(&self){
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
            l200=length_thresholds.0,
            l500=length_thresholds.1,
            l1000=length_thresholds.2,
            l2000=length_thresholds.3,
            l5000=length_thresholds.4,
            l10000=length_thresholds.5,
            l30000=length_thresholds.6,
            l50000=length_thresholds.7,
            l100000=length_thresholds.8,
            l1000000=length_thresholds.9,
            lp200=(length_thresholds.0 as f64 / n_reads as f64)*100.0,
            lp500=(length_thresholds.1 as f64 / n_reads as f64)*100.0,
            lp1000=(length_thresholds.2 as f64 / n_reads as f64)*100.0,
            lp2000=(length_thresholds.3 as f64 / n_reads as f64)*100.0,
            lp5000=(length_thresholds.4 as f64 / n_reads as f64)*100.0,
            lp10000=(length_thresholds.5 as f64 / n_reads as f64)*100.0,
            lp30000=(length_thresholds.6 as f64 / n_reads as f64)*100.0,
            lp50000=(length_thresholds.7 as f64 / n_reads as f64)*100.0,
            lp100000=(length_thresholds.8 as f64 / n_reads as f64)*100.0,
            lp1000000=(length_thresholds.9 as f64 / n_reads as f64)*100.0
        }

        if self.read_qualities.len() > 0 {
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
                q5=quality_thresholds.0,
                q7=quality_thresholds.1,
                q10=quality_thresholds.2,
                q12=quality_thresholds.3,
                q15=quality_thresholds.4,
                q20=quality_thresholds.5,
                q25=quality_thresholds.6,
                q30=quality_thresholds.7,
                qp5=(quality_thresholds.0 as f64 / n_reads as f64)*100.0,
                qp7=(quality_thresholds.1 as f64 / n_reads as f64)*100.0,
                qp10=(quality_thresholds.2 as f64 / n_reads as f64)*100.0,
                qp12=(quality_thresholds.3 as f64 / n_reads as f64)*100.0,
                qp15=(quality_thresholds.4 as f64 / n_reads as f64)*100.0,
                qp20=(quality_thresholds.5 as f64 / n_reads as f64)*100.0,
                qp25=(quality_thresholds.6 as f64 / n_reads as f64)*100.0,
                qp30=(quality_thresholds.7 as f64 / n_reads as f64)*100.0,
            }
        } else {
            eprintln!("\n");
        }

    }

    fn print_ranking(&mut self, top: usize){

        self.read_lengths.sort();
        self.read_lengths.reverse();
        eprintln!("Top ranking read lengths (bp)\n");
        for i in 0..top {
            eprintln!("{}. {:<12}", i+1, self.read_lengths[i]);
        }
        eprintln!("\n");
        
        if self.read_qualities.len() > 0 {
            self.read_qualities.sort_by(|a, b| b.partial_cmp(a).unwrap());
            eprintln!("Top ranking read qualities (Q)\n");
            for i in 0..top {
                eprintln!("{}. {:04.1}", i+1, self.read_qualities[i]);
            }
            eprintln!("\n");
        }

    }

}

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
    fn quality(&mut self, read_qualities: &Vec<f32>) -> (u64, u64, u64, u64, u64, u64, u64, u64) {

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
        (self.q5, self.q7, self.q10, self.q12, self.q15, self.q20, self.q25, self.q30)
    }

    fn length(&mut self, read_lengths: &Vec<u32>) -> (u64, u64, u64, u64, u64, u64, u64, u64, u64, u64) {

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
        (self.l200, self.l500, self.l1000, self.l2000, self.l5000, self.l10000, self.l30000, self.l50000, self.l100000, self.l1000000)
    }
}
