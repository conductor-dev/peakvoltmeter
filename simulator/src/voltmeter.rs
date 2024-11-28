use crate::{error::ConductorSimResult, Command, Input};
use serde::Deserialize;
use std::{
    net::UdpSocket,
    time::{Duration, Instant},
};

#[derive(Debug, Deserialize)]
struct Row {
    sample: i32,
}

fn read_csv(input: Input, delimiter: u8) -> ConductorSimResult<Vec<Row>> {
    let mut builder = csv::ReaderBuilder::new();
    let delimiter = builder.delimiter(delimiter);

    match input {
        Input::Stdin => Ok(delimiter
            .from_reader(std::io::stdin())
            .deserialize()
            .collect::<Result<_, csv::Error>>()?),
        Input::Path(path) => Ok(delimiter
            .from_path(path)?
            .deserialize()
            .collect::<Result<_, csv::Error>>()?),
    }
}

pub fn voltmeter(command: Command) -> ConductorSimResult<()> {
    let records = read_csv(command.file, command.delimiter)?;

    let stream = UdpSocket::bind("127.0.0.1:0")?;

    let seconds_per_sample = Duration::from_secs_f64(1.0 / (command.sample_rate as f64));
    let mut last_time = Instant::now();

    for record in records {
        stream.send_to(&record.sample.to_ne_bytes(), &command.target)?;

        while last_time.elapsed() < seconds_per_sample {
            // TODO: Maybe use
            // std::hint::spin_loop();
            // or
            // thread::yield_now();
        }
        last_time += seconds_per_sample;
    }

    Ok(())
}
