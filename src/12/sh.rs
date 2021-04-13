use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{close, dup2, execvp, fork, pipe, ForkResult, Pid};
use std::ffi::CString;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process::exit;
use whoami;

enum RedirectCategory {
    Append,
    Overwrite,
    None,
}

struct RedirectInfo<'a> {
    path: &'a str,
    category: RedirectCategory,
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut buf_in = BufReader::new(stdin.lock());
    let mut input_string = String::new();
    let mut buf_out = BufWriter::new(stdout.lock());

    loop {
        buf_out
            .write(format!("{}@{}$ ", whoami::username(), whoami::hostname()).as_bytes())
            .unwrap_or_else(|why| panic!("aborted: {}", why.to_string()));
        buf_out
            .flush()
            .unwrap_or_else(|why| panic!("aborted: {}", why.to_string()));

        buf_in
            .read_line(&mut input_string)
            .unwrap_or_else(|why| panic!("aborted: {}", why.to_string()));

        input_string.remove(input_string.len() - 1);

        let args: Vec<&str> = input_string.split_whitespace().collect();

        if args.len() == 0 {
            continue;
        }

        if args.len() == 1 && args[0] == "exit" {
            break;
        }

        let (pipe_args, redirect_info) = parse_redirect_info(slice_vec_with_str(args, "|"));

        if pipe_args.len() == 1 {
            singlestage_pipe(pipe_args, redirect_info).unwrap();
        } else {
            multistage_pipe(pipe_args, redirect_info).unwrap();
        }

        input_string.clear();
    }
}

fn parse_redirect_info<'a>(mut args: Vec<Vec<&'a str>>) -> (Vec<Vec<&'a str>>, RedirectInfo<'a>) {
    let args_len = args.len();
    let last_arg_len = args[args_len - 1].len();
    if last_arg_len < 3 {
        return (
            args,
            RedirectInfo {
                path: "",
                category: RedirectCategory::None,
            },
        );
    }
    match args[args_len - 1][last_arg_len - 2] {
        ">" => {
            let path = args[args_len - 1].pop().unwrap();
            args[args_len - 1].pop().unwrap();
            (
                args,
                RedirectInfo {
                    path: path,
                    category: RedirectCategory::Overwrite,
                },
            )
        }

        ">>" => {
            let path = args[args_len - 1].pop().unwrap();
            args[args_len - 1].pop().unwrap();
            (
                args,
                RedirectInfo {
                    path: path,
                    category: RedirectCategory::Append,
                },
            )
        }
        _ => (
            args,
            RedirectInfo {
                path: "",
                category: RedirectCategory::None,
            },
        ),
    }
}

fn singlestage_pipe(pipe_args: Vec<Vec<&str>>, redirect_info: RedirectInfo) -> nix::Result<()> {
    match unsafe { fork() }? {
        ForkResult::Parent { child, .. } => {
            match waitpid(child, None)? {
                WaitStatus::Exited(pid, status) => {
                    println!("child (PID={}) finished: exit, status={}", pid, status);
                }
                WaitStatus::Signaled(pid, status, _) => {
                    println!("child (PID={}) finished: signal, sig={}", pid, status);
                }
                _ => println!("abnoraml exit"),
            };
        }
        ForkResult::Child => {
            let first_args = pipe_args[0].clone();
            let command = CString::new(first_args[0].to_string()).unwrap();
            let command_args: Vec<CString> = first_args
                .iter()
                .map(|arg| CString::new(*arg).unwrap())
                .collect();

            match execvp(&command, &command_args) {
                Ok(_) => exit(0),
                Err(_) => {
                    eprintln!("{}: command not found", first_args[0]);
                    exit(1);
                }
            };
        }
    };

    Ok(())
}

fn multistage_pipe(pipe_args: Vec<Vec<&str>>, redirect_info: RedirectInfo) -> nix::Result<()> {
    let mut pipefd: Vec<(i32, i32)> = Vec::with_capacity(pipe_args.len());
    let mut children: Vec<Pid> = Vec::with_capacity(pipe_args.len());

    // [0, pipe_args.len()-1]
    for i in 0..pipe_args.len() {
        if i != pipe_args.len() - 1 {
            pipefd.push(pipe()?); //最後のコマンドでなければパイプを作成
        }

        match unsafe { fork()? } {
            ForkResult::Parent { child, .. } => {
                children.push(child);

                // 親側から実行済みのパイプを消す
                if i > 0 {
                    close(pipefd[i - 1].0)?;
                    close(pipefd[i - 1].1)?;
                }
            }
            ForkResult::Child => {
                let first_args = pipe_args[i].clone();
                let command = CString::new(first_args[0].to_string()).unwrap();
                let command_args: Vec<CString> = first_args
                    .iter()
                    .map(|arg| CString::new(*arg).unwrap())
                    .collect();

                if i == 0 {
                    dup2(pipefd[i].1, 1).unwrap_or_else(|_| exit(1));
                    close(pipefd[i].0).unwrap_or_else(|_| exit(1));
                    close(pipefd[i].1).unwrap_or_else(|_| exit(1));
                } else if i == pipe_args.len() - 1 {
                    dup2(pipefd[i - 1].0, 0).unwrap_or_else(|_| exit(1));
                    close(pipefd[i - 1].0).unwrap_or_else(|_| exit(1));
                    close(pipefd[i - 1].1).unwrap_or_else(|_| exit(1));
                } else {
                    // 0から取り出す(読み込み)
                    dup2(pipefd[i - 1].0, 0).unwrap_or_else(|_| exit(1));
                    close(pipefd[i - 1].0).unwrap_or_else(|_| exit(1));
                    close(pipefd[i - 1].1).unwrap_or_else(|_| exit(1));

                    // 1に入れる(書き込み)
                    dup2(pipefd[i].1, 1).unwrap_or_else(|_| exit(1));
                    close(pipefd[i].0).unwrap_or_else(|_| exit(1));
                    close(pipefd[i].1).unwrap_or_else(|_| exit(1));
                }

                match execvp(&command, &command_args) {
                    Ok(_) => exit(0),
                    Err(_) => {
                        eprintln!("{}: command not found", first_args[0]);
                        exit(1);
                    }
                };
            }
        };
    }

    for child in children {
        match waitpid(child, None)? {
            WaitStatus::Exited(pid, status) => {
                println!("child (PID={}) finished: exit, status={}", pid, status);
            }
            WaitStatus::Signaled(pid, status, _) => {
                println!("child (PID={}) finished: signal, sig={}", pid, status);
            }
            _ => println!("abnoraml exit"),
        };
    }

    Ok(())
}

fn slice_vec_with_str<'a, 'b>(args: Vec<&'a str>, slice_key: &'b str) -> Vec<Vec<&'a str>> {
    let positions: Vec<usize> = args
        .iter()
        .enumerate()
        .filter(|&(_, s)| *s == slice_key)
        .map(|(i, _)| i)
        .collect();

    slice_with_positions(args, positions)
}

fn slice_with_positions<T: Clone>(args: Vec<T>, mut positions: Vec<usize>) -> Vec<Vec<T>> {
    positions.push(args.len());
    let mut poslr: Vec<(usize, usize)> = Vec::new();
    poslr.push((0, positions[0]));

    for i in 1..positions.len() {
        poslr.push((positions[i - 1] + 1, positions[i]));
    }

    poslr
        .iter()
        .map(|&(l, r)| {
            let mut tmp = Vec::new();
            tmp.extend_from_slice(&args[l..r]);
            tmp
        })
        .collect::<Vec<Vec<T>>>()
}
