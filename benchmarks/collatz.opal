def collatz_steps(start: Int) -> Int
  let mut n: Int = start
  let mut steps: Int = 0
  while n != 1
    if n - (n / 2) * 2 == 0
      n = n / 2
    else
      n = 3 * n + 1
    end
    steps = steps + 1
  end
  return steps
end

def main()
  let mut i: Int = 1
  let mut total: Int = 0
  while i <= 300000
    total = total + collatz_steps(i)
    i = i + 1
  end
  puts total
end
