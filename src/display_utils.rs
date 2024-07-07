use std::fmt::Display;


pub struct DisplaySlice<'a, T: Display>(pub &'a [T]);

impl <'a, T: Display> Display for DisplaySlice<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[")?;
		for i in 0..self.0.len() {
			write!(f, "{}", self.0[i])?;
			if i < self.0.len() - 1 {
				write!(f, ", ")?;
			}
		}
		write!(f, "]")?;
		Ok(())
	}
}

#[extend::ext]
pub impl <'a, T: Display> &'a [T] {
	fn to_display(self) -> DisplaySlice<'a, T> {
		DisplaySlice(self)
	}
}