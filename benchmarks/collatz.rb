def collatz_steps(start)
  n = start
  steps = 0
  while n != 1
    if n - (n / 2) * 2 == 0
      n = n / 2
    else
      n = 3 * n + 1
    end
    steps += 1
  end
  steps
end

total = 0
i = 1
while i <= 300_000
  total += collatz_steps(i)
  i += 1
end
puts total
