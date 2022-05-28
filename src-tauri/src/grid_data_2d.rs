#![allow(dead_code)]
use std::cmp::min;

#[derive(Clone)]
pub struct DimensionData {
    vec: Vec<f64>,
    pub min: f64,
    pub max: f64,
}

impl DimensionData {
    fn new() -> Self {
        DimensionData {
            vec: Vec::new(),
            min: 0.0,
            max: 0.0,
        }
    }

    pub fn from(vec: &Vec<f64>) -> Self {
        let mut data = DimensionData::new();
        data.vec = vec.clone();
        data.find_limits();
        data
    }

    fn find_limits(&mut self) {
        if self.vec.is_empty() {
            self.min = 0.0;
            self.max = 0.0;
        } else {
            self.min = self.vec[0];
            self.max = self.vec[0];
            for v in self.vec.iter() {
                if *v < self.min {
                    self.min = *v;
                }
                if *v > self.max {
                    self.max = *v;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct GridData2D {
    pub x_data: DimensionData,
    pub y_data: DimensionData,
    pub values_data: DimensionData,
    cumulative: Vec<f64>,
}

impl GridData2D {
    fn new() -> Self {
        Self {
            x_data: DimensionData::new(),
            y_data: DimensionData::new(),
            values_data: DimensionData::new(),
            cumulative: Vec::new(),
        }
    }

    fn from(xs: &Vec<f64>, ys: &Vec<f64>, values: &Vec<f64>) -> Result<Self, String> {
        if xs.len() * ys.len() != values.len() {
            return Err(
                "The number of values must be equal to the number of x times y values".to_owned(),
            );
        }

        let mut grid = Self {
            x_data: DimensionData::from(xs),
            y_data: DimensionData::from(ys),
            values_data: DimensionData::from(values),
            cumulative: Vec::new(),
        };

        grid.calculate_cumulative();

        Ok(grid)
    }

    fn data_changed(&mut self) {
        // TODO: Rewritten directly, potential for refactor
        let mut already_sorted = true;

        let mut new_x = Vec::with_capacity(self.x_data.vec.len());
        let mut new_y = Vec::with_capacity(self.y_data.vec.len());

        for (i, x) in self.x_data.vec.iter().enumerate() {
            new_x.push((*x, i));
        }
        for (i, y) in self.y_data.vec.iter().enumerate() {
            new_y.push((*y, i));
        }

        new_x.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        new_y.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for (i, &(_, old_i)) in new_x.iter().enumerate() {
            if i != old_i {
                already_sorted = false;
                break;
            }
        }
        for (i, &(_, old_i)) in new_y.iter().enumerate() {
            if i != old_i {
                already_sorted = false;
                break;
            }
        }

        if !already_sorted {
            self.x_data.vec = new_x.iter().map(|x| x.0).collect();
            self.y_data.vec = new_y.iter().map(|y| y.0).collect();

            let mut new_values = Vec::with_capacity(self.values_data.vec.len());

            for x in &self.x_data.vec {
                for y in &self.y_data.vec {
                    new_values.push(x * y);
                }
            }

            self.values_data.vec = new_values;
        }

        self.find_limits();
        self.calculate_cumulative();
    }

    fn find_limits(&mut self) {
        self.x_data.find_limits();
        self.y_data.find_limits();
        self.values_data.find_limits();
    }

    fn calculate_cumulative(&mut self) {
        self.cumulative = self.values_data.vec.clone();

        for i in 1..self.x_data.vec.len() {
            for j in 1..self.y_data.vec.len() {
                self.cumulative[i + j * self.x_data.vec.len()] += self.cumulative
                    [i + (j - 1) * self.x_data.vec.len()]
                    + self.cumulative[(i - 1) + j * self.x_data.vec.len()]
                    - self.cumulative[(i - 1) + (j - 1) * self.x_data.vec.len()];
            }
        }
    }

    pub fn size(&self) -> usize {
        self.values_data.vec.len()
    }

    pub fn x_values(&self) -> &Vec<f64> {
        &self.x_data.vec
    }

    pub fn y_values(&self) -> &Vec<f64> {
        &self.y_data.vec
    }

    pub fn is_empty(&self) -> bool {
        self.values_data.vec.is_empty()
    }

    pub fn get_1d_index(&self, nx: usize, ny: usize) -> Option<usize> {
        if nx >= self.x_data.vec.len() || ny >= self.y_data.vec.len() {
            return None;
        }
        Some(nx + ny * self.x_data.vec.len())
    }

    pub fn get_1d_index_bounded(&self, nx: usize, ny: usize) -> Option<usize> {
        self.get_1d_index(
            min(nx, self.x_data.vec.len() - 1),
            min(ny, self.y_data.vec.len() - 1),
        )
    }

    pub fn get_value(&self, nx: usize, ny: usize) -> Option<f64> {
        let index = self.get_1d_index(nx, ny)?;
        Some(self.values_data.vec[index])
    }

    pub fn set_value(&mut self, nx: usize, ny: usize, value: f64) -> Option<()> {
        let index = self.get_1d_index(nx, ny)?;
        self.values_data.vec[index] = value;
        Some(())
    }

    pub fn get_value_bounded(&self, nx: usize, ny: usize) -> Option<f64> {
        let index = self.get_1d_index_bounded(nx, ny)?;
        Some(self.values_data.vec[index])
    }

    pub fn get_interpolated_value(&self, x: f64, y: f64) -> Option<f64> {
        if self.is_empty() {
            return None;
        }

        let closest_x = self
            .x_data
            .vec
            .binary_search_by(|val| val.partial_cmp(&x).unwrap())
            .unwrap_or_else(|e| e);
        let closest_y = self
            .y_data
            .vec
            .binary_search_by(|val| val.partial_cmp(&y).unwrap())
            .unwrap_or_else(|e| e);

        let second_closest_x = min(closest_x - 1, 0);
        let second_closest_y = min(closest_y - 1, 0);
        let ratio_x = self.x_data.vec[closest_x]
            - x / (self.x_data.vec[closest_x] - self.x_data.vec[second_closest_x]);
        let ratio_y = self.y_data.vec[closest_y]
            - y / (self.y_data.vec[closest_y] - self.y_data.vec[second_closest_y]);
        let inv_ratio_x = 1.0 - ratio_x;
        let inv_ratio_y = 1.0 - ratio_y;

        let mut result = 0.0;
        result += inv_ratio_x * inv_ratio_y * self.get_value_bounded(closest_x, closest_y).unwrap();
        result +=
            ratio_x * inv_ratio_y * self.get_value_bounded(second_closest_x, closest_y).unwrap();
        result +=
            inv_ratio_x * ratio_y * self.get_value_bounded(closest_x, second_closest_y).unwrap();
        result += ratio_x
            * ratio_y
            * self
                .get_value_bounded(second_closest_x, second_closest_y)
                .unwrap();
        Some(result)
    }

    pub fn value_at(&self, x: f64, y: f64) -> Option<f64> {
        if self.is_empty() {
            return None;
        }

        let closest_x = self
            .x_data
            .vec
            .binary_search_by(|val| val.partial_cmp(&x).unwrap())
            .unwrap_or_else(|e| e);
        let closest_y = self
            .y_data
            .vec
            .binary_search_by(|val| val.partial_cmp(&y).unwrap())
            .unwrap_or_else(|e| e);

        self.get_value(closest_x, closest_y)
    }

    pub fn sum_in_index_range(&self, range_x: (usize, usize), range_y: (usize, usize)) -> f64 {
        if self.is_empty() {
            return 0.0;
        }

        let range_x = (
            min(range_x.0, self.x_data.vec.len() - 1),
            min(range_x.1, self.x_data.vec.len() - 1),
        );
        let range_y = (
            min(range_y.0, self.y_data.vec.len() - 1),
            min(range_y.1, self.y_data.vec.len() - 1),
        );

        let mut result = self.cumulative[self.get_1d_index(range_x.1, range_y.1).unwrap()];
        if range_x.0 != 0 {
            result -= self.cumulative[self.get_1d_index(range_x.0 - 1, range_y.1).unwrap()];
        }
        if range_y.0 != 0 {
            result -= self.cumulative[self.get_1d_index(range_x.1, range_y.0 - 1).unwrap()];
        }
        if range_x.0 != 0 && range_y.0 != 0 {
            result += self.cumulative[self.get_1d_index(range_x.0 - 1, range_y.0 - 1).unwrap()];
        }
        result
    }
}
