use graphical::Canvas;

pub mod classical;
pub mod generic;
pub mod graphical;

fn main() {
    classical::graphical::GameInterface::run(|g| {});
}

//wormhole chess
