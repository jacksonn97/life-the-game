use crate::draw::Rect;

#[derive(Clone)]
pub struct Field {
    size: Rect,
    data: Vec<Vec<bool>>,
}

impl Field {

    #[inline]
    pub fn new(data: Vec<Vec<bool>>) -> Self {
        let terminal = Rect::term_size();
        let size = Self::data_size(&data);

        if terminal < size {
            panic!("Terminal size should be better then field size!");
        } if !data.iter().all(|v| v.len() == size.w() as usize) {
            panic!("All rows of the matrix should be same size!");
        }

        Field {
            size,
            data,
        }
    }

    #[inline]
    pub fn from_string(s: String) -> Self {
        let mut data: Vec<Vec<bool>> = vec![];

        for i in s.split('\n') {
            let mut r: Vec<bool> = vec![];
            for j in i.chars() {
                let b = match j {
                    '0' | ' ' => false,
                    _ => true,
                };
                r.push(b);
            }
            if !r.is_empty() {
                data.push(r);
            }
        }

        let terminal = Rect::term_size();
        let size = Self::data_size(&data);

        if terminal < size {
            panic!("Terminal size should be better then field size!");
        } if !data.iter().all(|v| v.len() == size.w() as usize) {
            panic!("All rows of the matrix should be same size!");
        }

        Field {
            size,
            data,
        }
    }

    #[inline]
    pub fn size(&self) -> &Rect {
        &self.size
    } 

    #[inline]
    pub fn data<'a>(&'a self) -> &'a Vec<Vec<bool>> {
        &self.data
    }

    #[inline]
    fn get(&self, i: i32, j: i32) -> Option<bool> {
        if i >= self.size.h() as i32 || j >= self.size.w() as i32 || i < 0 || j < 0 {
            None
        } else {
            Some(self.data[i as usize][j as usize])
        }
    }

    fn data_size(data: &Vec<Vec<bool>>) -> Rect {
        Rect::new(
        data.iter().next().unwrap().len() as u16,
        data.len() as u16)
    }

    pub fn tick(&mut self) {
        let (w, h) = self.size.unwrap();
        let mut n: Vec<Vec<bool>> = std::iter::repeat(vec![]).take(h as usize).collect();

        for i in 0..h as i32 {
            for j in 0..w as i32 {
                n[i as usize].push(Self::produce_value(
                        self.data[i as usize][j as usize],
                        &[
                        self.get(i-1, j-1),
                        self.get(i-1, j+1),
                        self.get(i+1, j-1),
                        self.get(i+1, j+1),

                        self.get(i-1, j),
                        self.get(i+1, j),
                        self.get(i, j-1),
                        self.get(i, j+1),

                ]))
            }
        }
        
        // println!("{n:?}\r");
        self.data = n;
    }

    fn produce_value(current: bool, n: &[Option<bool>; 8]) -> bool {
        let c = n.iter().filter(|c| c.is_some()).filter(|c| c.unwrap()).count();
        if !current && (3 == c) {
            true
        } else if current && (c == 2 || c == 3) {
            true
        } else if current && (c > 3 || c < 1) {
            false
        } else {
            false
        }
    }
}

