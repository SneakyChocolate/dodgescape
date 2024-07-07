
pub mod matrix {
    pub fn check(matrix: &Vec<Vec<f32>>) -> bool {
        let width = match matrix.first() {
            Some(e) => e.len(),
            None => return false,
        };
        // check width
        for row in matrix.iter() {
            if row.len() != width {return false;}
        }
        true
    }

    pub fn sort(matrix: &mut Vec<Vec<f32>>) {
        loop {
            let mut swap: Option<(usize, usize)> = None;
            for (r, row) in matrix.iter().enumerate() {
                for (c, col) in row.iter().enumerate() {
                    if *col != 0.0 {continue;}
                    for (mut r2, row2) in matrix[r..].iter().enumerate() {
                        r2 += r;
                        if *row2.get(c).unwrap() != 0.0 {
                            swap = Some((r, r2));
                        }
                    }
                }
            }
            match swap {
                Some(s) => matrix.swap(s.0, s.1),
                None => {break;},
            }
        }
    }

    pub fn solve(matrix: &Vec<Vec<f32>>) -> Vec<f32> {
        if check(&matrix) == false {panic!("wrong matrix input");};
        for row in matrix.iter() {
            
        }
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::matrix::*;

    #[test]
    fn check_test() {
        let matrix1 = vec![
            vec![0.0, 5.0],
            vec![1.0, 3.0],
        ];
        let matrix2 = vec![
            vec![0.0, 5.0],
            vec![1.0],
        ];
        assert_eq!(check(&matrix1), true);
        assert_eq!(check(&matrix2), false);
    }
    #[test]
    fn sort_test() {
        let mut matrix = vec![
            vec![0.0, 5.0],
            vec![1.0, 3.0],
        ];
        sort(&mut matrix);
        let expected = vec![
            vec![1.0, 3.0],
            vec![0.0, 5.0],
        ];
        assert_eq!(matrix, expected);
    }
}
