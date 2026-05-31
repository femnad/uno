use crate::args;
mod layout;

pub fn run(args: args::Args) {
    layout::write_layout(args.keyboard, args.config)
}