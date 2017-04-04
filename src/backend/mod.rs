use std::io::prelude::*;

pub struct Socket {
    socket: ::std::net::TcpStream,
}

impl Socket {
    pub fn new(ip: &str, port: u16) -> Self {
        let socket = match ::std::net::TcpStream::connect((ip, port)) {
            Ok(socket) => socket,
            Err(_) => panic!("Unable to connect to {}:{}", ip, port),
        };

        Socket {
            socket: socket,
        }
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

impl Clone for Socket {
    fn clone(&self) -> Self {
        Socket {
            socket: self.socket.try_clone().unwrap(),
        }
    }
}

pub struct Acquire {
    socket: Socket,
    started: bool,
}

impl Acquire {
    pub fn new(socket: Socket) -> Self {
        Acquire {
            socket: socket,
            started: false,
        }
    }

    pub fn start(&mut self) {
        self.socket.send("ACQ:START");
        self.started = true;
    }

    pub fn stop(&mut self) {
        self.socket.send("ACQ:STOP");
        self.started = false;
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn reset(&mut self) {
        self.socket.send("ACQ:RST");
    }

    pub fn set_units(&mut self, unit: &str) {
        self.socket.send(format!("ACQ:DATA:UNITS {}", unit).as_str());
    }

    pub fn set_decimation(&mut self, decimation: u8) {
        self.socket.send(format!("ACQ:DEC {}", decimation).as_str());
    }

    pub fn get_decimation(&mut self) -> u8 {
        self.socket.send("ACQ:DEC?");

        self.socket.receive()
            .parse()
            .unwrap()
    }

    pub fn get_data(&mut self) -> String {
        self.socket.send("ACQ:SOUR1:DATA?");

        self.socket.receive()
    }
}

pub struct Generator {
    socket: Socket,
    started: bool,
}

impl Generator {
    pub fn new(socket: Socket) -> Self {
        Generator {
            socket: socket,
            started: false,
        }
    }

    pub fn start(&mut self) {
        self.socket.send("OUTPUT1:STATE ON");
        self.started = true;
    }

    pub fn stop(&mut self) {
        self.socket.send("OUTPUT1:STATE OFF");
        self.started = false;
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn set_form(&mut self, form: &str) {
        self.socket.send(format!("OUTPUT1:FUNC {}", form).as_str())
    }
}

pub struct Trigger {
    socket: Socket,
}

impl Trigger {
    pub fn new(socket: Socket) -> Self {
        Trigger {
            socket: socket,
        }
    }

    pub fn set_level(&mut self, level: u8) {
        self.socket.send(format!("ACQ:TRIG:LEV {}", level).as_str());
    }

    pub fn enable(&mut self, source: &str) {
        self.socket.send(format!("ACQ:TRIG {}", source).as_str());
    }

    pub fn set_delay(&mut self, delay: u8) {
        self.socket.send(format!("ACQ:TRIG:DLY {}", delay).as_str());
    }
}

pub struct Redpitaya {
    pub acquire: Acquire,
    pub generator: Generator,
    pub trigger: Trigger,
}

impl Redpitaya {
    pub fn new(ip: &str, port: u16) -> Redpitaya {
        let socket = Socket::new(ip, port);

        Redpitaya {
            acquire: Acquire::new(socket.clone()),
            generator: Generator::new(socket.clone()),
            trigger: Trigger::new(socket.clone()),
        }
    }
}
