mod gdbson;

use std::{
    fmt::{Arguments, Display, Write as FmtWrite},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    process::{ChildStdin, ChildStdout, Command, Stdio},
    sync::mpsc::{channel, Receiver, Sender},
};

fn reader_thread(reader: ChildStdout, sender: Sender<String>) {
    let mut reader = BufReader::new(reader);
    let mut s = String::new();

    loop {
        s.clear();
        reader.read_line(&mut s).unwrap();

        sender.send(s.trim().to_string()).unwrap();
    }
}

macro_rules! w {
    ($dst:expr, $($arg:tt)*) => {
        $dst.write_to_gdb("out", std::format_args!($($arg)*))
    };
}

pub struct GDB {
    writer: BufWriter<ChildStdin>,
    log: BufWriter<File>,
    receiver: Receiver<String>,
}

impl GDB {
    pub fn new(program: &str) -> GDB {
        let child = Command::new("gdb")
            .args([program, "--interpreter", "mi"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let writer = child.stdin.unwrap();
        let reader = child.stdout.unwrap();

        let (sender, receiver) = channel();
        std::thread::spawn(|| reader_thread(reader, sender));

        let gdb = GDB {
            writer: BufWriter::new(writer),
            log: BufWriter::new(File::create("gdb_log.txt").unwrap()),
            receiver,
        };

        gdb
    }

    pub fn breakpoint(&mut self, file: &str, line: u32) {
        w!(self, "b {}:{}", file, line);
    }

    pub fn run(&mut self) {
        w!(self, "r");
    }

    pub fn register_names(&mut self) -> RegisterNames {
        w!(self, "-data-list-register-names");
        // sync op, so hopefully it will be the next thing?
        let message = self.recv().unwrap();

        match message {
            Message::RegisterNames(names) => RegisterNames { names },
            _ => todo!("{:?}", message),
        }
    }

    fn notify_async_output(&mut self, line: &str) -> Message {
        let comma = line.find(',').unwrap();
        let command = &line[..comma];
        // let rest = &line[comma+1..];

        match command {
            "breakpoint-created" => Message::BreakpointCreated,
            "thread-group-added"
            | "thread-group-started"
            | "thread-created"
            | "breakpoint-modified" => Message::Other,
            _ => todo!("{}", command),
        }
    }
    fn exec_async_output(&mut self, line: &str) -> Message {
        let comma = line.find(',').unwrap();
        let command = &line[..comma];
        let rest = &line[comma + 1..];

        match command {
            "running" => Message::Other,
            "stopped" => {
                let value = gdbson::parse(rest);
                let value = value.as_map();
                let reason: &str = (&value["reason"]).try_into().unwrap();

                match reason {
                    "breakpoint-hit" => Message::BreakpointHit,
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }

    fn sync_operation(&mut self, line: &str) -> Option<Message> {
        let comma = line.find(',').unwrap();
        let command = &line[..comma];
        let rest = &line[comma + 1..];

        match command {
            "done" => {}
            "running" | "error" => return None,
            _ => unreachable!(),
        }

        let value = gdbson::parse(rest);
        let value = value.as_map();
        let value = &value["register-names"];
        let value = value.as_list();

        let registers = value
            .into_iter()
            .map(|x| x.as_string().to_string())
            .collect();

        Some(Message::RegisterNames(registers))
    }

    fn recv_impl(&mut self, wait: bool) -> Option<Message> {
        loop {
            let all = if wait {
                self.receiver.recv().ok()
            } else {
                self.receiver.try_recv().ok()
            }?;

            self.write_log("in ", format_args!("{}", all));

            let line = &all[1..];

            if all == "(gdb)" {
                // ??
                continue;
            }

            match all.as_bytes()[0] {
                b'~' => {
                    assert!(matches!(line.as_bytes(), [b'\"', .., b'\"']), "{}", line);
                    let line = &line[1..line.len() - 1];
                    print!("{}", unescape_c(line));
                }
                b'=' => {
                    return Some(self.notify_async_output(line));
                }
                b'^' => {
                    return self.sync_operation(line);
                }
                b'*' => {
                    return Some(self.exec_async_output(line));
                }
                b'&' => {
                    self.write_log("log", format_args!("{}", unescape_c(line)));
                }
                _ => todo!("{}", all),
            }
        }
    }

    pub fn recv_async(&mut self) -> Option<Message> {
        self.recv_impl(false)
    }

    pub fn recv(&mut self) -> Option<Message> {
        self.recv_impl(true)
    }

    fn write_to_gdb(&mut self, d: &str, args: Arguments) {
        writeln!(self.log, "{}|\t{}", d, args).unwrap();

        writeln!(self.writer, "{}", args).unwrap();
        self.writer.flush().unwrap();
    }
    fn write_log(&mut self, d: &str, args: Arguments) {
        writeln!(self.log, "{}|\t{}", d, args).unwrap();
        self.log.flush().unwrap();
    }
}

#[derive(Debug)]
pub enum Message {
    BreakpointCreated,
    BreakpointHit,
    RegisterNames(Vec<String>),
    Other,
}

pub struct RegisterNames {
    names: Vec<String>,
}

pub trait Debuggable {
    fn init(&mut self, gdb: &mut GDB);
}

fn unescape_c(input: &str) -> UnescapeC {
    UnescapeC { s: input }
}

struct UnescapeC<'x> {
    s: &'x str,
}
impl<'x> Display for UnescapeC<'x> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unescape = |c| match c {
            'n' => '\n',
            't' => '\t',
            '\\' => '\\',
            '\"' => '\"',
            _ => todo!("{}", c),
        };
        let mut was_escape = false;
        for c in self.s.chars() {
            match (was_escape, c) {
                (true, _) => {
                    let new = unescape(c);
                    f.write_char(new)?;
                    was_escape = false;
                }
                (false, '\\') => {
                    was_escape = true;
                }
                _ => {
                    f.write_char(c)?;
                }
            }
        }

        Ok(())
    }
}

/*

All output sequences end in a single line containing a period.
The token is from the corresponding request. If an execution command is interrupted by the `-exec-interrupt' command, the token associated with the `*stopped' message is the one of the original execution command, not the one of the interrupt command.
status-async-output contains on-going status information about the progress of a slow operation. It can be discarded. All status output is prefixed by `+'.
exec-async-output contains asynchronous state change on the target (stopped, started, disappeared). All async output is prefixed by `*'.
notify-async-output contains supplementary information that the client should handle (e.g., a new breakpoint information). All notify output is prefixed by `='.
console-stream-output is output that should be displayed as is in the console. It is the textual response to a CLI command. All the console output is prefixed by `~'.
target-stream-output is the output produced by the target program. All the target output is prefixed by `@'.
log-stream-output is output text coming from GDB's internals, for instance messages that should be displayed as part of an error log. All the log output is prefixed by `&'.
New GDB/MI commands should only output lists containing values.

*/
