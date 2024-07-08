
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
        matrix.sort_by(|a, b| {
            let ap = match a.iter().position(|e| {*e != 0.0}) {
                Some(p) => p,
                None => a.len(),
            };
            let bp = match b.iter().position(|e| {*e != 0.0}) {
                Some(p) => p,
                None => b.len(),
            };
            ap.partial_cmp(&bp).unwrap()
            
        });
        // loop {
        //     let mut swap: Option<(usize, usize)> = None;
        //     for (r, row) in matrix.iter().enumerate() {
        //         for (c, col) in row.iter().enumerate() {
        //             if *col != 0.0 {continue;}
        //             for (mut r2, row2) in matrix[r..].iter().enumerate() {
        //                 r2 += r;
        //                 if *row2.get(c).unwrap() != 0.0 {
        //                     swap = Some((r, r2));
        //                 }
        //             }
        //         }
        //     }
        //     match swap {
        //         Some(s) => matrix.swap(s.0, s.1),
        //         None => {break;},
        //     }
        // }
    }

    pub fn div(row: &mut Vec<f32>, value: f32) {
        let nrow = row.iter().map(|e| e / value).collect::<Vec<f32>>();
        *row = nrow;
    }
    pub fn mul(row: &mut Vec<f32>, value: f32) {
        let nrow = row.iter().map(|e| e * value).collect::<Vec<f32>>();
        *row = nrow;
    }
    pub fn add(row1: &mut Vec<f32>, row2: &Vec<f32>) {
        let zip = row1.iter().zip(row2.iter());
        let nrow = zip.map(|(r1, r2)| {*r1 + *r2}).collect::<Vec<f32>>();
        *row1 = nrow;
    }
    pub fn sub(row1: &mut Vec<f32>, row2: &Vec<f32>) {
        let zip = row1.iter().zip(row2.iter());
        let nrow = zip.map(|(r1, r2)| {*r1 - *r2}).collect::<Vec<f32>>();
        *row1 = nrow;
    }

    pub fn normalize(matrix: &mut Vec<Vec<f32>>) {
        if check(&matrix) == false {
            panic!("wrong matrix input");
        };
        sort(matrix);
        println!("sort");
        let len = matrix.len();
        for r in 0..len {
            println!("on row {r}");
            let row = matrix.get_mut(r).unwrap();
            let pivot = row.iter().find(|e| {**e != 0.0});
            match pivot {
                Some(pivot) => { div(row, *pivot); },
                None => { },
            };

            let mut row = row.clone();
            for u in (r + 1)..len {
                println!("running delte");
                let mut subrow = row.clone();
                let row = matrix.get_mut(u).unwrap();
                // let pivot = row.iter().find(|e| {**e != 0.0});
                let pivot = row.get(r);
                match pivot {
                    Some(pivot) => {
                        mul(&mut subrow, *pivot);
                        sub(row, &subrow);
                    },
                    None => {},
                }
            }
            println!("end of row");
        }
        println!("finishing");
    }
}

#[cfg(test)]
mod matrix_tests {
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
        // test 1
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

        // test 2
        let mut matrix = vec![
            vec![0.0, 5.0, 0.1],
            vec![0.0, 0.0, 0.5],
            vec![1.0, 3.0, 0.2],
        ];
        sort(&mut matrix);
        let expected = vec![
            vec![1.0, 3.0, 0.2],
            vec![0.0, 5.0, 0.1],
            vec![0.0, 0.0, 0.5],
        ];
        assert_eq!(matrix, expected);
    }
    #[test]
    fn add_test() {
        let mut a = vec![1.2, 5.0];
        let b = vec![8.8, -5.1];
        add(&mut a, &b);
        let expected = vec![10.0, 5.0 - 5.1];
        assert_eq!(a, expected);
    }
    #[test]
    fn normalize_test() {
        let mut matrix = vec![
            vec![0.0, 5.0, 0.1],
            vec![0.0, 0.0, 0.5],
            vec![1.0, 3.0, 0.2],
        ];
        normalize(&mut matrix);
        let expected = vec![
            vec![1.0, 3.0, 0.2],
            vec![0.0, 1.0, 0.1 / 5.0],
            vec![0.0, 0.0, 1.0],
        ];
        assert_eq!(matrix, expected);
    }
}
