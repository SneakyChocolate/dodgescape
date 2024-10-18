use crate::Float;


pub type Matrix = Vec<Vec<Float>>;

pub mod matrix {
    use crate::Float;

    use super::Matrix;

    pub fn check(matrix: &Matrix) -> bool {
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

    pub fn sort(matrix: &mut Matrix) {
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
    }

    pub fn div(row: &mut Vec<Float>, value: Float) {
        let nrow = row.iter().map(|e| e / value).collect::<Vec<Float>>();
        *row = nrow;
    }
    pub fn mul(row: &mut Vec<Float>, value: Float) {
        let nrow = row.iter().map(|e| e * value).collect::<Vec<Float>>();
        *row = nrow;
    }
    pub fn add(row1: &mut Vec<Float>, row2: &Vec<Float>) {
        let zip = row1.iter().zip(row2.iter());
        let nrow = zip.map(|(r1, r2)| {*r1 + *r2}).collect::<Vec<Float>>();
        *row1 = nrow;
    }
    pub fn sub(row1: &mut Vec<Float>, row2: &Vec<Float>) {
        let zip = row1.iter().zip(row2.iter());
        let nrow = zip.map(|(r1, r2)| {*r1 - *r2}).collect::<Vec<Float>>();
        *row1 = nrow;
    }

    pub fn normalize(matrix: &mut Matrix) {
        if check(&matrix) == false {
            panic!("wrong matrix input");
        };
        sort(matrix);
        let len = matrix.len();
        for r in 0..len {
            let row = matrix.get_mut(r).unwrap();
            let pivot = row.iter().find(|e| {**e != 0.0});
            match pivot {
                Some(pivot) => { div(row, *pivot); },
                None => { },
            };

            let mut row = row.clone();
            for u in 0..len {
                if u == r {continue;}
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
        }
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
        // works fine tested with https://onlinemschool.com/math/assistance/equation/gaus/
        let mut matrix = vec![
            vec![3.0, 1.5, 5.0, -5.0],
            vec![8.0, -15.0, 0.0, 3.2],
            vec![0.0, 4.6, 0.2, 60.7],
        ];
        normalize(&mut matrix);
        println!("{:#?}", matrix);
        assert_eq!(true, true);
    }
}
