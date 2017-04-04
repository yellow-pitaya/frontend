use std::io::prelude::*;

pub struct Redpitaya {
    socket: ::std::net::TcpStream,
    acquire_started: bool,
    generator_started: bool,
}

impl Redpitaya {
    pub fn new(ip: &str, port: u16) -> Redpitaya {
        let socket = match ::std::net::TcpStream::connect((ip, port)) {
            Ok(socket) => socket,
            Err(_) => panic!("Unable to connect to {}:{}", ip, port),
        };

        Redpitaya {
            socket: socket,
            acquire_started: false,
            generator_started: false,
        }
    }

    pub fn acquire_start(&mut self) {
        self.send("ACQ:START");
        self.acquire_started = true;
    }

    pub fn acquire_stop(&mut self) {
        self.send("ACQ:STOP");
        self.acquire_started = false;
    }

    pub fn acquire_is_started(&self) -> bool {
        self.acquire_started
    }

    pub fn acquire_reset(&mut self) {
        self.send("ACQ:RST");
    }

    pub fn trigger_set_level(&mut self, level: u8) {
        self.send(format!("ACQ:TRIG:LEV {}", level).as_str());
    }

    pub fn trigger_enable(&mut self, source: &str) {
        self.send(format!("ACQ:TRIG {}", source).as_str());
    }

    pub fn trigger_set_delay(&mut self, delay: u8) {
        self.send(format!("ACQ:TRIG:DLY {}", delay).as_str());
    }

    pub fn acquire_set_units(&mut self, unit: &str) {
        self.send(format!("ACQ:DATA:UNITS {}", unit).as_str());
    }

    pub fn acquire_set_decimation(&mut self, decimation: u8) {
        self.send(format!("ACQ:DEC {}", decimation).as_str());
    }

    pub fn acquire_get_decimation(&mut self) -> u8 {
        self.send("ACQ:DEC?");

        self.receive()
            .parse()
            .unwrap()
    }

    pub fn get_data(&mut self) -> String {
        self.send("ACQ:SOUR1:DATA?");

        self.receive()
    }

    pub fn generator_start(&mut self) {
        self.send("OUTPUT1:STATE ON");
        self.generator_started = true;
    }

    pub fn generator_stop(&mut self) {
        self.send("OUTPUT1:STATE OFF");
        self.generator_started = false;
    }

    pub fn generator_is_started(&self) -> bool {
        self.generator_started
    }

    pub fn generator_set_form(&mut self, form: &str) {
        self.send(format!("OUTPUT1:FUNC {}", form).as_str())
    }

    fn send(&mut self, command: &str) {
        info!("> {}", command);

        self.socket.write(
            format!("{}\r\n", command).as_bytes()
        );
    }

    fn receive(&mut self) -> String {
        let mut message = String::new();
        let mut reader = ::std::io::BufReader::new(self.socket.try_clone().unwrap());

        reader.read_line(&mut message);

        let message = message.trim_right_matches("\r\n");

        debug!("< {}", message);

        message.into()
    }
}
