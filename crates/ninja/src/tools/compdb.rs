use crate::graph::State;

pub(super) fn run(state: &State, args: &[String]) -> Result<u8, String> {
    let mut rules = Vec::new();
    let mut expand_rsp = false;
    let mut options = true;
    for arg in args {
        if options && arg == "--" {
            options = false;
        } else if options && arg.starts_with('-') && arg != "-" {
            for flag in arg[1..].chars() {
                match flag {
                    'x' => expand_rsp = true,
                    'h' => {
                        print!("usage: ninja -t compdb [options] [rules]\n\noptions:\n  -x     expand @rspfile style response file invocations\n");
                        return Ok(1);
                    }
                    _ => return Err(format!("compdb: unknown option -{flag}")),
                }
            }
        } else {
            rules.push(arg.as_str());
        }
    }
    let dir = std::env::current_dir().map_err(|e| format!("cannot determine current directory: {e}"))?;
    print!("{}", super::compdb_data::render_rules(state, &dir.to_string_lossy(), &rules, expand_rsp));
    Ok(0)
}
