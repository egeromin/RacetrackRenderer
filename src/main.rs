extern crate pxl;
use pxl::*;


const RACETRACK: &[u8] = include_bytes!("../racetrack.values");
#[derive(Copy,Clone)]
enum TrackPoint {
    Boundary,
    End,
    Beginning,
    Empty,
}

use TrackPoint::*;


impl TrackPoint {

    fn from_byte(byte:u8) -> TrackPoint {
        match byte {
            0 => Boundary,
            1 => End,
            2 => Beginning,
            3 => Empty,
            _ => panic!("No phallic racetrack: got :{}", byte),
        }
    }

    fn pixel(self) -> Pixel {
        match self {
            Boundary => Pixel{red:0,green:0,blue:0},
            End => Pixel{red:0,green:255,blue:0},
            Beginning => Pixel{red:255,green:0,blue:0},
            Empty => Pixel{red:255,green:255,blue:255},
        }
    }
}

struct Game {
    track: Vec<TrackPoint>,
}

impl Program for Game {
    fn new() -> Game {
        assert_eq!(RACETRACK.len(), 2500);
        let track = RACETRACK.iter().cloned().map(TrackPoint::from_byte).collect();
        Game{track}
    }

    fn render(&mut self, pixels: &mut [Pixel]) {
        for x in 0..50 {
            for y in 0..50 {
                let pixel = self.track[y*50 + x].pixel();
                for i in 0..5 {
                    for j in 0..5 {
                        pixels[(5*y+j+3)*256 + 5*x + i+3] = pixel;
                    }
                }
            }
        }
    }
}

fn main() {
    run::<Game>();
}
