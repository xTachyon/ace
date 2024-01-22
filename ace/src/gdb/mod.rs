mod gdbson;

use std::{
    collections::VecDeque,
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

#[derive(PartialEq, Eq)]
enum MessageKind {
    Sync,
    Async,
}

struct MessageInfo {
    message: Message,
    kind: MessageKind,
}

pub struct GDB {
    writer: BufWriter<ChildStdin>,
    log: BufWriter<File>,
    receiver: Receiver<String>,
    queue: VecDeque<MessageInfo>,
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
            queue: VecDeque::new(),
        };

        gdb
    }

    pub fn breakpoint(&mut self, file: &str, line: u32) {
        w!(self, "b {}:{}", file, line);
    }
    pub fn breakpoint_fn(&mut self, fun: &str) {
        w!(self, "b {}", fun);
    }

    pub fn step(&mut self) {
        w!(self, "s");
    }

    pub fn run(&mut self) {
        w!(self, "r");
    }

    pub fn register_names(&mut self) -> RegisterNames {
        w!(self, "-data-list-register-names");
        // sync op, so hopefully it will be the next thing?
        // return RegisterNames { names: Vec::new() };
        let message = self.recv_sync_message();

        match message {
            Message::RegisterNames(names) => RegisterNames { inner: names },
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
                    "end-stepping-range" => Message::EndSteppingRange,
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }

    fn sync_operation(&mut self, line: &str) -> Message {
        let (command, rest) = split_comma(line);

        match command {
            "done" | "running" => {}
            "error" => return Message::Other,
            _ => unreachable!(),
        }

        if rest.is_empty() {
            return Message::Other;
        }

        let value = gdbson::parse(rest);
        let value = value.as_map();
        let value = &value["register-names"];
        let value = value.as_list();

        let registers = value
            .into_iter()
            .map(|x| x.as_string().to_string())
            .collect();

        Message::RegisterNames(registers)
    }

    fn recv_deserialize(&mut self, wait: bool) -> Option<(Message, MessageKind)> {
        let result = loop {
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
                b'^' => {
                    break (self.sync_operation(line), MessageKind::Sync);
                }
                b'=' => {
                    break (self.notify_async_output(line), MessageKind::Async);
                }
                b'*' => {
                    break (self.exec_async_output(line), MessageKind::Async);
                }
                b'~' => {
                    assert!(matches!(line.as_bytes(), [b'\"', .., b'\"']), "{}", line);
                    let line = &line[1..line.len() - 1];
                    print!("{}", unescape_c(line));
                }
                b'&' => {
                    self.write_log("log", format_args!("{}", unescape_c(line)));
                }
                _ => todo!("{}", all),
            };
        };
        Some(result)
    }

    fn recv_impl(&mut self, wait: bool) -> Option<Message> {
        if let Some(first) = self.queue.pop_front() {
            return Some(first.message);
        }
        Some(self.recv_deserialize(wait)?.0)
    }

    pub fn recv_async(&mut self) -> Option<Message> {
        self.recv_impl(false)
    }

    pub fn recv(&mut self) -> Option<Message> {
        self.recv_impl(true)
    }

    fn recv_sync_message(&mut self) -> Message {
        if let Some(index) = self.queue.iter().position(|x| x.kind == MessageKind::Sync) {
            let msg = self.queue.remove(index).expect("message must exist");
            return msg.message;
        }

        loop {
            let (msg, kind) = self.recv_deserialize(true).unwrap();
            if kind == MessageKind::Sync {
                return msg;
            }

            self.queue.push_back(MessageInfo { message: msg, kind });
        }
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

fn split_comma(line: &str) -> (&str, &str) {
    match line.find(',') {
        Some(comma) => (&line[..comma], &line[comma + 1..]),
        None => (line, ""),
    }
}

#[derive(Debug)]
pub enum Message {
    BreakpointCreated,
    BreakpointHit,
    EndSteppingRange,
    RegisterNames(Vec<String>),
    Other,
}

pub struct RegisterNames {
    pub inner: Vec<String>,
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
