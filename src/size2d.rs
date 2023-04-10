use std::fmt;
use std::str::FromStr;



#[derive(Clone)]
pub struct Size2D(pub usize, pub usize);

impl fmt::Display for Size2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.0, self.1)
    }
}

impl FromStr for Size2D {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        {
            let mut parts = s.split('x');
            let width = parts.next().ok_or("No width given")?;
            let height = parts.next().ok_or("No height given")?;
            let width: usize = width.parse().map_err(|_| "Invalid number")?;
            let height: usize = height.parse().map_err(|_| "Invalid number")?;

            match parts.next() {
                Some(_) => Err("Too many dimensions given"),
                None => Ok(Size2D(width, height))
            }
        }.map_err(|e| e.to_string())
    }
}
