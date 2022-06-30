// use std::{fs::File, io::Write};
use std::{env};

use ffmpeg::{
    codec::packet::Packet,
    format::{input},
    media::Type,
    format::stream::Stream,
};

pub struct StreamCounter {
    ty: Type,
    index: usize,
    base: f64,
    count: u64,
    pts: Option<i64>,
    key_count: u64,
    key_pts: Option<i64>,
}
impl StreamCounter {
    pub fn new(stream: Stream<'_>) -> Self {
        let decoder = stream.codec().decoder();
        let time_base = 0.001; //decoder.time_base();
        let ty = decoder.medium();
        // println!("[{}] ty={:?} base={:?}", stream.index(), ty, time_base);
        Self {
            ty,
            index: stream.index(),
            base: time_base,
            count: 0,
            pts: None,
            key_count: 0,
            key_pts: None,
        }
    }
    pub fn index(&self) -> usize { self.index } 
    pub fn consume(&mut self, packet: &Packet) {
        if let (true, Some(pts)) = (packet.is_key(), packet.pts()) {
            let bytes_delta = self.count - self.key_count;
            if let (true, Some(key_pts)) = (bytes_delta > 0, self.key_pts) {
                let pts_delta = pts - key_pts;
                let bps = 8.0 * bytes_delta as f64 / (pts_delta as f64 * self.base);
                let time = pts as f64 * self.base;
                println!("[{}] ty={:?} time={}, bps={}", self.index, self.ty, time, bps);
            }
            self.key_count = self.count;
            self.key_pts = Some(pts);
        }
        self.pts = packet.pts().or(self.pts);
        self.count += packet.size() as u64;
    }
    pub fn flush(&mut self) {
        let bytes_delta = self.count - self.key_count;
        if let (true, Some(pts), Some(key_pts)) = (bytes_delta > 0, self.pts, self.key_pts) {
            let pts_delta = pts - key_pts;
            if pts_delta > 0 {
                let bps = 8.0 * bytes_delta as f64 / (pts_delta as f64 * self.base);
                let time = pts as f64 * self.base;
                println!("[{}] ty={:?} time={}, bps={}", self.index, self.ty, time, bps);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg::init()
        .expect("couldn't load or init ffmpeg");
    let infile = env::args().nth(1)
        .expect("please specify input file as command line argument");

    let mut ictx = input(&infile)?;
    println!("{:?}", ictx.metadata());
    let mut counters = ictx.streams().map(|stream| {
        StreamCounter::new(stream)
    }).collect::<Vec<_>>();

    for res in ictx.packets() {
        let (stream, packet) = res?;
        if let Some(counter) = counters.iter_mut().find(|d| d.index() == stream.index()) {
            counter.consume(&packet);
        }
    }

    for mut counter in counters {
        counter.flush();
    }

    Ok(())
}
