use std::process::Command;

pub trait Executor {
    fn run(&self, args: &[&str]) -> String;
    fn stream(&self, args: &[&str], on_line: &mut dyn FnMut(&str));
}


pub struct Kubectl;

impl Executor for Kubectl {
    fn run(&self, args: &[&str]) -> String {
        let output = Command::new("kubectl")
            .args(args)
            .output()
            .expect("failed to run kubectl");

        String::from_utf8_lossy(&output.stdout).to_string()
    }

    fn stream(&self, args: &[&str], on_line: &mut dyn FnMut(&str)) {
        let mut child = Command::new("kubectl")
            .args(args)
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn");

        let stdout = child.stdout.take().expect("no stdout");
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            if let Ok(line) = line {
                on_line(&line);
            }
        }
    }
}