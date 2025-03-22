pub mod hourglass;

use clap::Parser;
use chrono::{NaiveDateTime, NaiveTime, TimeDelta, ParseResult};
use hourglass::Hourglass;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start of time range. (today)
    #[arg(long)]
    begin: Option<String>,

    /// End of time range. (if this is less than begin, it's interpreted to be tomorrow)
    #[arg(long)]
    end: Option<String>,

    /// Length of time range. (for example, 90s, 1m30s, 1y2d3h4m5s)
    #[arg(long)]
    length: Option<String>,

    /// Total width of the hourglass. (must be odd)
    #[arg(long, default_value_t = 7)]
    width: u32,

    /// Total height of the hourglass.
    #[arg(long, default_value_t = 12)]
    height: u32,

    /// Visual updates per second.
    #[arg(long, default_value_t = 20.0)]
    frames_per_sec: f64,

    /// Simulation updates per visual update. (10 frames per sec * 5 steps per frame = 50 steps per sec)
    #[arg(long, default_value_t = 2)]
    steps_per_frame: u32,

    /* TODO
    /// Whether to flip the hourglass over once the time is elapsed.
    #[arg(long, default_value_t = false)]
    repeat: bool,
    */

    /// How much of the hourglass to fill with sand. 0 is no sand, 1 is completely fully.
    #[arg(long, default_value_t = 0.75)]
    fullness: f32
}


fn parse_timestamp(timestamp: &str) -> ParseResult<NaiveDateTime> {
    Ok(NaiveDateTime::new(
        chrono::Local::now().naive_local().date(),
        NaiveTime::parse_from_str(timestamp, "%H:%M:%S").or_else(|_| {
            NaiveTime::parse_from_str(timestamp, "%H:%M")
        })?
    ))
}

fn parse_time(time: &str) -> Result<TimeDelta, &'static str> {
    fn try_parse_to_seconds(field: &str) -> Result<u64, &'static str> {
        let chars: Vec<char> = field.chars().collect();
        if chars.len() < 2 {
            return Err("time part must be at least 2 chars long");
        }

        let (unit, number_chars): (&char, &[char]) = chars.split_last().expect("expected vector with length of at least 2 to have a last element");
        let number: u64 = match number_chars.iter().collect::<String>().parse::<u64>() {
            Ok(x) => x,
            Err(_) => return Err("cannot parse time part number")
        };

        let multiplier = match *unit {
            's' => 1,
            'm' => 60,
            'h' => 60 * 60,
            'd' => 60 * 60 * 24,
            'y' => 60 * 60 * 24 * 365,
            _ => return Err("invalid time unit (valid units are s, d, h, d, and y)")
        };

        Ok(number * multiplier)
    }

    let mut total_seconds: u64 = 0;
    for field in time.split_inclusive(|ch: char| !ch.is_digit(10)) {
        total_seconds += try_parse_to_seconds(field)?;
    }

    Ok(TimeDelta::seconds(total_seconds.try_into().unwrap()))
}


struct TimeRange {
    start: NaiveDateTime,
    duration: TimeDelta
}

impl TimeRange {

    pub fn try_from_args(begin: Option<NaiveDateTime>, end: Option<NaiveDateTime>, length: Option<TimeDelta>) -> Result<TimeRange, &'static str> {
        let now = chrono::Local::now().naive_local();

        match (&begin, &end, &length) {
            (None, None, None) => Err("must define time range with some combination of `begin`, `end`, and `length`"),
            (None, None, Some(length)) => Ok(TimeRange {
                start: now,
                duration: *length
            }),
            (None, Some(end), None) => Ok(TimeRange {
                start: now,
                duration: *end - now
            }),
            (None, Some(end), Some(length)) => Ok(TimeRange {
                start: *end - *length,
                duration: *length
            }),
            (Some(_), None, None) => Err("must provide duration with `end` or `length`"),
            (Some(begin), None, Some(length)) => Ok(TimeRange {
                start: *begin,
                duration: *length
            }),
            (Some(begin), Some(end), None) => Ok(TimeRange {
                start: *begin,
                duration: if end > begin {
                    *end - *begin
                } else {
                    (*end + TimeDelta::days(1)) - *begin
                }
            }),
            (Some(begin), Some(end), Some(length)) => if (*end - *begin) == *length {
                Ok(TimeRange {
                    start: *begin,
                    duration: *length
                })
            } else {
                Err("`length` and `begin`..`end` must define the same duration")
            },
        }
    }

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let time_range = TimeRange::try_from_args(
        if let Some(begin_arg) = &args.begin { Some(parse_timestamp(begin_arg)?) } else { None },
        if let Some(end_arg) = &args.end { Some(parse_timestamp(end_arg)?) } else { None },
        if let Some(length_arg) = &args.length { Some(parse_time(length_arg)?) } else { None }
    )?;

    let mut glass = Hourglass::new(args.width.try_into().unwrap(), args.height.try_into().unwrap());
    glass.fill_with_sand_from_top(args.fullness / 2.0);
    glass.pinch();
    glass.settle_state(&mut rand::rng());

    loop {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // Clear and go to top left corner
        println!("{}", glass);

        let now = chrono::Local::now().naive_local();
        let elapsed = now - time_range.start;

        let time_progress: f64 = elapsed.num_milliseconds() as f64 / time_range.duration.num_milliseconds() as f64;

        let top_sand = glass.count_top_sand();
        let bottom_sand = glass.count_bottom_sand();
        let sand_progress: f64 = if top_sand + bottom_sand != 0 {
            bottom_sand as f64 / (top_sand + bottom_sand) as f64
        } else {
            0.0
        };

        if sand_progress < time_progress {
            glass.unpinch();
        } else {
            glass.pinch();
        }

        //println!("elapsed: {} sand: {} time: {}", elapsed, sand_progress, time_progress);
        //println!("begin: {} duration: {} now: {}", time_range.start.format("%H:%M:%S"), time_range.duration, now.format("%H:%M:%S"));

        // TODO stop simulating until next unpinch when steady state is reached
        // TODO catch up when behind time
        for _ in 0..args.steps_per_frame {
            glass.advance(&mut rand::rng());
        }

        std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / args.frames_per_sec as f64));
    }
}
