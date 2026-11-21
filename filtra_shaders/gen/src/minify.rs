// Adapted from `vello_gpu_shaders` (Apache-2.0 OR MIT).

use naga::back::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use naga::{Function, Module, TypeInner};

/// Renames every declaration except the entry points and removes comments and whitespace.
pub(crate) fn minify_wgsl(source: &str) -> String {
    let mut module = naga::front::wgsl::parse_str(source).expect("linked WGSL should parse");
    rename_module(&mut module);
    let info = Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .expect("renamed WGSL should validate");
    let source = wgsl::write_string(&module, &info, wgsl::WriterFlags::empty())
        .expect("renamed WGSL should serialize");

    minify_code(&source)
}

/// Removes comments and whitespace, keeping preprocessor directives on lines of their own.
pub(crate) fn minify_glsl(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut code = String::new();

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            append(&mut output, &code);
            code.clear();

            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }

            output.push_str(trimmed);
            output.push('\n');
        } else {
            code.push_str(line);
            code.push('\n');
        }
    }

    append(&mut output, &code);
    output
}

fn append(output: &mut String, source: &str) {
    let minified = minify_code(source);

    if minified.is_empty() {
        return;
    }

    if !output.is_empty()
        && !output.ends_with('\n')
        && needs_separator(
            output.chars().next_back().unwrap(),
            minified.chars().next().unwrap(),
        )
    {
        output.push(' ');
    }

    output.push_str(&minified);
}

fn rename_module(module: &mut Module) {
    let mut names = Names::default();
    let types: Vec<_> = module.types.iter().map(|(handle, _)| handle).collect();

    for handle in types {
        let original = module.types[handle].clone();
        let mut ty = original.clone();
        rename(&mut ty.name, &mut names);

        if let TypeInner::Struct { members, .. } = &mut ty.inner {
            for member in members {
                rename(&mut member.name, &mut names);
            }
        }

        if ty != original {
            module.types.replace(handle, ty);
        }
    }

    for (_, constant) in module.constants.iter_mut() {
        rename(&mut constant.name, &mut names);
    }

    for (_, global) in module.global_variables.iter_mut() {
        rename(&mut global.name, &mut names);
    }

    for (_, function) in module.functions.iter_mut() {
        rename_function(function, &mut names);
    }

    // Only the function inside each entry point is renamed; the entry point keeps its name.
    for entry_point in &mut module.entry_points {
        rename_function(&mut entry_point.function, &mut names);
    }
}

fn rename_function(function: &mut Function, names: &mut Names) {
    rename(&mut function.name, names);

    for argument in &mut function.arguments {
        rename(&mut argument.name, names);
    }

    for (_, local) in function.local_variables.iter_mut() {
        rename(&mut local.name, names);
    }

    for name in function.named_expressions.values_mut() {
        *name = names.next();
    }
}

fn rename(name: &mut Option<String>, names: &mut Names) {
    if name.is_some() {
        *name = Some(names.next());
    }
}

// Spreadsheet-style names: `A` to `Z`, then `AA`, `AB` and so on.
#[derive(Default)]
struct Names {
    next: usize,
}

impl Names {
    fn next(&mut self) -> String {
        let mut value = self.next;
        self.next += 1;
        let mut bytes = Vec::with_capacity(3);

        loop {
            bytes.push(b'A' + (value % 26) as u8);
            value /= 26;

            if value == 0 {
                break;
            }

            value -= 1;
        }

        bytes.reverse();
        String::from_utf8(bytes).unwrap()
    }
}

fn minify_code(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    let mut pending_separator = false;

    while index < chars.len() {
        let current = chars[index];
        let next = chars.get(index + 1).copied();

        if current.is_whitespace() {
            pending_separator = true;
            index += 1;
            continue;
        }

        if current == '/' && next == Some('/') {
            pending_separator = true;

            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }

            continue;
        }

        if current == '/' && next == Some('*') {
            pending_separator = true;
            index += 2;

            while index + 1 < chars.len() && !(chars[index] == '*' && chars[index + 1] == '/') {
                index += 1;
            }

            index = (index + 2).min(chars.len());
            continue;
        }

        if pending_separator
            && !output.is_empty()
            && needs_separator(output.chars().next_back().unwrap(), current)
        {
            output.push(' ');
        }

        pending_separator = false;
        output.push(current);
        index += 1;
    }

    output
}

// Whether removing the whitespace between two characters would merge two tokens.
fn needs_separator(left: char, right: char) -> bool {
    let identifier = |c: char| c.is_ascii_alphanumeric() || c == '_';

    (identifier(left) && identifier(right))
        || matches!(
            (left, right),
            ('+', '+')
                | ('-', '-')
                | ('-', '>')
                | ('/', '/')
                | ('/', '*')
                | ('<', '<')
                | ('<', '=')
                | ('>', '>')
                | ('>', '=')
                | ('=', '=')
                | ('!', '=')
                | ('&', '&')
                | ('|', '|')
                | (':', ':')
        )
        || (left.is_ascii_digit() && right == '.')
        || (left == '.' && right.is_ascii_digit())
}
