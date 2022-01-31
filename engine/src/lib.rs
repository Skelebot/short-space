extern crate color_eyre as eyre;
extern crate nalgebra as na;
extern crate ncollide3d as nc;

pub mod assets;
pub mod graphics;
pub mod input;
pub mod physics;
pub mod spacetime;
pub mod state;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
