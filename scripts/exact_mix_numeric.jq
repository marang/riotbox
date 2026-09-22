# Numeric evidence predicates shared by the exact RuntimeMix validators.
# Policy values belong to the producing manifest, not the synthetic source.
# This validates recorded technical evidence; it does not authenticate metrics
# or grant a musical/holdout pass. See audio_numeric_values.md and RBX-379.
def finite_number: type == "number" and isfinite and (isnan | not);
def nonnegative_number: finite_number and . >= 0;
def positive_number: finite_number and . > 0;
def positive_integer: positive_number and . == floor;
def nonnegative_integer: nonnegative_number and . == floor;

def exact_pack_thresholds_ok:
  .max_exact_mix_limited_sample_count == 0
  and all([.min_mix_rms, .min_monitor_delta_rms,
           .min_isolated_tr909_regression_rms][]; positive_number and . <= 1)
  and (.max_source_monitor_silence_ratio | nonnegative_number and . < 1);

def exact_limiter_ok($max_limited):
  ($max_limited == 0)
  and (.threshold | positive_number)
  and (.ceiling | positive_number)
  and .ceiling > .threshold and .ceiling < 1
  and (.limited_sample_count | nonnegative_integer)
  and .limited_sample_count <= $max_limited
  and .pre.clip_count == 0 and .post.clip_count == 0
  and (.applied == (.limited_sample_count > 0));

def fill_exit_boundary_ok($sample_rate):
  ($sample_rate | positive_integer)
  and (.window_ms | positive_integer)
  and .window_frames == ([2, ($sample_rate * .window_ms / 1000 | floor)] | max)
  and all([.thresholds.max_boundary_step,
           .thresholds.max_boundary_to_local_p99_ratio,
           .thresholds.max_boundary_to_attack_rms_ratio][]; positive_number)
  and all([.boundary_step, .local_adjacent_step_p99,
           .boundary_to_local_p99_ratio, .post_boundary_attack_rms,
           .boundary_to_attack_rms_ratio][]; nonnegative_number)
  # Rust rejects a discontinuity only when ALL three strict maxima are exceeded.
  # A natural strong attack alone is not a click. Equality passes.
  and ((.boundary_step > .thresholds.max_boundary_step
    and .boundary_to_local_p99_ratio > .thresholds.max_boundary_to_local_p99_ratio
    and .boundary_to_attack_rms_ratio > .thresholds.max_boundary_to_attack_rms_ratio) | not);

def alpha_return_correlation_ok:
  # Pre-RIOTBOX-1420 V1 manifests did not export this field. This is the exact
  # old Rust f32 threshold promoted into JSON, NOT a fallback for invalid values.
  (if has("max_hook_to_changed_return_correlation") then
    .max_hook_to_changed_return_correlation
   else 0.9850000143051147 end) as $maximum
  | ($maximum | positive_number) and $maximum <= 1
  and (.hook_to_changed_return_correlation | finite_number)
  and (.hook_to_changed_return_correlation | fabs) <= $maximum;

def exact_source_format_ok($graph):
  (.sample_rate | positive_integer)
  and (.channel_count | positive_integer)
  and (.source.sample_rate | positive_integer)
  and (.source.channel_count | positive_integer)
  and .source.sample_rate == $graph.source.sample_rate
  and .source.channel_count == $graph.source.channel_count;
