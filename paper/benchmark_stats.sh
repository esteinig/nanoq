# Run stats task benchmarks on an idle system.
#
# When we ran tools in sequence within the benchmarking script from previous versions
# `rust-bio-tools` and `nanoq -f -s` were slowed down. This was likely because of high memory
# usage of some tools and need for re-caching in subsequent iterations. We avoid this by running
# each tool in its own execution loop that includes a pre-cautionary 'cold-start' iteration, not
# considered in the benchmark evaluation (10 replicates).
#
# Compressed and uncompressed loops to align with distinct commands for RBT (but not others)


# --> RUST-BIO-TOOLS <--

# uncompressed
for f in /data/*.fq; do 
    for i in {1..11}; do
        t rbt sequence-stats -q < $f 2> benchmark; tail -1 benchmark >> rbt_stat_fq
    done
done

# compressed
for f in /data/*.fq.gz; do 
    for i in {1..11}; do
        (t zcat $f | rbt sequence-stats -q) 2> benchmark; tail -1 benchmark >> rbt_stat_fq.gz
    done
done

# --> NANOQ FAST <--

# uncompressed
for f in /data/*.fq; do
    for i in {1..11}; do 
        t nanoq -f -s -i $f 2> benchmark; tail -1 benchmark >> nanoqf_stat_fq
    done
done

# compressed
for f in /data/*.fq.gz; do
    for i in {1..11}; do 
        t nanoq -f -s -i $f 2> benchmark; tail -1 benchmark >> nanoqf_stat_fq.gz
    done
done

# --> NANOQ <--

# uncompressed
for f in /data/*.fq; do
    for i in {1..11}; do 
        t nanoq -s -i $f 2> benchmark; tail -1 benchmark >> nanoqf_stat_fq
    done
done

# compressed
for f in /data/*.fq.gz; do
    for i in {1..11}; do 
        t nanoq -s -i $f 2> benchmark; tail -1 benchmark >> nanoqf_stat_fq.gz
    done
done

# --> NANOSTAT T1 <--

# uncompressed
for f in /data/*.fq; do 
    for i in {1..11}; do
        t NanoStat --fastq $f --threads 1 2> benchmark; tail -1 benchmark >> nanostat_stat_fq
    done
done

# compressed
for f in /data/*.fq.gz; do 
    for i in {1..11}; do
        t NanoStat --fastq $f --threads 1 2> benchmark; tail -1 benchmark >> nanostat_stat_fq.gz
    done
done

# --> NANOSTAT T8 <--

# uncompressed
for f in /data/*.fq; do 
    for i in {1..11}; do
        t NanoStat --fastq $f --threads 8 2> benchmark; tail -1 benchmark >> nanostat_stat_fq
    done
done

# compressed
for f in /data/*.fq.gz; do 
    for i in {1..11}; do
        t NanoStat --fastq $f --threads 8 2> benchmark; tail -1 benchmark >> nanostat_stat_fq.gz
    done
done