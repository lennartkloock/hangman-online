use std::process::Command;

fn main() {
    Command::new("npx")
        .arg("tailwindcss")
        .arg("-i")
        .arg("./src/input.css")
        .arg("-o")
        .arg("./out/output.css")
        .output()
        .expect("Error running tailwindcss");
}
