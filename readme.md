Usage:

```
cargo run --release <video-file> > log.txt
awk 'match($0, /ty=Video time=(.*), bps=(.*)/, a) { print a[1]" "a[2] }' log.txt > video-rate.dat
gnuplot plot.plt
```

![example plot](assets/20220628-weed-stream.png)
