# Plan

1. Replace `succession_change_key` with `happens_after`.
2. `:2447`, `:7460`, `:10888` — sort lexicographically by `predecessor_id`; rewrite the
   strict-sort message.
3. `:7519`, `:14722`, `:14842` — compare creation time.
4. Three tests: the five-digit sort agreement, the backwards-predecessor refusal, and the
   ordinary-direction control. Plus a unit test that equal timestamps still yield a strict
   total order.
5. Measure digest exposure across the archive before landing, and record the numbers.
6. Discriminate against a separate checkout.
