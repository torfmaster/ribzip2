use std::vec;

#[derive(Debug)]
pub(crate) struct KMeansResult {
    pub(crate) means: Vec<Vec<usize>>,
    pub(crate) assignments: Vec<u8>,
}

/// Solve the k means clustering problem
pub(crate) struct KMeansProblem<'a> {
    pub(crate) dimension: usize,
    pub(crate) data: &'a [Vec<u8>],
    pub(crate) num_iterations: usize,
    pub(crate) num_clusters: usize,
}

impl<'a> KMeansProblem<'a> {
    /// Solve the k means clustering problem using Lloyd's algorithm
    /// weight the centers by 10_000 to make them representable by usize
    pub(crate) fn solve(self) -> KMeansResult {
        let mut centers = vec![];

        // Choose "random" centers
        for i in 0..self.num_clusters {
            centers.push(
                self.data[i * self.data.len() / self.num_clusters]
                    .iter()
                    .map(|x| *x as f32)
                    .collect::<Vec<_>>(),
            );
        }

        let mut cluster_assignments = vec![0; self.data.len()];

        for _ in 0..self.num_iterations {
            // compute assignment
            for (data_point_number, point) in self.data.iter().enumerate() {
                let mut min_assignment = 0;
                let mut min_distance: Option<_> = None;

                for (cluster_number, center) in centers.iter().enumerate() {
                    let distance = euclidean_distance(center, point);

                    if let Some(min_distance_value) = min_distance {
                        if distance < min_distance_value {
                            min_distance = Some(distance);
                            min_assignment = cluster_number;
                        }
                    } else {
                        min_distance = Some(distance);
                        min_assignment = cluster_number;
                    }
                }

                cluster_assignments[data_point_number] = min_assignment as u8;
            }

            // clear centers
            for i in centers.iter_mut() {
                *i = vec![0.0; self.dimension];
            }

            let mut cluster_sizes = vec![0usize; self.num_clusters];

            // update centers
            for (data_point, assigment) in self.data.iter().zip(cluster_assignments.iter()) {
                let current_cluster_size = cluster_sizes[*assigment as usize];
                for (entry, point) in centers[*assigment as usize]
                    .iter_mut()
                    .zip(data_point.iter())
                {
                    *entry = ((*entry as f32) * current_cluster_size as f32 + (*point as f32))
                        / (current_cluster_size as f32 + 1.0);
                }
                cluster_sizes[*assigment as usize] += 1;
            }
        }

        KMeansResult {
            assignments: cluster_assignments,
            means: centers
                .iter()
                .map(|x| {
                    x.iter()
                        .map(|x| (*x * 10_000.0) as usize)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        }
    }
}

fn euclidean_distance(v1: &[f32], v2: &[u8]) -> f32 {
    ((v1.iter()
        .zip(v2.iter())
        .map(|(x, y)| (*x - *y as f32).powi(2)))
    .sum::<f32>())
    .sqrt()
}
