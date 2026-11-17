use std::io::{Stdin, Write, stdout};

pub fn read_stdin(stdin: &mut Stdin, input_value: &mut String, prompt: &str) -> Result<(), String> {
    print!("Enter {}: ", prompt);

    stdout()
        .flush()
        .map_err(|x| format!("STDOUT error: {:?}", x))?;

    stdin
        .read_line(input_value)
        .map_err(|x| format!("STDIN error: {:?}", x))?;

    // To remove trailing \n
    *input_value = input_value.split('\n').next().unwrap().to_string();

    Ok(())
}