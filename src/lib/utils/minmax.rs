fn maxmin_f64(nums: Vec<f64>, min_mode: bool) -> f64 {
  let mut ret_val: f64 = 0f64;
  let mut has_match = false;
  for n in nums.iter().copied() {
      if !has_match {
          ret_val = n;
          has_match = true;
      } else if (min_mode && n < ret_val) || (!min_mode && n > ret_val) {
          ret_val = n;
      }
  }
  ret_val
}

pub fn max_f64(nums: Vec<f64>) -> f64 {
  maxmin_f64(nums, false)
}

pub fn min_f64(nums: Vec<f64>) -> f64 {
  maxmin_f64(nums, true)
}