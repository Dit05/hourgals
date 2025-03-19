pub mod hourglass;

use hourglass::Hourglass;


fn main() {
    let mut glass = Hourglass::new(7, 12);
    let mut sand_to_add = 30;
    let mut no_moves_since = 0;

    loop {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // Clear and go to top left corner
        println!("{}", glass);

        if sand_to_add > 0 && glass.try_add_sand() {
            sand_to_add -= 1;
        }

        let moves = glass.advance(&mut rand::rng());
        if moves == 0 {
            no_moves_since += 1;
        } else {
            no_moves_since = 0;
        }

        if no_moves_since >= 10 {
            no_moves_since = 0;
            glass.flip();
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
