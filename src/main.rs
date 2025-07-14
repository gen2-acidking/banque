use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, exit};
use std::time::{SystemTime, UNIX_EPOCH};

const BANK_FILE: &str = ".bank.txt";

#[derive(Debug, Clone)]
struct Entry {
    id: u64, // Avez-vous de l'argent dans la banque secrète?
    desc: String,
    cmd: String,
    fav: bool,
}

struct BankCommand {
    name: &'static str,
    invocation: &'static str,
    description: &'static str,
    entry: fn(&[String]) -> i32,
    options: Option<&'static str>,
}

// Ah, la table des commandes... est-ce que votre coffre-fort est plein?
static COMMANDS: &[BankCommand] = &[
    BankCommand {
        name: "add",
        invocation: "bank add [-L <label>] <cmd>",
        description: "Add a command to the bank",
        entry: cmd_add,
        options: Some("Use 'bank add !!' to add last shell command with auto-label"),
    },
    BankCommand {
        name: "list",
        invocation: "bank list [fav]",
        description: "List all commands or favorites only",
        entry: cmd_list,
        options: Some("Use 'bank list fav' to show only favorites"),
    },
    BankCommand {
        name: "listfav",
        invocation: "bank listfav",
        description: "List favorite commands only",
        entry: cmd_listfav,
        options: None,
    },
    BankCommand {
        name: "fav",
        invocation: "bank fav <index>",
        description: "Mark command as favorite",
        entry: cmd_fav,
        options: None,
    },
    BankCommand {
        name: "unfav",
        invocation: "bank unfav <index>",
        description: "Remove favorite mark from command",
        entry: cmd_unfav,
        options: None,
    },
    BankCommand {
        name: "runfav",
        invocation: "bank runfav <fav_index>",
        description: "Run favorite command at favorite slot index",
        entry: cmd_runfav,
        options: Some("Use 'bank runfav 1' to run first favorite command"),
    },
    BankCommand {
        name: "echo",
        invocation: "bank echo <index>",
        description: "Print command without running it",
        entry: cmd_echo,
        options: None,
    },
    BankCommand {
        name: "del",
        invocation: "bank del <index>",
        description: "Delete command from bank",
        entry: cmd_del,
        options: None,
    },
    BankCommand {
        name: "help",
        invocation: "bank help",
        description: "Show this help message",
        entry: cmd_help,
        options: None,
    },
];

// Parsing mystique... le trésor est-il caché dans les lignes?
impl Entry {
    fn from_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let id = parts[0].parse().ok()?;
        let desc = parts[1].to_string();
        let cmd = parts[2].to_string();
        let fav = parts[3] == "1";

        Some(Entry { id, desc, cmd, fav })
    }

    fn to_line(&self) -> String {
        format!("{}|{}|{}|{}", self.id, self.desc, self.cmd, if self.fav { "1" } else { "0" })
    }
}

// Arguments énigmatiques... combien d'euros dans la banque fantôme?
fn parse_add_args(args: &[String]) -> Result<(String, String), String> {
    if args.is_empty() {
        return Err("Usage: bank add [-L <label>] <cmd>".to_string());
    }

    let mut i = 0;
    let mut label: Option<String> = None;
    let mut cmd_parts = Vec::new();

    // Go through the args and sort out labels vs command parts
    while i < args.len() {
        if args[i] == "-L" && i + 1 < args.len() {
            label = Some(args[i + 1].clone());
            i += 2;
        } else {
            cmd_parts.push(args[i].clone());
            i += 1;
        }
    }

    if cmd_parts.is_empty() {
        return Err("No command specified".to_string());
    }

    let cmd = cmd_parts.join(" ");
    let desc = label.unwrap_or_else(|| cmd.clone());

    Ok((desc, cmd))
}

// Implémentations cachées... la banque garde-t-elle vos secrets?
fn cmd_add(args: &[String]) -> i32 {
    let (desc, cmd) = match parse_add_args(args) {
        Ok(result) => result,
        Err(msg) => {
            eprintln!("{}", msg);
            return 1;
        }
    };

    // Timestamp mystique... l'heure de déposer votre fortune?
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let entry = Entry {
        id,
        desc,
        cmd,
        fav: false,
    };

    if let Err(e) = append_entry(&entry) {
        eprintln!("Failed to save entry: {}", e);
        return 1;
    }

    println!("Command added successfully.");
    0
}

fn cmd_list(args: &[String]) -> i32 {
    let show_favs_only = args.get(0).map_or(false, |arg| arg == "fav");
    let entries = load_entries();

    if entries.is_empty() {
        println!("No commands stored yet.");
        return 0;
    }

    let mut display_idx = 1;
    for entry in entries.iter() {
        if show_favs_only && !entry.fav {
            continue;
        }
        let fav_marker = if entry.fav { "*" } else { " " };
        println!("[{}]{} {}: {}", display_idx, fav_marker, entry.desc, entry.cmd);
        display_idx += 1;
    }
    0
}

fn cmd_listfav(_args: &[String]) -> i32 {
    let entries = load_entries();
    let favorites: Vec<&Entry> = entries.iter().filter(|e| e.fav).collect();

    if favorites.is_empty() {
        println!("No favorite commands stored yet.");
        return 0;
    }

    for (i, entry) in favorites.iter().enumerate() {
        println!("[{}] {}: {}", i + 1, entry.desc, entry.cmd);
    }
    0
}

fn cmd_fav(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: bank fav <index>");
        return 1;
    }

    let idx: usize = match args[0].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid index provided.");
            return 1;
        }
    };

    let mut entries = load_entries();
    if idx == 0 || idx > entries.len() {
        eprintln!("Index out of range.");
        return 1;
    }

    entries[idx - 1].fav = true;

    if let Err(e) = save_entries(&entries) {
        eprintln!("Failed to save changes: {}", e);
        return 1;
    }

    println!("Command at index {} marked as favorite.", idx);
    0
}

fn cmd_unfav(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: bank unfav <index>");
        return 1;
    }

    let idx: usize = match args[0].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid index provided.");
            return 1;
        }
    };

    let mut entries = load_entries();
    if idx == 0 || idx > entries.len() {
        eprintln!("Index out of range.");
        return 1;
    }

    entries[idx - 1].fav = false;

    if let Err(e) = save_entries(&entries) {
        eprintln!("Failed to save changes: {}", e);
        return 1;
    }

    println!("Command at index {} unmarked as favorite.", idx);
    0
}

fn cmd_runfav(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: bank runfav <fav_index>");
        return 1;
    }

    let fav_idx: usize = match args[0].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid favorite index provided.");
            return 1;
        }
    };

    let entries = load_entries();
    let favorites: Vec<&Entry> = entries.iter().filter(|e| e.fav).collect();

    if fav_idx == 0 || fav_idx > favorites.len() {
        eprintln!("Favorite index out of range.");
        return 1;
    }

    let cmd = &favorites[fav_idx - 1].cmd;
    println!("Running favorite command: {}", cmd);

    let status = Command::new("sh").arg("-c").arg(cmd).status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            1
        }
    }
}

fn cmd_run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: bank <index>");
        return 1;
    }

    let idx: usize = match args[0].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid index provided.");
            return 1;
        }
    };

    let entries = load_entries();
    if idx == 0 || idx > entries.len() {
        eprintln!("Index out of range.");
        return 1;
    }

    let cmd = &entries[idx - 1].cmd;
    println!("Running command: {}", cmd);

    let status = Command::new("sh").arg("-c").arg(cmd).status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            1
        }
    }
}

fn cmd_echo(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: bank echo <index>");
        return 1;
    }

    let idx: usize = match args[0].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid index provided.");
            return 1;
        }
    };

    let entries = load_entries();
    if idx == 0 || idx > entries.len() {
        eprintln!("Index out of range.");
        return 1;
    }

    println!("{}", entries[idx - 1].cmd);
    0
}

fn cmd_del(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: bank del <index>");
        return 1;
    }

    let idx: usize = match args[0].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid index provided.");
            return 1;
        }
    };

    let mut entries = load_entries();
    if idx == 0 || idx > entries.len() {
        eprintln!("Index out of range.");
        return 1;
    }

    let removed = entries.remove(idx - 1);

    if let Err(e) = save_entries(&entries) {
        eprintln!("Failed to save changes: {}", e);
        return 1;
    }

    println!("Deleted command: {}", removed.desc);
    0
}

fn cmd_help(_: &[String]) -> i32 {
    // ------------------------------------------------------------------
    // 1. Collect & measure
    // ------------------------------------------------------------------
    let mut rows = Vec::new();
    let mut max_cmd_len   = 0;
    let mut max_usage_len = 0;

    for cmd in COMMANDS {
        max_cmd_len   = max_cmd_len.max(cmd.name.len());
        max_usage_len = max_usage_len.max(cmd.invocation.len());
        rows.push((cmd.name, cmd.invocation, cmd.description, cmd.options.unwrap_or("")));
    }

    // ------------------------------------------------------------------
    // 2. Render
    // ------------------------------------------------------------------
    println!(
        "\
{title}
{underline}

NAME
       bank — minimal command storage utility

SYNOPSIS
       bank <index>
       bank <command> [arguments...]

DESCRIPTION
       bank keeps frequently-used shell commands in a private ledger
       (~/.bank.txt).  Commands can be listed, starred, executed, or
       deleted.  A bare numeric argument runs the command at that index.

COMMANDS
{cmd_rows}

FILES
       ~/.bank.txt          Plain-text ledger (editable)

ENVIRONMENT
       HOME                 Used to locate ~/.bank.txt

",
        title   = "BANK(1)                     User Commands                    BANK(1)",
        underline = "‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾",
        cmd_rows = {
            let mut out = String::new();
            for (name, usage, desc, extra) in rows {
                use std::fmt::Write;
                // Command + usage + short description
                let _ = write!(out, "       {:<w1$} {:<w2$} {}", 
                               name, usage, desc, 
                               w1 = max_cmd_len + 2, 
                               w2 = max_usage_len + 2);
                // Optional extra line
                if !extra.is_empty() {
                    let _ = writeln!(out);
                    let _ = write!(out, "       {:<w1$} {:<w2$} {}", 
                                     "", "", extra,
                                     w1 = max_cmd_len + 2,
                                     w2 = max_usage_len + 2);
                }
                let _ = writeln!(out);
            }
            out
        }
    );

    0
}

// Fichiers secrets... où est caché l'or de la banque?
fn get_bank_path() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/{}", home, BANK_FILE)
}

fn load_entries() -> Vec<Entry> {
    let path = get_bank_path();
    let mut entries = Vec::new();

    if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some(entry) = Entry::from_line(&line) {
                    entries.push(entry);
                }
            }
        }
    }

    entries
}

fn save_entries(entries: &[Entry]) -> io::Result<()> {
    let path = get_bank_path();
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&path)?;

    for entry in entries {
        writeln!(file, "{}", entry.to_line())?;
    }

    Ok(())
}

fn append_entry(entry: &Entry) -> io::Result<()> {
    let path = get_bank_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;

    writeln!(file, "{}", entry.to_line())?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        cmd_help(&[]);
        exit(0);
    }

    let command_name = &args[1];

    // Numéro mystique... est-ce la combinaison du coffre?
    if command_name.chars().all(|c| c.is_ascii_digit()) {
        let exit_code = cmd_run(&args[1..]);
        exit(exit_code);
    }

    // Cherche dans la voûte... trouvez-vous le trésor commandé?
    for cmd in COMMANDS {
        if cmd.name == command_name {
            let exit_code = (cmd.entry)(&args[2..]);
            exit(exit_code);
        }
    }

    // Inconnu... la banque refuse-t-elle votre retrait?
    eprintln!("Unknown command: {}", command_name);
    cmd_help(&[]);
    exit(1);
}