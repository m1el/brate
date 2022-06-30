set terminal png size 1920,1080
set output 'plot.png'
set xlabel 'time, seconds'
set ylabel 'bitrate, bits/s'
plot '20220628-weed-stream.dat' with lines
