# Run stats task benchmarks on an idle system.
#
# When we ran tools in sequence within the benchmarking script from previous versions
# `rust-bio-tools` and `nanoq -f -s` were slowed down. This was likely because of high memory
# usage of some tools and need for re-caching in subsequent iterations. We avoid this by running
# each tool in its own execution loop that includes a pre-cautionary 'cold-start' iteration, not
# considered in the benchmark evaluation (10 replicates).
#
# Compressed and uncompressed loops to align with distinct commands for RBT (but not others)


# --> NANOQ FAST <--

# uncompressed
for f in /data/*.fq; do
    for i in {1..11}; do 
        t nanoq --input $f --min-len 5000 --fast > /dev/null 2> benchmark
        tail -1 benchmark >> nanoqf_filt_fq
    done
done

# compressed
for f in /data/*.fq.gz; do
    for i in {1..11}; do 
        t nanoq --input $f --min-len 5000 --fast > /dev/null 2> benchmark 
        tail -1 benchmark >> nanoqf_filt_gz
    done
done

# --> NANOQ <--

# uncompressed
for f in /data/*.fq; do
    for i in {1..11}; do 
        t nanoq --input $f --min-len 5000 > /dev/null 2> benchmark 
        tail -1 benchmark >> nanoq_filt_fq
    done
done

# compressed
for f in /data/*.fq.gz; do
    for i in {1..11}; do 
        t nanoq --input $f --min-len 5000 > /dev/null 2> benchmark 
        tail -1 benchmark >> nanoq_filt_gz
    done
done

# --> FILTLONG <--

# uncompressed
for f in /data/*.fq; do 
    for i in {1..11}; do
        t filtlong --min_length 5000 $f > /dev/null 2> benchmark
        tail -1 benchmark >> filtlong_filt_fq
    done
done

# compressed
for f in /data/*.fq; do 
    for i in {1..11}; do
        t filtlong --min_length 5000 $f > /dev/null 2> benchmark
        tail -1 benchmark >> filtlong_filt_gz
    done
done


# --> NANOFILT <--

# uncompressed
for f in /data/*.fq; do 
    for i in {1..11}; do
        t NanoFilt --length 5000 $f > /dev/null 2> benchmark 
        tail -1 benchmark >> nanofilt_filt_fq
    done
done

# compressed
for f in /data/*.fq.gz; do 
    for i in {1..11}; do
        (t gunzip -c $f| NanoFilt --length 5000 > /dev/null) 2> benchmark 
        tail -1 benchmark >> nanofilt_filt_gz
    done
done
